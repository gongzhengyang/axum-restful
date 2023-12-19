use aide::gen::GenContext;
use aide::openapi::Operation;
use std::fmt::Debug;

use aide::OperationOutput;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use sea_orm::DbErr;
use serde::Serialize;
use snafu::{Location, Snafu};

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

#[derive(Debug, JsonSchema, Serialize)]
pub struct ErrorMessage {
    pub message: String,
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
            Json(ErrorMessage {
                message: format!("{}", self),
            }),
        )
            .into_response()
    }
}

impl OperationOutput for AppError {
    type Inner = Self;

    fn operation_response(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Option<aide::openapi::Response> {
        <Json<ErrorMessage> as OperationOutput>::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        let mut resp = vec![];
        if let Some(response) = <() as OperationOutput>::operation_response(ctx, operation) {
            resp.push((Some(204), response));
        }
        if let Some(response) = <AppError as OperationOutput>::operation_response(ctx, operation) {
            resp.push((Some(500), response));
        }
        resp
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
