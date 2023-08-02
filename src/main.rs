#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use bcrypt;
use board_rust::db;
use board_rust::models::post::{NewPost, Post, PostForm};
use board_rust::models::sign_in::SignInForm;
use board_rust::models::user::{NewUser, User, UserForm};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket::response::{Flash, Redirect};
use rocket_contrib::templates::Template;
use serde_json;
use std::collections::HashMap;

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                new_user,
                create_user,
                sign_out,
                sign_in,
                sign_in_page,
                create_post,
                new_post,
                edit_post,
                update_post,
            ],
        )
        .attach(Template::fairing())
        .launch();
}

#[get("/")]
fn index(cookies: Cookies) -> Result<Template, Redirect> {
    // Check if the user has logged in.
    let user = check_cookie(cookies);
    if user.is_err() {
        return Err(Redirect::to("/users/new"));
    }

    // Create context.
    let mut context = HashMap::<String, String>::new();

    // Get list of posts.
    use board_rust::schema::posts::dsl::*;
    let mut conn = db::establish_connection().get().unwrap();
    let post_list = posts
        .order(published_at.desc())
        .limit(10)
        .load::<Post>(&mut conn)
        .expect("Error loading posts");

    // Insert user and post list into context.
    context.insert("user".to_string(), serde_json::to_string(&user).unwrap());
    context.insert(
        "posts".to_string(),
        serde_json::to_string(&post_list).unwrap(),
    );

    // Render index page.
    Ok(Template::render("index", &context))
}

fn check_cookie(mut cookies: Cookies) -> Result<User, String> {
    // Get user_id from cookie.
    let cookie = cookies.get_private("user_id");
    if cookie.is_none() {
        return Err("No user_id cookie".to_string());
    }
    let user_id = cookie.unwrap().value().parse::<i32>().unwrap();

    // Get user from database.
    use board_rust::schema::users::dsl::*;
    let mut conn = db::establish_connection().get().unwrap();
    let user = users.find(user_id).get_result::<User>(&mut conn);

    // Check if the user exists.
    match user {
        Ok(u) => Ok(u),
        Err(e) => {
            cookies.remove_private(Cookie::named("user_id"));
            Err(e.to_string())
        }
    }
}

#[get("/users/new")]
fn new_user() -> Template {
    let context = HashMap::<String, String>::new();
    Template::render("users/new", &context)
}

#[post("/users", data = "<user_form>")]
fn create_user(
    mut cookies: Cookies,
    user_form: Form<UserForm>,
) -> Result<Redirect, Flash<Redirect>> {
    // Hash the password.
    let password_hash = bcrypt::hash(&user_form.password, bcrypt::DEFAULT_COST).unwrap();

    // Get the database connection.
    let mut conn = db::establish_connection().get().unwrap();

    // Create the new user.
    let new_user = NewUser {
        name: &user_form.name.to_string(),
        email: &user_form.email.to_string(),
        password: &password_hash,
    };

    use board_rust::schema::users;

    // Insert the new user into the database.
    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .map_err(|_| Flash::error(Redirect::to("/users/new"), "Failed to create user."))?;

    // Set a cookie with the user's id.
    cookies.add_private(Cookie::new("user_id", user.id.to_string()));

    Ok(Redirect::to("/"))
}

#[get("/sign_in")]
fn sign_in_page() -> Template {
    let context = HashMap::<String, String>::new();
    Template::render("sign_in", &context)
}

#[post("/sign_in", data = "<sign_in_form>")]
fn sign_in(
    mut cookies: Cookies,
    sign_in_form: Form<SignInForm>,
) -> Result<Redirect, Flash<Redirect>> {
    // Get the database connection.
    let mut conn = db::establish_connection().get().unwrap();

    // Get the user from the database.
    use board_rust::schema::users::dsl::*;
    let user = users
        .filter(email.eq(&sign_in_form.email))
        .first::<User>(&mut conn)
        .map_err(|_| Flash::error(Redirect::to("/sign_in"), "Invalid email or password."))?;

    // Check the password.
    if bcrypt::verify(&sign_in_form.password, &user.password).unwrap() {
        // Set a cookie with the user's id.
        cookies.add_private(Cookie::new("user_id", user.id.to_string()));

        Ok(Redirect::to("/"))
    } else {
        Err(Flash::error(
            Redirect::to("/sign_in"),
            "Invalid email or password.",
        ))
    }
}

