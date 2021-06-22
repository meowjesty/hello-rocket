use std::io::Cursor;

use rocket::{
    http::{hyper::StatusCode, ContentType, Status},
    response::Responder,
    serde::{json::serde_json, Deserialize, Serialize},
    Response,
};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Error)]
pub(crate) enum AppError {
    #[error("`title` field of `Task` cannot be empty!")]
    EmptyTitle,

    #[error("`{0}` id not found!")]
    IdNotFound(u64),

    #[error("Internal server error!")]
    Internal,
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status_code = match self {
            AppError::EmptyTitle => 400,
            AppError::IdNotFound(_) => 404,
            AppError::Internal => 500,
        };

        let error_string = serde_json::to_string_pretty(&self).unwrap();
        Response::build()
            .sized_body(error_string.len(), Cursor::new(error_string))
            .header(ContentType::Text)
            .status(Status::new(status_code))
            .ok()
    }
}
