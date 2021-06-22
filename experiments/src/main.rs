use std::{
    io,
    path::{Path, PathBuf},
    sync::atomic::AtomicU64,
};

use log::error;
use rocket::{
    fairing::{self, AdHoc},
    form::{Form, Strict},
    serde::{
        json::{serde_json, Json},
        Deserialize, Serialize,
    },
    FromForm, State,
};

use rocket::{
    data::{FromData, Outcome, ToByteUnit},
    fs::NamedFile,
    get,
    http::{CookieJar, Status},
    post,
    request::{self},
    routes,
    tokio::task::spawn_blocking,
    Build, Config, Rocket,
};

use sqlx::ConnectOptions;

#[get("/")]
async fn index() -> &'static str {
    "Hello, rocket!"
}

#[get("/blocking_task")]
async fn blocking_task() -> io::Result<Vec<u8>> {
    // In a real app, use rocket::fs::NamedFile or tokio::fs::File.
    let vec = spawn_blocking(|| std::fs::read("data.txt"))
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Interrupted, e))??;

    Ok(vec)
}

#[get("/hello/<name>")]
async fn hello(name: &str) -> String {
    format!("Hello, {}", name)
}

// NOTE(alex): Arguments must implement `FromParam`.
#[get("/hello/<name>/<age>")]
async fn hello_params(name: &str, age: u8) -> String {
    format!("Hello, {} {}", name, age)
}

// NOTE(alex): Whatever comes after `/file` will be matched into arguments that support
// `FromSegment`. Rocket has a `FileServer`, so don't use this to server static files.
#[get("/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).await.ok()
}

// NOTE(alex): This matches on `/foo/ANYTHING/bar`, `<_>` is a wildcard that ignores the parameters
// passed, that's why the function has no arguments. This matches on 3-segment requests that start
// with `/foo/` and end with `/bar`.
#[get("/foo/<_>/bar")]
async fn foo_bar() -> &'static str {
    "Foo _____ bar!"
}

// NOTE(alex): `<_..>` the `..` means ignore multiple. This matches on every get request that starts
// with `/any/`.
//
// NOTE(alex): Thanks to default route ranking, this doesn't collide with the available routes, as
// wildcard routes have lower rank.
#[get("/<_..>")]
async fn everything() -> &'static str {
    "Hey, you're here."
}

// NOTE(alex): Rocket will forward the requests and check for matches by rank, so it goes to `user`,
// then to `user_signed`, and to `user_str`, failing if none match. This requires the `rank`
// attribute, otherwise rocket will error out with route collision.
#[get("/user/<id>")]
async fn user(id: u32) -> String {
    format!("User is u32 {}", id)
}

#[get("/user/<id>", rank = 2)]
async fn user_signed(id: i32) -> String {
    format!("User is i32 {}", id)
}

#[get("/user/<id>", rank = 3)]
async fn user_str(id: &str) -> String {
    format!("User is string {}", id)
}

// NOTE(alex): `CookieJar` is a request guard, its implementation of `FromRequest` will give a
// type-level proof that we only execute the function if the argument is valid (request validation).
//
// NOTE(alex): It appears that `FromRequest` is just to validate the request by itself, not the
// data (body). To guard data, implement `FromData`.
#[get("/foo/cookies")]
async fn foo_cookies(cookies: &CookieJar<'_>) -> Option<String> {
    cookies
        .get("message")
        .map(|crumb| format!("Message: {}", crumb.value()))
}

// NOTE(alex): Requires the `secrets` feature. Debug will generate a 256-bit key by default, but
// release mode won't, so you need to generate one (e.g. `openssl rand -base64 32`).
#[get("/foo/private/cookies")]
async fn foo_private_cookies(cookies: &CookieJar<'_>) -> Option<String> {
    cookies
        .get_private("message")
        .map(|crumb| format!("Message: {}", crumb.value()))
}

#[derive(Debug, Serialize, Deserialize, FromForm)]
struct Thing {
    #[field(validate = with(|name| name.is_empty(), "Empty name"))]
    name: String,
    #[field(validate = range(0..))]
    color: u32,
}

// NOTE(alex): This is good to plug data validation.
#[rocket::async_trait]
impl<'r> FromData<'r> for Thing {
    type Error = String;

