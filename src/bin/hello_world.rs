use rocket::serde::json::{json, Value};
use rocket::{self, get, launch, routes};

/// # Endpoints
/// `http://localhost:8000/hello`
///
/// # Response
/// ## Header
/// `content-type: text/html`
/// ## Content
/// `"Hello World"`
///
/// ## Returns
/// A string slice with `'static` lifetime
#[get("/hello")]
fn hello() -> &'static str {
    "Hello World"
}

/// # Endpoints
/// `http://localhost:8000/hello_json`
///
/// # Response
/// ## Header
/// `content-type: application/json`
/// ## Content
///  `{"msg": "hello world"}`
#[get("/hello_json", format = "json")]
fn hello_json() -> Value {
    json!({ "msg":"hello world" })
}

/// Launch the server
#[launch]
fn root() -> _ {
    rocket::build()
        .mount("/", routes![hello])
        .mount("/", routes![hello_json])
}
