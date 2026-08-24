use diesel::connection::SimpleConnection;
use diesel::r2d2::{self, ConnectionManager, CustomizeConnection};
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Enables foreign-key enforcement (and a busy timeout) on every pooled
/// connection. SQLite's `foreign_keys` pragma is per-connection and defaults
/// off, so it must be set on each acquire rather than once at startup.
#[derive(Debug)]
struct ConnectionOptions;

impl CustomizeConnection<SqliteConnection, r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), r2d2::Error> {
        conn.batch_execute("PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")
            .map_err(r2d2::Error::QueryError)
    }
}

pub fn establish_pool() -> DbPool {
    let database_url = dotenvy::var("DATABASE_URL").unwrap_or_else(|_| "library.db".to_string());
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .connection_customizer(Box::new(ConnectionOptions))
        .build(manager)
        .expect("Failed to create DB pool")
}

pub fn initialize_db(pool: &DbPool) {
    let mut conn = pool.get().expect("Failed to get DB connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}
