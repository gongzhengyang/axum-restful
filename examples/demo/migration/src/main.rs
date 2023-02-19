use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    // cli::run_cli(migration::Migrator).await;
    let db = axum_restful::db::get_db_connection_pool().await;
    migration::Migrator::up(db, None).await.unwrap();
}
