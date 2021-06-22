use rocket::{get, launch, routes};

const WELCOME: &'static str = include_str!("./../strings/welcome.txt");

#[get("/")]
async fn index() -> &'static str {
    WELCOME
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
