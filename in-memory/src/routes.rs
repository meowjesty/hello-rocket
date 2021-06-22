use rocket::{
    delete, get,
    http::Status,
    post, put,
    response::status::{Created, Custom},
    serde::json::Json,
    State,
};
use std::sync::atomic::Ordering;

use crate::{
    errors::AppError,
    models::{InsertTask, Task, UpdateTask},
    AppData,
};

#[post("/tasks", data = "<insert_task>")]
pub(crate) async fn insert(
    app_data: &State<AppData>,
    insert_task: InsertTask,
) -> Result<Created<Json<Task>>, AppError> {
    if insert_task.non_empty_title.trim().is_empty() {
        Err(AppError::EmptyTitle)
    } else {
        let id = app_data.id_tracker.fetch_add(1, Ordering::Relaxed);

        let new_task = Task {
            id,
            title: insert_task.non_empty_title.to_owned(),
            details: insert_task.details.to_owned(),
        };

        let mut task_list = app_data
            .task_list
            // Try to acquire lock, convert to a 'catch-all' error on failure.
            .try_lock()
            .map_err(|_| AppError::Internal)?;

        task_list.push(new_task.clone());

        Ok(Created::new("/tasks").body(Json(new_task)))
    }
}

#[get("/tasks")]
pub(crate) async fn find_all(app_data: &State<AppData>) -> Result<Json<Vec<Task>>, AppError> {
    let task_list = app_data
        .task_list
        .try_lock()
        .map_err(|_| AppError::Internal)?
        .clone();

    Ok(Json(task_list))
}

#[get("/tasks/<id>")]
pub(crate) async fn find_by_id(app_data: &State<AppData>, id: u64) -> Result<Json<Task>, AppError> {
    let task_list = app_data
        .task_list
        .try_lock()
        .map_err(|_| AppError::Internal)?;

    let task = task_list
        .iter()
        .find(|t| t.id == id)
        .ok_or(AppError::IdNotFound(id))?
        .clone();

    Ok(Json(task))
}

#[delete("/tasks/<id>")]
pub(crate) async fn delete(
    app_data: &State<AppData>,
    id: u64,
) -> Result<Custom<Json<Task>>, AppError> {
    let mut task_list = app_data
        .task_list
        .try_lock()
        .map_err(|_| AppError::Internal)?;

    let (index, _) = task_list
        .iter()
        .enumerate()
        .find(|(_, t)| t.id == id)
        .ok_or(AppError::IdNotFound(id))?;

    let task = task_list.remove(index);

    Ok(Custom(Status::Ok, Json(task)))
}

#[put("/tasks", data = "<update_task>")]
pub(crate) async fn update(
    app_data: &State<AppData>,
    update_task: UpdateTask,
) -> Result<Created<Json<Task>>, AppError> {
    let mut task_list = app_data
        .task_list
        .try_lock()
        .map_err(|_| AppError::Internal)?;

    let mut task = task_list
        .iter_mut()
        .find(|t| t.id == update_task.id)
        .ok_or(AppError::IdNotFound(update_task.id))?;

    task.title = update_task.new_title.to_owned();
    task.details = update_task.details.to_owned();

    Ok(Created::new("/tasks").body(Json(task.clone())))
}
