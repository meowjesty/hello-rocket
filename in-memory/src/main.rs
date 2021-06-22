use std::sync::{atomic::AtomicU64, Mutex};

use models::Task;
use rocket::{
    get, launch, routes,
    serde::{Deserialize, Serialize},
};
use routes::{delete, find_all, find_by_id, insert, update};

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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(AppData {
            id_tracker: AtomicU64::new(0),
            task_list: Mutex::new(Vec::with_capacity(32)),
        })
        .mount(
            "/",
            routes![index, insert, find_all, find_by_id, delete, update],
        )
}
