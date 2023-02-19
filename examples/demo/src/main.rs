use axum_restful::get_db_connection_pool;
use sea_orm::*;
use sea_orm::sea_query::Expr;
use demo::entities::student::{self, Entity as Student};


#[tokio::main]
async fn main() {

    let s = student::ActiveModel::from_json(serde_json::json!({
        "name": "a",
        "region": "china",
        "age": "test"
    })).unwrap();
    let db = get_db_connection_pool().await;
    let result = s.insert(db).await;
    println!("insert {result:?}---");

    let students: Vec<student::Model> = Student::find()
        .order_by_asc(student::Column::Name)
        .all(db)
        .await.unwrap();
    println!("select {students:?}");

    let update_results = Student::update_many()
        .col_expr(student::Column::Name, Expr::value("changed"))
        .exec(db)
        .await
        .unwrap();
    println!("updated results: {update_results:?}");

}
