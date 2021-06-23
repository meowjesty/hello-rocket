use crate::errors::AppError;
use rocket::{
    data::{FromData, Outcome, ToByteUnit},
    http::Status,
    serde::{json::serde_json, Deserialize, Serialize},
};
use sqlx::{FromRow, SqlitePool};

const FIND_BY_PATTERN: &'static str = include_str!("./../queries/find_by_pattern.sql");
const FIND_ONGOING: &'static str = include_str!("./../queries/find_ongoing.sql");
const FIND_ALL: &'static str = include_str!("./../queries/find_all.sql");
const FIND_BY_ID: &'static str = include_str!("./../queries/find_by_id.sql");
const INSERT: &'static str = include_str!("./../queries/insert.sql");
const UPDATE: &'static str = include_str!("./../queries/update.sql");
const DELETE: &'static str = include_str!("./../queries/delete.sql");

const COMPLETED: &'static str = include_str!("./../queries/done.sql");
const UNDO: &'static str = include_str!("./../queries/undo.sql");

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct Task {
    pub(crate) id: i64,
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
    pub(crate) id: i64,
    pub(crate) new_title: String,
    pub(crate) details: String,
}

impl InsertTask {
    pub(crate) async fn insert(&self, db_pool: &SqlitePool) -> Result<Task, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(INSERT)
            .bind(&self.non_empty_title.to_string())
            .bind(&self.details)
            .execute(&mut connection)
            .await?;

        let id = result.last_insert_rowid();

        let new_task = Task {
            id,
            title: self.non_empty_title.to_owned(),
            details: self.details.to_owned(),
        };

        Ok(new_task)
    }
}

impl UpdateTask {
    pub(crate) async fn update(&self, db_pool: &SqlitePool) -> Result<u64, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(UPDATE)
            .bind(&self.new_title)
            .bind(&self.details)
            .bind(&self.id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }
}

impl Task {
    pub(crate) async fn delete(db_pool: &SqlitePool, task_id: i64) -> Result<u64, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(DELETE)
            .bind(task_id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }

    pub(crate) async fn done(pool: &SqlitePool, task_id: i64) -> Result<i64, AppError> {
        let mut connection = pool.acquire().await?;
        let result = sqlx::query(COMPLETED)
            .bind(task_id)
            .execute(&mut connection)
            .await?;

        Ok(result.last_insert_rowid())
    }

    pub(crate) async fn undo(db_pool: &SqlitePool, task_id: i64) -> Result<u64, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(UNDO)
            .bind(task_id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }

    pub(crate) async fn find_all(db_pool: &SqlitePool) -> Result<Vec<Self>, AppError> {
        let result = sqlx::query_as(FIND_ALL).fetch_all(db_pool).await?;

        Ok(result)
    }

    pub(crate) async fn find_ongoing(db_pool: &SqlitePool) -> Result<Vec<Self>, AppError> {
        let result = sqlx::query_as(FIND_ONGOING).fetch_all(db_pool).await?;

        Ok(result)
    }

    pub(crate) async fn find_by_pattern(
        db_pool: &SqlitePool,
        search_pattern: &str,
    ) -> Result<Vec<Self>, AppError> {
        let result = sqlx::query_as(FIND_BY_PATTERN)
            .bind(search_pattern)
            .fetch_all(db_pool)
            .await?;

        Ok(result)
    }

    pub(crate) async fn find_by_id(
        db_pool: &SqlitePool,
        task_id: i64,
    ) -> Result<Option<Self>, AppError> {
        let result = sqlx::query_as(FIND_BY_ID)
            .bind(task_id)
            .fetch_optional(db_pool)
            .await?;

        Ok(result)
    }
}

#[rocket::async_trait]
impl<'r> FromData<'r> for InsertTask {
    type Error = AppError;

    async fn from_data(
        req: &'r rocket::Request<'_>,
        data: rocket::Data<'r>,
    ) -> rocket::data::Outcome<'r, Self> {
        let limit = req.limits().get("self").unwrap_or(512.bytes());
        let as_string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Outcome::Failure((Status::PayloadTooLarge, AppError::Internal)),
            Err(fail) => {
                return Outcome::Failure((Status::InternalServerError, AppError::IO(fail)));
            }
        };

        let insert: InsertTask = serde_json::from_str(&as_string).unwrap();
        if insert.non_empty_title.trim().is_empty() {
            // TODO(alex) [high] 2021-06-22: This is where the error is returned from (the actual
            // response), read the note in errors.rs.
            return Outcome::Failure((Status::UnprocessableEntity, AppError::EmptyTitle));
        }

        Outcome::Success(insert)
    }
}

#[rocket::async_trait]
impl<'r> FromData<'r> for UpdateTask {
    type Error = AppError;

    async fn from_data(
        req: &'r rocket::Request<'_>,
        data: rocket::Data<'r>,
    ) -> rocket::data::Outcome<'r, Self> {
        let limit = req.limits().get("self").unwrap_or(512.bytes());
        let as_string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Outcome::Failure((Status::PayloadTooLarge, AppError::Internal)),
            Err(fail) => {
                return Outcome::Failure((Status::InternalServerError, AppError::IO(fail)))
            }
        };

        let update: UpdateTask = serde_json::from_str(&as_string).unwrap();

        if update.new_title.trim().is_empty() {
            // TODO(alex) [high] 2021-06-22: This is where the error is returned from (the actual
            // response), read the note in errors.rs.
            return Outcome::Failure((Status::UnprocessableEntity, AppError::EmptyTitle));
        }

        Outcome::Success(update)
    }
}
