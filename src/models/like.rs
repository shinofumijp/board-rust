use diesel::prelude::*;
use diesel::sql_types::Timestamp;

#[derive(Queryable)]
#[diesel(table_name = likes)]
pub struct Like {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
