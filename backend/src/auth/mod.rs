use crate::database;
use crate::database::SharedDatabase;
use log::info;
use rocket::request::{FromRequest, Outcome};
use rocket::response::status;
use rocket::{Request, State};
use rocket_contrib::json::Json;
use serde_json::json;
use sha3::Digest;

mod jwt;

#[derive(Deserialize)]
pub struct UserLogin {
    name: String,
    #[serde(rename = "passwordBase64")]
    password_base64: String,
}

/// Length == 8 !!!
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserID(pub String);

impl std::fmt::Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "UserId:{}", self.0)
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for UserID {
    type Error = ();
    async fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        use rocket::http::Status;


        if let Some(token) = request.headers().get("Authorization").next() {
            if token.starts_with("Bearer ") {
                let jwt = &token[7..];

                if let Ok(jwt) = crate::auth::jwt::validate_and_parse(jwt) {
                    return Outcome::Success(jwt.user_id);
                }
                
            }
        }
        Outcome::Failure((Status::Unauthorized, ()))
    }
}

use rocket::http::RawStr;
use rocket::request::FromFormValue;

use self::jwt::JWT;

impl<'v> FromFormValue<'v> for UserID {
    type Error = ();

    fn from_form_value(token: &'v RawStr) -> Result<UserID, ()> {
        crate::auth::jwt::validate_and_parse(token).map(|jwt| jwt.user_id).map_err(drop)
    }
}

impl UserID {
    pub fn debug_access() -> Self {
        UserID("DEBUG_ID".into())
    }
}

pub const AUTH_TOKEN_LEN: usize = 16;

#[inline]
fn quad_to_char(b: u8) -> char {
    if b < 10 {
        return (b + 0x30) as char;
    }
    (b + 0x57) as char
}

pub fn hash_str_to_hex(strng: &str) -> String {
    let mut hasher = sha3::Sha3_256::new();
    hasher.update(strng.as_bytes());
    let mut res = String::with_capacity(64);

    for e in hasher.finalize().iter() {
        res.push(quad_to_char(*e >> 4));
        res.push(quad_to_char(*e & 0x0f));
    }

    res
}

/// Sends token on success, else error
#[post("/user/login", data = "<login_data>")]
pub fn login(
    mut login_data: Json<UserLogin>,
    db: State<SharedDatabase>,
) -> Result<String, status::Unauthorized<&'static str>> {
    let hashed_pw = hash_str_to_hex(login_data.password_base64.as_str());
    //println!("{}", hashed_pw);
    if let Some(user) = db.get_user(database::GetUserQuery::ByName(&login_data.name)) {
        if user.hashed_pw == hashed_pw {
            info!("User login: {}", user.id);
            let jwt = jwt::to_jwt(JWT {
                profile_picture_url: None,
                user_id: user.id,
                user_name: std::mem::replace(&mut login_data.name, String::new()),
            })
            .map_err(|s| status::Unauthorized(Some(s)))?;

            return Ok(jwt);
        }
    }

    Err(status::Unauthorized(Some("Username or password unknown")))
}

/// Sends token on success, else error
#[get("/user/logout")]
pub fn logout(user: UserID) {
    info!("User logout: {}", user);
    //println!("{}", hashed_pw);
    warn!("Because we are using JWT, logout is currently not implemented on server side");
}

/// Sends token on success, else error
#[get("/user", rank = 1)]
pub fn my_user(_user: UserID) -> Result<Json<serde_json::Value>, status::BadRequest<&'static str>> {
    Ok(Json(json!({"loggedIn": true})))
}

#[get("/user", rank = 2)]
pub fn my_user_not_loggedin() -> status::BadRequest<&'static str> {
    status::BadRequest(None)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    //use super::*;

    #[test]
    fn test_to_hex() {}
}
