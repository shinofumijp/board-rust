use crate::db;
use crate::models::post::Post;
use crate::models::user::User;
use crate::schema::likes;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use serde::Serialize;

#[derive(Queryable, Serialize, Selectable, Identifiable, PartialEq, Associations, Debug)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Post))]
#[diesel(table_name = likes)]
pub struct Like {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = likes)]
struct NewLike {
    pub user_id: i32,
    pub post_id: i32,
}

impl Like {
    pub fn create(param_user_id: i32, param_post_id: i32) -> Result<Like, Error> {
        use crate::schema::likes::dsl::*;
        // Create the new like.
        let like = NewLike {
            user_id: param_user_id,
            post_id: param_post_id,
        };

        let mut connection = db::establish_connection().get().unwrap();
        diesel::insert_into(likes)
            .values(&like)
            .get_result(&mut connection)
    }

    pub fn delete(like_id: i32) -> Result<usize, Error> {
        use crate::schema::likes::dsl::*;

        let mut connection = db::establish_connection().get().unwrap();
        diesel::delete(likes.filter(id.eq(like_id))).execute(&mut connection)
    }

    pub fn get_by_user_and_post(query_user_id: i32, query_post_id: i32) -> Option<Like> {
        use crate::schema::likes::dsl::*;

        let mut connection = db::establish_connection().get().unwrap();
        likes
            .filter(user_id.eq(query_user_id))
            .filter(post_id.eq(query_post_id))
            .first::<Like>(&mut connection)
            .ok()
    }
}
