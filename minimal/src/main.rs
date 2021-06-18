use rocket::{get, routes};

const WELCOME: &'static str = include_str!("./../strings/welcome.txt");

#[get("/")]
async fn index() -> &'static str {
    WELCOME
}

#[rocket::main]
async fn main() {
    rocket::build()
        .mount("/", routes![index])
        .launch()
        .await
        .unwrap();
}
