use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    response::Responder,
    Response,
};
use thiserror::Error;

// TODO(alex) [high] 2021-06-22: Derive the `responder` and use the `status_code` attribute.
#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("`title` field of `Task` cannot be empty!")]
    EmptyTitle,

    #[error("`{0}` id not found!")]
    IdNotFound(u64),

    #[error("Internal server error!")]
    Internal,

    #[error("`{0}`")]
    IO(#[from] std::io::Error),
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let status = match self {
            AppError::EmptyTitle => Status::UnprocessableEntity,
            AppError::IdNotFound(_) => Status::NotFound,
            AppError::Internal => Status::InternalServerError,
            AppError::IO(_) => Status::InternalServerError,
        };

        let error_string = format!("{}", self);
        Response::build()
            .sized_body(error_string.len(), Cursor::new(error_string))
            .header(ContentType::Text)
            .status(status)
            .ok()
    }
}
