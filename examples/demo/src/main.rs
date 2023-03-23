mod entities;

use axum::{Json, Router};
use axum_restful::get_db_connection_pool;
use axum_restful::views::ModelView;
use entities::student;
use sea_orm::sea_query::Expr;
use sea_orm::*;

#[tokio::main]
async fn main() {
    let s = student::ActiveModel::from_json(serde_json::json!({
        "name": "a",
        "region": "china",
        "age": "test"
    }))
    .unwrap();
    let db = get_db_connection_pool().await;
    let result = s.insert(db).await;
    println!("insert {result:?}---");
    //
    let students: Vec<student::Model> = student::Entity::find()
        .order_by_asc(student::Column::Name)
        .all(db)
        .await
        .unwrap();
    println!("select {students:?}");
    //
    // let update_results = student::Entity::update_many()
    //     .col_expr(student::Column::Name, Expr::value("changed"))
    //     .exec(db)
    //     .await
    //     .unwrap();
    // println!("updated results: {update_results:?}");
    //
    // let delete_results = student::Entity::delete_many()
    //     .exec(db)
    //     .await
    //     .unwrap();
    // println!("{:?}", delete_results);
    let id = 1;
    let student = student::Entity::find_by_id(id).one(db).await.unwrap();
    println!("{student:?}");
    struct StudentView;

    impl ModelView<student::ActiveModel> for StudentView {}

    let app = Router::new().nest("/api", StudentView::get_http_routes());

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    // let data = serde_json::json!({
    // "name": "a",
    // "region": "china",
    // "age": "test"
    // });
    // let result = StudentView::http_post(Json(data)).await;
    // println!("{result:?}");
}
