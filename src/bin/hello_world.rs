use rocket::serde::json::{json, Value};
use rocket::{self, get, launch, routes};
#[get("/hello")]
fn hello() -> &'static str {
    "Hello World"
}

#[get("/hello_json", format = "json")]
fn hello_json() -> Value {
    json!({ "msg":"hello world" })
}

#[launch]
fn root() -> _ {
    rocket::build()
        .mount("/", routes![hello])
        .mount("/", routes![hello_json])
}
