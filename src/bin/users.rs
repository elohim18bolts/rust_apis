use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::{self, catchers, launch, routes, State};

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: usize,
    username: String,
}

#[rocket::get("/users", format = "json")]
async fn get_users<'r>(users: &'r State<Mutex<Vec<User>>>) -> Json<Vec<User>> {
    let usrs = users.lock().await;
    Json(usrs.to_vec())
}

#[rocket::get("/user/<id>")]
async fn get_user<'r>(id: usize, users: &'r State<Mutex<Vec<User>>>) -> Option<Json<User>> {
    let users = users.lock().await.to_vec();
    let user = users.iter().find(|user| user.id == id);
    match user {
        Some(u) => Some(Json(u.clone())),
        None => None,
    }
}

#[rocket::get("/add/user/<id>/<username>")]
async fn add_user<'r>(id: usize, username: &str, users: &'r State<Mutex<Vec<User>>>) -> Value {
    let mut usrs = users.lock().await;
    usrs.push(User {
        id,
        username: String::from(username),
    });
    return json!("User added");
}

#[rocket::post("/add", format = "json", data = "<message>")]
async fn post_user<'r>(message: Json<User>, users: &'r State<Mutex<Vec<User>>>) -> Value {
    let mut users = users.lock().await;
    users.push(User {
        id: message.id,
        username: message.username.clone(),
    });
    json!({"msg": "user added"})
}

#[rocket::catch(404)]
fn not_found() -> Value {
    json!({
        "error": "No user found"
    })
}

#[launch]
fn root() -> _ {
    let mut users = vec![
        User {
            id: 1,
            username: "John".to_owned(),
        },
        User {
            id: 2,
            username: "Anna".to_owned(),
        },
        User {
            id: 3,
            username: "Martha".to_owned(),
        },
    ];
    rocket::build()
        .mount("/", routes![get_users, get_user, add_user, post_user])
        .register("/", catchers![not_found])
        .manage(Mutex::new(users))
}