    async fn from_data(
        req: &'r rocket::Request<'_>,
        data: rocket::Data<'r>,
    ) -> rocket::data::Outcome<'r, Self> {
        let limit = req.limits().get("thing").unwrap_or(256.bytes());
        let string = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => return Outcome::Failure((Status::PayloadTooLarge, "Too large".to_string())),
            Err(fail) => {
                return Outcome::Failure((Status::InternalServerError, format!("{}", fail)))
            }
        };

        // NOTE(alex): For long lived borrows?
        let string = request::local_cache!(req, string);
        let thing: Thing = serde_json::from_str(&string).unwrap();

        if thing.name.is_empty() {
            return Outcome::Failure((Status::BadRequest, format!("Thing name is empty!")));
        }

        Outcome::Success(thing)
    }
}

#[post("/things", data = "<thing>")]
async fn new_thing(thing: Thing) -> String {
    serde_json::to_string_pretty(&thing).unwrap()
}

// NOTE(alex): Rocket uses its own version of serde and serde_json (just json here). This is similar
// to `new_thing`, but here it just does json validation.
#[post("/things", data = "<thing>")]
async fn json_thing(thing: Json<Thing>) -> String {
    let result = thing.0;
    serde_json::to_string_pretty(&result).unwrap()
}

// NOTE(alex): `FromForm` guard is more elaborate than `FromData`, it can be derived with macro,
// meanwhile manually implementing it requires checking each field/value in the form request.
// These forms may be composed together with the `FromFormField`.
// It uses relaxed rules, extra fields are ignored, missing fields will default to something.
#[post("/things/form", data = "<thing>")]
async fn form_thing(thing: Form<Thing>) {
    println!("Form {:?}", thing);
}

// NOTE(alex): Strict version of form, the default is `Lenient`. Can also be used in parts of the
// struct itself.
#[post("/things/form", data = "<thing>")]
async fn strict_thing(thing: Form<Strict<Thing>>) {
    println!("Form {:?}", thing);
}

#[get("/things?<name>&<color>")]
async fn query_thing(name: &str, color: f32) {
    println!("Query {} {}", name, color);
}

#[derive(Debug)]
struct Counter {
    count: AtomicU64,
}

#[get("/state")]
async fn get_state(counter: &State<Counter>) {
    println!("{:#?}", counter);
}

// #[database("experiments")]
// struct ExperimentsDbConn(diesel::SqliteConnection);

// #[get("/state/database")]
// async fn get_with_db(db_conn: ExperimentsDbConn) -> String {
//     db_conn.run(|c| todo!()).await;

//     format!("DONE")
// }

type Database = sqlx::SqlitePool;

// NOTE(alex): Fairings = middleware
async fn init_db(rocket: Rocket<Build>) -> fairing::Result {
    use rocket_sync_db_pools::Config;

    let config = match Config::from("sqlx", &rocket) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to read SQLx config: {}", e);
            return Err(rocket);
        }
    };

    let mut opts = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&config.url)
        .create_if_missing(true);

    opts.disable_statement_logging();
    let db = match Database::connect_with(opts).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to connect to SQLx database: {}", e);
            return Err(rocket);
        }
    };

    // if let Err(e) = sqlx::migrate!("db/sqlx/migrations").run(&db).await {
    //     error!("Failed to initialize SQLx database: {}", e);
    //     return Err(rocket);
    // }

    Ok(rocket.manage(db))
}

#[rocket::main]
async fn main() {
    let config = Config {
        port: 8080,
        ..Config::debug_default()
    };

    rocket::build()
        .configure(config)
        // globa state (app-wide)
        .manage(Counter {
            count: AtomicU64::new(0),
        })
        // .attach(ExperimentsDbConn::fairing())
        .attach(AdHoc::try_on_ignite("DB SETUP", init_db))
        .mount(
            "/",
            routes![
                index,
                blocking_task,
                hello,
                hello_params,
                files,
                foo_bar,
                everything,
                user,
                user_signed,
                user_str,
                foo_cookies,
                foo_private_cookies,
                new_thing,
                json_thing,
                form_thing,
                strict_thing,
                query_thing,
                get_state
            ],
        )
        .launch()
        .await
        .unwrap();
}
