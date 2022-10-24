///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//  This program tries to implement basic http authentication.                                                                                                                                                                                           //
//  Every time that the api caller wants to access a secret message that belongs to a specific user, it has to send a header in he request.                                                                                                              //
//  This header has the format Authorization: Basic <credentials>.                                                                                                                                                                                       //
//  The Authorization header field is constructed as follows:[9]                                                                                                                                                                                         //
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

use rocket;
use rocket::serde::{json::Json, Deserialize, Serialize};

type Username = String;
type Secret = String;

#[derive(Deserialize, Serialize)]
enum Status {
    Ok,
    InvalidCredentials,
    Error,
}

#[derive(Deserialize, Serialize)]
struct Response {
    status: Status,
    msg: Option<String>,
    secret: Option<Secret>,
}

#[rocket::get("/")]
async fn index() -> Json<Response> {
    let resp = Response {
        status: Status::Ok,
        msg: None,
        secret: Some(Secret::from("This is a secret")),
    };
    Json(resp)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _ = rocket::build()
        .mount("/", rocket::routes![index])
        .launch()
        .await?;
    Ok(())
}
