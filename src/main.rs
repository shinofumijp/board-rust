#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use bcrypt;
use board_rust::db;
use board_rust::models::post::Post;
use board_rust::models::user::{NewUser, User, UserForm};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket::response::{Flash, Redirect};
use rocket::{get, post};
use rocket_contrib::templates::Template;
use serde_json;
use std::collections::HashMap;

fn main() {
    rocket::ignite()
        .mount("/", routes![index, new_user, create_user,])
        .attach(Template::fairing())
        .launch();
}

#[get("/")]
fn index(cookies: Cookies) -> Result<Template, Redirect> {
    let user = check_cookie(cookies);
    if user.is_none() {
        return Err(Redirect::to("/users/new"));
    }
    let mut context = HashMap::<String, String>::new();

    use board_rust::schema::posts::dsl::*;
    let mut conn = db::establish_connection().get().unwrap();

    context.insert("user".to_string(), serde_json::to_string(&user).unwrap());
    let post_list = posts
        .order(published_at.desc())
        .limit(10)
        .load::<Post>(&mut conn)
        .expect("Error loading posts");

    context.insert(
        "posts".to_string(),
        serde_json::to_string(&post_list).unwrap(),
    );

    println!("{:?}", context);

    Ok(Template::render("index", &context))
}

fn check_cookie(mut cookies: Cookies) -> Option<User> {
    use board_rust::schema::users::dsl::*;

    let cookie = cookies.get_private("user_id");
    if cookie.is_none() {
        return None;
    }

    let user_id = cookie.unwrap().value().parse::<i32>().unwrap();

    let mut conn = db::establish_connection().get().unwrap();
    let user = users.filter(id.eq(user_id)).first::<User>(&mut conn);

    match user {
        Ok(u) => Some(u),
        Err(_) => {
            cookies.remove_private(Cookie::named("user_id"));
            None
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
    use board_rust::schema::users;

    let password_hash = bcrypt::hash(&user_form.password, bcrypt::DEFAULT_COST).unwrap();
    let mut conn = db::establish_connection().get().unwrap();

    let new_user = NewUser {
        name: &user_form.name.to_string(),
        email: &user_form.email.to_string(),
        password: &password_hash,
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .map_err(|_| Flash::error(Redirect::to("/users/new"), "Failed to create user."))?;

    cookies.add_private(Cookie::new("user_id", user.id.to_string()));

    Ok(Redirect::to("/"))
}
