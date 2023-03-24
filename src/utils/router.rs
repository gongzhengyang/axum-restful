use axum::http::StatusCode;
use axum::response::IntoResponse;

pub fn handle_not_found() -> impl IntoResponse{
    (StatusCode::NOT_FOUND, "nothing to see here")
}
