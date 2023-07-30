use crate::db;
use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::request::FromForm;
use validator::ValidationError;
use validator_derive::Validate;

#[derive(FromForm, Validate)]
pub struct UserForm {
    #[validate(length(min = 3), custom = "validate_name_unique")]
    pub name: String,
    #[validate(email, length(min = 1), custom = "validate_email_unique")]
    pub email: String,
    #[validate(length(min = 8), custom = "validate_password_ascii")]
    pub password: String,
}

#[derive(Queryable, Identifiable, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a String,
    pub email: &'a String,
    pub password: &'a String,
}

fn validate_email_unique(given_email: &str) -> Result<(), ValidationError> {
    use crate::schema::users::dsl::*;

    let mut connection = db::establish_connection().get().unwrap();
    let result = users
        .filter(email.eq(given_email))
        .load::<User>(&mut connection);

    match result {
        Ok(_) => Err(ValidationError::new("Email is already taken")),
        Err(diesel::NotFound) => Ok(()),
        Err(err) => panic!("Error loading user: {}", err),
    }
}

fn validate_name_unique(given_name: &str) -> Result<(), ValidationError> {
    use crate::schema::users::dsl::*;

    let mut connection = db::establish_connection().get().unwrap();
    let results = users
        .filter(name.eq(given_name))
        .limit(1)
        .load::<User>(&mut connection)
        .expect("Error loading users");
    if results.len() > 0 {
        return Err(ValidationError::new("Name is already taken"));
    }
    Ok(())
}

fn validate_password_ascii(password: &str) -> Result<(), ValidationError> {
    if password.is_ascii() {
        Ok(())
    } else {
        Err(ValidationError::new("non_ascii"))
    }
}
