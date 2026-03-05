use diesel::r2d2::{self, ConnectionManager};
use diesel::{sql_query, RunQueryDsl, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn establish_pool() -> DbPool {
    let database_url = dotenvy::var("DATABASE_URL").unwrap_or_else(|_| "library.db".to_string());
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool")
}

pub fn initialize_db(pool: &DbPool) {
    let mut conn = pool.get().expect("Failed to get DB connection");
    sql_query("PRAGMA foreign_keys = ON").execute(&mut conn).ok();
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}
