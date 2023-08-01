use crate::config::CONFIG;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(&CONFIG.database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
