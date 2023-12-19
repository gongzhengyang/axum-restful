use aide::OperationOutput;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use snafu::{Location, Snafu};
use std::fmt::Debug;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum AppError {
    #[snafu(display("internal server error"))]
    InternalServer { location: Location },

    #[snafu(display("create instance error"))]
    CreateInstance { source: DbErr, location: Location },

    #[snafu(display("instance not found with primary key: {}", pk))]
    PrimaryKeyNotFound { pk: u64, location: Location },

    #[snafu(display("query database failed: {}", source))]
    OperateDatabase { source: DbErr, location: Location },

    #[snafu(display("option value is none"))]
    OptionValueNone { location: Location },

    #[snafu(display("unkonwn error"))]
    Unknown,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = match self {
            AppError::PrimaryKeyNotFound { .. } => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        tracing::error!("error happened: {self:?}");
        (
            status_code,
            Json(serde_json::json!({
                "message": format!("{}", self)
            })),
        )
            .into_response()
    }
}

impl OperationOutput for AppError {
    type Inner = Self;
}

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub struct BoxedError {
    inner: Box<dyn Send + Sync + std::error::Error>,
}

impl BoxedError {
    pub fn new<E: Send + Sync + 'static + std::error::Error>(err: E) -> Self {
        Self {
            inner: Box::new(err),
        }
    }
}

impl std::fmt::Display for BoxedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for BoxedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}
