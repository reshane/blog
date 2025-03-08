use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    LoginFailure,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("->> {:<12} - {self:?}", "INTO_RES");

        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
    }
}
