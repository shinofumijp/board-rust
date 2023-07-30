use chrono::NaiveDateTime;
use diesel::prelude::*;
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
