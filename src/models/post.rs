use crate::schema::posts;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use rocket::request::FromForm;
use serde::Serialize;

#[derive(Queryable, Serialize)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: i32,
    pub content: String,
    pub user_id: i32,
    pub published_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub content: &'a str,
    pub user_id: i32,
    pub published_at: NaiveDateTime,
}

#[derive(FromForm, Debug)]
pub struct PostForm {
    pub content: String,
}
