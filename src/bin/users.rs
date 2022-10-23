use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::{self, catchers, launch, routes, State};

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: usize,
    username: String,
}

/// # Endpoint
/// `/users`
///
/// # Request
/// `GET /users`
///
/// # Response
/// `content-type: application/json`
///
/// Json with all the entries
///
/// ## Example
/// ``` bash
/// > curl http://127.0.0.1/users
/// > { id: 1, username: Peter, id: 2, username: Martha }
/// ```
#[rocket::get("/users", format = "json")]
async fn get_users<'r>(users: &'r State<Mutex<Vec<User>>>) -> Json<Vec<User>> {
    let usrs = users.lock().await;
    Json(usrs.to_vec())
}

/// # Endpoints
/// `/user/<id>`
///
/// # Request
/// `GET /user/<id>`
///
/// # Response
/// `content-type: application/json`
///
/// If the user with id `<id>` exists, then a json with the user id and username returned.
///
/// If the user with id `<id>` does not exists, then the json `{status: error, msg: User not found}` will be returned.
///
/// ## Example
/// ``` bash
/// > curl http://127.0.0.1:8000/user/1
/// > { id: 1, username: Peter }
/// ```
/// ``` bash
/// > curl http://127.0.0.1:8000/user/200
/// > { "status": "error", "msg": "User not found"}
/// ```
#[rocket::get("/user/<id>")]
async fn get_user<'r>(id: usize, users: &'r State<Mutex<Vec<User>>>) -> Result<Json<User>, Value> {
    let users = users.lock().await.to_vec();
    let user = users.iter().find(|user| user.id == id);
    match user {
        Some(u) => Ok(Json(u.clone())),
        None => json!({"status": "error", "msg": "User not found"}),
    }
}

/// # Endpoints
/// `/remove`
/// # Request
/// `POST /remove`
/// `data <userid>`
///
/// # Response
/// `content-type: application/json`
///
/// If the user with id `<userid>` exists, then the json `{"status": "Ok", "msg": "User removed"}` will be returned.
///
/// If the user with id `<userid>` does not exists, then the json `{"status": "error", "msg": "User with id <userid> not found"}` will be returned.
///
/// ## Example
/// ``` bash
/// > curl http://127.0.0.1:8000/remove -X POST -H 'content-type: application/json' -d '1'
/// > {"status": "Ok", "msg": "User removed"}
/// ```
/// ``` bash
/// > curl http://127.0.0.1:8000/remove -X POST -H 'content-type: application/json' -d '200'
/// > { "status": "error", "msg": "User with id 200 not found"}
/// ```
#[rocket::post("/remove", data = "<userid>")]
async fn delete_user<'r>(userid: Json<usize>, users: &'r State<Mutex<Vec<User>>>) -> Value {
    let mut users = users.lock().await;
    if let Some((index, _)) = users.iter().enumerate().find(|t| t.1.id == userid.0) {
        users.remove(index);
        return json!({"status": "Ok", "msg": "User removed"});
    }
    return json!({"status": "error", "msg": format!("User with id {} not found", userid.0)});
}

/// # Endpoints
/// `/update`
/// # Request
/// `POST /update`
/// `data <{ "id": <userid: usize>, "username": <username_to_update: string> }>`
///
/// # Response
/// `content-type: application/json`
///
/// If the user with id `<userid>` exists, then the json `{"id": <userid>, "username": <username_to_update>}` will be returned. The user will also be updated with the new username.
///
/// If the user with id `<userid>` does not exists, then the json `{"status": "error", "msg": "User  not found"}` will be returned.
///
/// ## Example
/// ``` bash
/// > curl http://127.0.0.1:8000/update -X POST -H 'content-type: application/json' -d '{"id": 1, username: "Someone Else"}'
/// > {"id": 1, "username": "Someone else"}
/// ```
/// ``` bash
/// > curl http://127.0.0.1:8000/update -X POST -H 'content-type: application/json' -d '{"id": 200, username: "Someone Else"}'
/// > { "status": "error", "msg": "User with id 200 not found"}
/// ```
#[rocket::post("/update", data = "<user>")]
async fn update_user<'r>(
    user: Json<User>,
    users: &'r State<Mutex<Vec<User>>>,
) -> Result<Json<User>, Value> {
    let mut users = users.lock().await;
    if let Some(mut usr) = users.iter_mut().find(|u| u.id == user.id) {
        usr.username = user.username.to_string();
        return Ok(user);
    }
    Err(json!({"status": "error", "msg": "No user found."}))
}

/// # Endpoints
/// `/add`
/// # Request
/// `POST /add`
/// `data <{ "id": <userid: usize>, "username": <username: string> }>`
///
/// # Response
/// `content-type: application/json`
///
/// If the user with id `<userid>` does not exists, then the json `{"msg": "user added"}` will be returned, and the user will be appended to the list of users.
///
/// If the user with id `<userid>` exists, then the json `{"status": "error", "msg": "User with id <userid> already exists"}` will be returned.
///
/// ## Example
/// ``` bash
/// > curl http://127.0.0.1:8000/add -X POST -H 'content-type: application/json' -d '{"id": 1, username: "Someone Else"}'
/// > {"status": "error", "msg": "User with id 1 already exists"}
/// ```
/// ``` bash
/// > curl http://127.0.0.1:8000/update -X POST -H 'content-type: application/json' -d '{"id": 200, username: "Someone Else"}'
/// > {"msg": "user added"}
/// ```
#[rocket::post("/add", format = "json", data = "<message>")]
async fn post_user<'r>(message: Json<User>, users: &'r State<Mutex<Vec<User>>>) -> Value {
    let mut users = users.lock().await;
    if let Some(user) = users.iter().find(|u| u.id == message.id) {
        return json!({"status": "error", "msg": format!("User with id {} already exists",user.id)});
    }
    users.push(User {
        id: message.id,
        username: message.username.clone(),
    });
    json!({"msg": "user added"})
}

#[rocket::catch(404)]
fn not_found() -> Value {
    json!({
        "error": "[404] No api endpoint reached"
    })
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
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
        .mount(
            "/",
            routes![get_users, get_user, post_user, delete_user, update_user],
        )
        .register("/", catchers![not_found])
        .manage(Mutex::new(users))
        .launch()
        .await?;
    Ok(())
}
