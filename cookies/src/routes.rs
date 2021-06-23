use rocket::{
    delete, get,
    http::Status,
    post, put,
    response::status::{Accepted, Created, Custom},
    serde::json::Json,
    State,
};
use sqlx::SqlitePool;

use crate::{
    errors::AppError,
    models::{InsertTask, Task, UpdateTask},
};

#[post("/tasks", data = "<insert_task>")]
pub(crate) async fn insert(
    db_pool: &State<SqlitePool>,
    insert_task: InsertTask,
) -> Result<Created<Json<Task>>, AppError> {
    let task = insert_task.insert(db_pool).await?;

    Ok(Created::new("/tasks").body(Json(task)))
}

#[put("/tasks", data = "<update_task>")]
pub(crate) async fn update(
    db_pool: &State<SqlitePool>,
    update_task: UpdateTask,
) -> Result<Created<String>, AppError> {
    let rows_affected = update_task.update(db_pool).await?;

    Ok(Created::new("/tasks").body(rows_affected.to_string()))
}

#[delete("/tasks/<id>")]
pub(crate) async fn delete(
    db_pool: &State<SqlitePool>,
    id: i64,
) -> Result<Accepted<String>, AppError> {
    let rows_affected = Task::delete(db_pool, id).await?;
    Ok(Accepted(Some(rows_affected.to_string())))
}

#[post("/tasks/<id>/done")]
pub(crate) async fn done(db_pool: &State<SqlitePool>, id: i64) -> Result<Custom<String>, AppError> {
    let created_id = Task::done(db_pool, id).await?;

    if created_id == 0 {
        Ok(Custom(Status::NotModified, "".to_string()))
    } else {
        Ok(Custom(Status::Created, created_id.to_string()))
    }
}

#[post("/tasks/<id>/undo")]
pub(crate) async fn undo(db_pool: &State<SqlitePool>, id: i64) -> Result<Custom<String>, AppError> {
    let num_modified = Task::undo(db_pool, id).await?;

    if num_modified == 0 {
        Ok(Custom(Status::NotModified, "".to_string()))
    } else {
        Ok(Custom(Status::Created, num_modified.to_string()))
    }
}

#[get("/tasks")]
pub(crate) async fn find_all(db_pool: &State<SqlitePool>) -> Result<Json<Vec<Task>>, AppError> {
    let tasks = Task::find_all(db_pool).await?;

    Ok(Json(tasks))
}

#[get("/tasks/ongoing")]
pub(crate) async fn find_ongoing(db_pool: &State<SqlitePool>) -> Result<Json<Vec<Task>>, AppError> {
    let tasks = Task::find_ongoing(db_pool).await?;

    Ok(Json(tasks))
}

#[get("/tasks?<pattern>")]
pub(crate) async fn find_by_pattern(
    db_pool: &State<SqlitePool>,
    pattern: &str,
) -> Result<Json<Vec<Task>>, AppError> {
    let tasks = Task::find_by_pattern(db_pool, pattern).await?;

    Ok(Json(tasks))
}

#[get("/tasks/<id>")]
pub(crate) async fn find_by_id(
    db_pool: &State<SqlitePool>,
    id: i64,
) -> Result<Json<Option<Task>>, AppError> {
    let task = Task::find_by_id(db_pool, id).await?;

    Ok(Json(task))
}
