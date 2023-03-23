use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::sync::OnceCell;

static DB_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_db_connection_pool() -> &'static DatabaseConnection {
    DB_CONNECTION
        .get_or_init(|| async {
            let db_uri = std::env::var("DATABASE_URL")
                .unwrap_or("postgres://demo-user:demo-password@localhost:5432/demo".to_owned());
            let mut opt = ConnectOptions::new(db_uri);
            opt.max_connections(100)
                .min_connections(5)
                .connect_timeout(Duration::from_secs(5))
                .acquire_timeout(Duration::from_secs(5))
                .idle_timeout(Duration::from_secs(100))
                .max_lifetime(Duration::from_secs(100))
                .sqlx_logging(true)
                .set_schema_search_path("public".to_owned())
                .sqlx_logging_level(log::LevelFilter::Info); // Setting default PostgreSQL schema

            Database::connect(opt)
                .await
                .expect("connect database error")
        })
        .await
}
