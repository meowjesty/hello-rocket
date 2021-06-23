use std::sync::{atomic::AtomicU64, Mutex};

use errors::AppError;
use models::Task;
use rocket::{
    get, routes,
    serde::{Deserialize, Serialize},
};
use routes::{
    delete, done, find_all, find_by_id, find_by_pattern, find_ongoing, insert, undo, update,
};
use sqlx::SqlitePool;

mod errors;
mod models;
mod routes;

const WELCOME: &'static str = include_str!("./../strings/welcome.txt");
const CREATE_DATABASE: &'static str = include_str!("./../queries/create_database.sql");

#[derive(Serialize, Deserialize)]
struct AppData {
    id_tracker: AtomicU64,
    task_list: Mutex<Vec<Task>>,
}

#[get("/")]
async fn index() -> &'static str {
    WELCOME
}

/// NOTE(alex): This function should be part of some setup script, it's here for convenience. It
/// could be moved to the `build.rs`, by adding `sqlx` and `tokio` as `dev-dependencies`:
async fn create_database(db_pool: &SqlitePool) -> Result<u64, AppError> {
    let mut connection = db_pool.acquire().await?;

    let result = sqlx::query(CREATE_DATABASE)
        .execute(&mut connection)
        .await?;

    Ok(result.rows_affected())
}

#[rocket::main]
async fn main() {
    let db_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(env!("DATABASE_FILE"))
        .create_if_missing(true);

    let db_pool = SqlitePool::connect_with(db_options.clone()).await.unwrap();

    if let Some(_) = option_env!("CREATE_DATABASE") {
        create_database(&&db_pool).await.unwrap();
    }

    rocket::build()
        .manage(db_pool)
        .mount(
            "/",
            routes![
                index,
                insert,
                update,
                delete,
                done,
                undo,
                find_all,
                find_ongoing,
                find_by_pattern,
                find_by_id
            ],
        )
        .launch()
        .await
        .unwrap();
}
