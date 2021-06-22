use std::io::Cursor;

use crate::errors::AppError;
use rocket::{
    data::{FromData, Outcome, ToByteUnit},
    http::{ContentType, Status},
    response::Responder,
    serde::{json::serde_json, Deserialize, Serialize},
    Response,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Task {
    pub(crate) id: u64,
    pub(crate) title: String,
    pub(crate) details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InsertTask {
    pub(crate) non_empty_title: String,
    pub(crate) details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UpdateTask {
    pub(crate) id: u64,
    pub(crate) new_title: String,
    pub(crate) details: String,
}

// impl<'r> Responder<'r, 'static> for Task {
//     fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
//         let task_string = serde_json::to_string_pretty(&self).unwrap();
//         Response::build()
//             .sized_body(task_string.len(), Cursor::new(task_string))
//             .header(ContentType::JSON)
//             .ok()
//     }
// }

#[rocket::async_trait]
impl<'r> FromData<'r> for InsertTask {
    type Error = AppError;

    async fn from_data(
        req: &'r rocket::Request<'_>,
        data: rocket::Data<'r>,
    ) -> rocket::data::Outcome<'r, Self> {
        let limit = req.limits().get("insert_task").unwrap_or(512.bytes());
        let as_string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Outcome::Failure((Status::PayloadTooLarge, AppError::Internal)),
            Err(fail) => {
                return Outcome::Failure((Status::InternalServerError, AppError::Internal))
            }
        };

        let insert_task: InsertTask = serde_json::from_str(&as_string).unwrap();
        if insert_task.non_empty_title.trim().is_empty() {
            return Outcome::Failure((Status::BadRequest, AppError::EmptyTitle));
        }

        Outcome::Success(insert_task)
    }
}

#[rocket::async_trait]
impl<'r> FromData<'r> for UpdateTask {
    type Error = AppError;

    async fn from_data(
        req: &'r rocket::Request<'_>,
        data: rocket::Data<'r>,
    ) -> rocket::data::Outcome<'r, Self> {
        let limit = req.limits().get("update_task").unwrap_or(512.bytes());
        let as_string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Outcome::Failure((Status::PayloadTooLarge, AppError::Internal)),
            Err(fail) => {
                return Outcome::Failure((Status::InternalServerError, AppError::Internal))
            }
        };

        let update_task: UpdateTask = serde_json::from_str(&as_string).unwrap();

        if update_task.new_title.trim().is_empty() {
            return Outcome::Failure((Status::BadRequest, AppError::EmptyTitle));
        }

        Outcome::Success(update_task)
    }
}
