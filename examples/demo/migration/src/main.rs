use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    let db = axum_restful::get_db_connection_pool().await;
    migration::Migrator::refresh(db).await.unwrap();
}
