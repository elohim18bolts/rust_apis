///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//  This program tries to implement basic http authentication.                                                                                                                                                                                           //
//  Every time that the api caller wants to access a secret message that belongs to a specific user, it has to send a header in he request.                                                                                                              //
//  This header has the format Authorization: Basic <credentials>.                                                                                                                                                                                       //
//  The Authorization header field is constructed as follows:                                                                                                                                                                                         //
//                                                                                                                                                                                                                                                       //
// The username and password are combined with a single colon (:). This means that the username itself cannot contain a colon.                                                                                                                           //
// The resulting string is encoded into an octet sequence. The character set to use for this encoding is by default unspecified, as long as it is compatible with US-ASCII, but the server may suggest use of UTF-8 by sending the charset parameter.[9] //
// The resulting string is encoded using a variant of Base64 (+/ and with padding).                                                                                                                                                                      //
// The authorization method and a space (e.g. "Basic ") is then prepended to the encoded string.                                                                                                                                                         //
//                                                                                                                                                                                                                                                       //
// For example, if the browser uses Aladdin as the username and open sesame as the password, then the field's value is the Base64 encoding of Aladdin:open sesame, or QWxhZGRpbjpvcGVuIHNlc2FtZQ==. Then the Authorization header field will appear as:  //
//
//Authorization: Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

mod basic_auth {
    use base64;
    pub type Username = String;
    pub type Secret = String;
    type Password = String;

    pub trait Auth {
        fn encode(&self) -> String;
        fn decode(header: &str) -> Option<User>;
    }
    #[derive(PartialEq, Debug)]
    pub struct User {
        username: Username,
        password: Password,
    }
    impl User {
        pub fn new(username: &str, password: &str) -> Self {
            Self {
                username: Username::from(username),
                password: Password::from(password),
            }
        }
        pub fn get_name(&self) -> String {
            self.username.clone()
        }
    }
    impl From<[&str; 2]> for User {
        fn from(pair: [&str; 2]) -> Self {
            Self {
                username: pair[0].to_owned(),
                password: pair[1].to_owned(),
            }
        }
    }
    impl Auth for User {
        fn encode(&self) -> String {
            let username_encoded = base64::encode(&self.username);
            let password_encoded = base64::encode(&self.password);
            format!("{username_encoded}:{password_encoded}")
        }
        fn decode(header: &str) -> Option<User> {
            //The header comes in the for Basic username_encoded:password_encoded
            let mut chunks = header.split_whitespace();
            if chunks.next() != Some("Basic") {
                return None;
            }
            match chunks.next() {
                Some(creds) => {
                    let usr_pass: Vec<&str> = creds.split(":").collect();
                    if usr_pass.len() != 2 {
                        return None;
                    }
                    let (username, password) =
                        match (base64::decode(usr_pass[0]), base64::decode(usr_pass[1])) {
                            (Ok(usr), Ok(pass)) => {
                                match (String::from_utf8(usr), String::from_utf8(pass)) {
                                    (Ok(u), Ok(p)) => (u.trim().to_string(), p.trim().to_string()),
                                    _ => return None,
                                }
                            }
                            _ => return None,
                        };
                    return Some(User::new(&username, &password));
                }
                None => return None,
            }
        }
    }
}

use basic_auth::{Auth, User};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::{self, State};

#[derive(Debug)]
struct BasicAuthHeader<'r>(&'r str);
#[rocket::async_trait]
impl<'r> FromRequest<'r> for BasicAuthHeader<'r> {
    type Error = Status;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Authorization") {
            Some(header) => {
                println!("{:?}", header);
                Outcome::Success(BasicAuthHeader(header))
            }
            None => Outcome::Failure((rocket::http::Status::BadRequest, Status::Error)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
enum Status {
    Ok,
    InvalidCredentials,
    Error,
}

#[derive(Deserialize, Serialize)]
struct Response<'r> {
    status: Status,
    msg: Option<&'r str>,
    secret: Option<basic_auth::Secret>,
}

#[rocket::get("/")]
async fn index<'r>(
    header: BasicAuthHeader<'_>,
    users: &'r State<Mutex<Vec<User>>>,
) -> Json<Response<'r>> {
    let users = users.lock().await;
    if let Some(usr) = User::decode(&header.0) {
        let user = users.iter().find(|u| u == &&usr);
        match user {
            Some(user) => {
                println!("User: {:?}", user);
                let resp = Response {
                    status: Status::Ok,
                    msg: Some("Amazing"),
                    secret: Some(format!("This is a secret from {}", user.get_name())),
                };
                return Json(resp);
            }
            None => {
                let resp = Response {
                    status: Status::InvalidCredentials,
                    msg: None,
                    secret: None,
                };
                return Json(resp);
            }
        }
    }
    let resp = Response {
        status: Status::Error,
        msg: Some("Invalid or missing authorization header"),
        secret: None,
    };
    Json(resp)
}

#[rocket::catch(400)]
async fn bad_req<'r>() -> Json<Response<'r>> {
    Json(Response {
        status: Status::Error,
        msg: Some("Bad Request. Missing authorization header"),
        secret: None,
    })
}
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let users = vec![
        User::new("Peter", "1234"),
        User::new("John", "password"),
        User::new("Martha", "This is an amazing password"),
        User::new("Maria", "Pass:user:Home"),
        User::new("Anna", "p@bl0"),
    ];
    for user in &users {
        println!("{}", user.encode());
    }
    let user = User::new("Hero", "password");
    let _ = rocket::build()
        .mount("/", rocket::routes![index])
        .manage(Mutex::new(users))
        .register("/", rocket::catchers![bad_req])
        .launch()
        .await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::basic_auth::{Auth, User};
    #[test]
    fn test_header_encode() {
        let username = "SampleUser";
        let password = "password";
        let user = User::new(&username, &password);
        assert_eq!(
            format!("{}:{}", base64::encode(username), base64::encode(password)),
            user.encode(),
            "The encoded header should be {}:{}",
            base64::encode(username),
            base64::encode(password)
        );
        dbg!(user.encode());
    }

    #[test]
    fn test_header_decode() {
        let header = "Basic U2FtcGxlVXNlcg==:cGFzc3dvcmQ=";
        assert_eq!(
            Some(User::new("SampleUser", "password")),
            User::decode(header)
        );
    }
}
