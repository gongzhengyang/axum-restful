use http::Uri;
use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::sync::OnceCell;

static DB_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

/// only init once
/// will read some env keys if exists.
///
/// env **`DATABASE_URL`** to specific the database connect options, default is **`postgres://demo-user:demo-password@localhost:5432/demo`**
///
/// env **`SQLX_LOGGING`** to config the sqlx logging, default is **`false`**, can chaneg into **`true`**
///
/// env **`SQLX_LOGGING_LEVEL`** to config the sqlx logging level, default is `info`, choices is `debug`, `info`, `warning`..., same as `log::Level`.
pub async fn get_db_connection_pool() -> &'static DatabaseConnection {
    DB_CONNECTION
        .get_or_init(|| async {
            let db_uri = std::env::var("DATABASE_URL")
                .unwrap_or("postgres://demo-user:demo-password@localhost:5432/demo".to_owned());
            let parsed_uri = db_uri.parse::<Uri>().unwrap();
            tracing::info!(
                "intial {} connection at {}:{} database {}",
                parsed_uri.scheme_str().unwrap(),
                parsed_uri.host().unwrap(),
                parsed_uri.port().unwrap(),
                parsed_uri.path()
            );
            let sqlx_logging = std::env::var("SQLX_LOGGING")
                .unwrap_or("false".to_owned())
                .parse::<bool>()
                .unwrap_or(false);
            let sqlx_logging_level = std::env::var("SQLX_LOGGING_LEVEL")
                .unwrap_or("info".to_owned())
                .parse::<log::LevelFilter>()
                .unwrap();
            let opt = ConnectOptions::new(db_uri)
                .max_connections(100)
                .min_connections(5)
                .connect_timeout(Duration::from_secs(5))
                .acquire_timeout(Duration::from_secs(5))
                .idle_timeout(Duration::from_secs(100))
                .max_lifetime(Duration::from_secs(100))
                .sqlx_logging(sqlx_logging)
                .set_schema_search_path("public".to_owned())
                .sqlx_logging_level(sqlx_logging_level)
                .to_owned(); // Setting default PostgreSQL schema
            Database::connect(opt)
                .await
                .expect("connect database error")
        })
        .await
}
