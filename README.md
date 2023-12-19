## `axum-restful`

A restful framework based on `axum` and `sea-orm`. Inspired by `django-rest-framework`.

The goal of the project is to build an enterprise-level production framework.

## Features

- a Trait for the `struct` generated by `sea-orm` to provide with GET, PUT, DELETE methods
- `tls` support
- `prometheus` metrics and metrics server
- `graceful shutdown`support
- `swagger document` generate based on [`aide`](https://github.com/tamasfe/aide)  

## Quick start

A full example is exists at `axum-restful/examples/demo`.

First, you can create a new crate like `cargo new axum-restful-demo`.

#### Build a database service

You should have a database service before. It is recommended to use `postgresql` database.

you can use docker and docker compose to start a `postgresql`

create a `compose.yaml`  in the same directory as `Cargo.toml`

```yaml
services:
  postgres:
    image: postgres:15-bullseye
    container_name: demo-postgres
    restart: always
    volumes:
      - demo-postgres:/var/lib/postgresql/data
    ports:
      - "127.0.0.1:5432:5432"
    environment:
      - POSTGRES_DB=${POSTGRES_DB}
      - POSTGRES_USER=${POSTGRES_USER}
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}

volumes:
  demo-postgres: {}
```

a `.env`file like 

```
# config the base pg connect params
POSTGRES_DB=demo
POSTGRES_USER=demo-user
POSTGRES_PASSWORD=demo-password

# used by axum-restful framework to specific a database connection
DATABASE_URL=postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:5432/${POSTGRES_DB}
```

finally, you can build a service with `docker compose up -d`

#### Write and migrate a migration

For more details, please refer to the [`sea-orm`](https://www.sea-ql.org/SeaORM/docs/index/) documentation.

Install the `sea-orm-cli` with `cargo`

```shell
$ cargo install sea-orm-cli
```

Configure dependencies and workspace in `Cargo.toml`

```toml
[package]
name = "demo"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
[workspace]
members = [".", "migration"]

[dependencies]
aide = "0.13"
axum = "0.7"
axum-restful = "0.5"
chrono = "0.4"
migration = { path = "./migration" }
once_cell = "1"
schemars = { version = "0.8", features = ["chrono"] }
sea-orm = { version = "0.12", features = ["macros", "sqlx-postgres", "runtime-tokio-rustls"] }
sea-orm-migration = { version = "0.12", features = ["sqlx-postgres", "runtime-tokio-rustls",] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

Setup the migration directory in `./migration`

```shell
$ sea-orm-cli migrate init
```

project structure changed into 

```
├── Cargo.lock
├── Cargo.toml
├── compose.yaml
├── migration
│   ├── Cargo.toml
│   ├── README.md
│   └── src
│       ├── lib.rs
│       ├── m20220101_000001_create_table.rs
│       └── main.rs
└── src
    └── main.rs
```

edit the `m20****_******_create_table.rs` file blow `./migration/src`

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Student::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Student::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Student::Name).string().not_null())
                    .col(ColumnDef::new(Student::Region).string().not_null())
                    .col(ColumnDef::new(Student::Age).small_integer().not_null())
                    .col(ColumnDef::new(Student::CreateTime).date_time().not_null())
                    .col(ColumnDef::new(Student::Score).double().not_null())
                    .col(
                        ColumnDef::new(Student::Gender)
                            .boolean()
                            .not_null()
                            .default(Expr::value(true)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(Student::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Student {
    Table,
    Id,
    Name,
    Region,
    Age,
    CreateTime,
    Score,
    Gender,
}
```

edit `migration/Cargo.toml` to add dependencies 

```toml
[dependencies]
...
axum-restful = "0.5"
```

edit `migration/src/main.rs`  to specific a database connection an migrate 

```rust
use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    // cli::run_cli(migration::Migrator).await;
    let db = axum_restful::get_db_connection_pool().await;
    migration::Migrator::up(db, None).await.unwrap();
}
```

migrate the migration files

```shell
$ cd migration
$ cargo run
```

finally, you can see two tables named `sql_migrations` and `student`generated.

#### Generate entities

at the project root path 

```shell
$ sea-orm-cli generate entity -o src/entities
```

will generate entities configure and code, now project structure changed into 

```
├── Cargo.lock
├── Cargo.toml
├── compose.yaml
├── migration
│   ├── Cargo.toml
│   ├── README.md
│   └── src
│       ├── lib.rs
│       ├── m20220101_000001_create_table.rs
│       └── main.rs
└── src
    ├── entities
    │   ├── mod.rs
    │   ├── prelude.rs
    │   └── student.rs
    └── main.rs
```

edit the `src/entities/student.rs` to add derive `Default, Serialize, Deserialize` 

```rust
//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.0

use schemars::JsonSchema;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, JsonSchema, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "student")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub region: String,
    pub age: i16,
    pub create_time: DateTime,
    #[sea_orm(column_type = "Double")]
    pub score: f64,
    pub gender: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

```

edit `src/main.rs`

```rust
use schemars::JsonSchema;
use sea_orm_migration::prelude::MigratorTrait;
use tokio::net::TcpListener;

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

    aide::gen::on_error(|error| {
        tracing::error!("swagger api gen error: {error}");
    });
    aide::gen::extract_schemas(true);

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
    let app = StudentView::http_router_with_swagger(path, StudentView::model_api_router()).await.unwrap();

    let addr = "0.0.0.0:3000";
    tracing::info!("listen at {addr}");
    tracing::info!("visit http://127.0.0.1:3000/docs/swagger/ for swagger api");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

```

`StudentView impl the ModelView<T>`, the `T` is `student::ActiveModel` that represent the `student table configure` in the database, if will has full HTTP methods with GET, POST, PUT, DELETE.

you can see the server is listen at port 3000

#### Verify the service

#### Swagger

if you `impl axum_restful::swagger::SwaggerGenerator` above, then you can visit `http://127.0.0.1:3000/docs/swagger/`  at your browser, you will see a swagger document is generated

![swagger-ui](https://github.com/gongzhengyang/axum-restful/blob/main/statics/swagger-ui-demo.png)

## License


Licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.