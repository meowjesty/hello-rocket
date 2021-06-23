use std::sync::{atomic::AtomicU64, Mutex};

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

#[derive(Serialize, Deserialize)]
struct AppData {
    id_tracker: AtomicU64,
    task_list: Mutex<Vec<Task>>,
}

#[get("/")]
async fn index() -> &'static str {
    WELCOME
}

#[rocket::main]
async fn main() {
    let db_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(env!("DATABASE_URL"))
        .create_if_missing(true);
    let db_pool = SqlitePool::connect_with(db_options).await.unwrap();

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