#[delete("/sign_out")]
fn sign_out(mut cookies: Cookies) -> Redirect {
    cookies.remove_private(Cookie::named("user_id"));
    Redirect::to("/")
}

#[get("/posts/new")]
fn new_post(cookies: Cookies) -> Result<Template, Redirect> {
    // Check if the user has logged in.
    let user = check_cookie(cookies);
    if user.is_err() {
        return Err(Redirect::to("/users/new"));
    }

    // Create context.
    let mut context = HashMap::<String, String>::new();

    // Insert user into context.
    context.insert("user".to_string(), serde_json::to_string(&user).unwrap());

    // Render new post page.
    Ok(Template::render("posts/new", &context))
}

#[post("/posts", data = "<post_form>")]
fn create_post(cookies: Cookies, post_form: Form<PostForm>) -> Result<Redirect, Flash<Redirect>> {
    // Check if the user has logged in.
    let user = check_cookie(cookies);
    if user.is_err() {
        return Err(Flash::error(
            Redirect::to("/users/new"),
            "You must sign in to post.",
        ));
    }

    // Get the database connection.
    let mut conn = db::establish_connection().get().unwrap();

    // Create the new post.
    let new_post = NewPost {
        content: &post_form.content.to_string(),
        published_at: chrono::Local::now().naive_local(),
        user_id: user.unwrap().id,
    };

    use board_rust::schema::posts;

    // Insert the new post into the database.
    diesel::insert_into(posts::table)
        .values(&new_post)
        .execute(&mut conn)
        .map_err(|_| Flash::error(Redirect::to("/posts/new"), "Failed to create post."))?;

    Ok(Redirect::to("/"))
}

#[get("/posts/<post_id>/edit")]
fn edit_post(cookies: Cookies, post_id: i32) -> Result<Template, Flash<Redirect>> {
    // Check if the user has logged in.
    let user: User;
    match check_cookie(cookies) {
        Ok(u) => user = u,
        Err(e) => return Err(Flash::error(Redirect::to("/users/new"), e)),
    };

    // Get the database connection.
    let mut conn = db::establish_connection().get().unwrap();

    // Get the post from the database.
    use board_rust::schema::posts::dsl::*;
    let post = posts
        .filter(id.eq(post_id))
        .filter(user_id.eq(user.id))
        .first::<Post>(&mut conn)
        .map_err(|_| Flash::error(Redirect::to("/"), "Failed to update post."))?;

    // Check if the user is the author of the post.
    if post.user_id != user.id {
        return Err(Flash::error(
            Redirect::to("/"),
            "You are not the author of the post.",
        ));
    }

    // Create context.
    let mut context = HashMap::<String, String>::new();

    // Insert user and post into context.
    context.insert("user".to_string(), serde_json::to_string(&user).unwrap());
    context.insert("post".to_string(), serde_json::to_string(&post).unwrap());
    context.insert("update_path".to_string(), format!("/posts/{}", post.id));

    // Render edit post page.
    Ok(Template::render("posts/edit", &context))
}

#[post("/posts/<post_id>", data = "<post_form>")]
fn update_post(
    cookies: Cookies,
    post_id: i32,
    post_form: Form<PostForm>,
) -> Result<Redirect, Flash<Redirect>> {
    // Check if the user has logged in.
    let user: User;
    match check_cookie(cookies) {
        Ok(u) => user = u,
        Err(e) => return Err(Flash::error(Redirect::to("/users/new"), e)),
    };

    // Get the database connection.
    let mut conn = db::establish_connection().get().unwrap();

    // Get the post from the database.
    use board_rust::schema::posts::dsl::*;
    let post = posts
        .filter(id.eq(post_id))
        .filter(user_id.eq(user.id))
        .first::<Post>(&mut conn)
        .map_err(|_| Flash::error(Redirect::to("/"), "Failed to update post."))?;

    // Update the post.
    diesel::update(posts.find(id))
        .set(content.eq(&post_form.content.to_string()))
        .execute(&mut conn)
        .map_err(|_| {
            Flash::error(
                Redirect::to(format!("/posts/{}/edit", post.id)),
                "Failed to update post.",
            )
        })?;

    Ok(Redirect::to("/"))
}
