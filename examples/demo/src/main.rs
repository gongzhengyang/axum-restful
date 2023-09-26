use schemars::JsonSchema;
use sea_orm_migration::prelude::MigratorTrait;

use axum_restful::swagger::SwaggerGeneratorExt;
use axum_restful::views::ModelViewExt;

use crate::entities::student;

mod check;
mod entities;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let db = axum_restful::get_db_connection_pool().await;
    let _ = migration::Migrator::down(db, None).await;
    migration::Migrator::up(db, None).await.unwrap();
    tracing::info!("migrate success");

    /// student
    #[derive(JsonSchema)]
    struct StudentView;

    impl ModelViewExt<student::ActiveModel> for StudentView {
        fn order_by_desc() -> student::Column {
            student::Column::Id
        }
    }

    let path = "/api/student";
    let app = StudentView::http_router(path);
    check::check_curd_operate_correct(app.clone(), path, db).await;

    // if you want to generate swagger docs
    // impl OperationInput and SwaggerGenerator and change app into http_routers_with_swagger
    impl aide::operation::OperationInput for student::Model {}
    impl axum_restful::swagger::SwaggerGeneratorExt<student::ActiveModel> for StudentView {}
    let app = StudentView::http_router_with_swagger(path, StudentView::model_api_router());

    let addr = "0.0.0.0:3000";
    tracing::info!("listen at {addr}");
    tracing::info!("visit http://127.0.0.1:3000/docs/swagger/ for swagger api");
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap()
}
