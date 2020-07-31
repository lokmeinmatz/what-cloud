use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use rocket::response::status;
use crate::database;
use rand::Rng;
use sha3::Digest;
use rocket::{State, Request};
use log::{info, trace};
use crate::database::SharedDatabase;
use crate::token_validizer::ActiveTokenStorage;
use rocket::request::{FromRequest, Outcome};

#[derive(Deserialize)]
pub struct UserLogin {
    name: String,
    password_base64: String
}

#[derive(Serialize)]
pub struct UserLoginResponse {
    name: String,
    profile_picture_url: Option<String>,
    auth_token: String
}

#[derive(Debug, Clone)]
pub struct UserID(pub String);

impl<'a, 'r> FromRequest<'a, 'r> for UserID {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        use rocket::http::Status;
        let token_storage = request.guard::<State<ActiveTokenStorage>>().unwrap();

        if let Some(token) = request.headers().get("Authorization").next() {
            if token.starts_with("Basic ") {
                let auth_token = &token[6..];

                if auth_token.len() == crate::auth::AUTH_TOKEN_LEN {
                    trace!("auth token req: {}", auth_token);
                    if let Some(ud) = token_storage.get_user_data(auth_token.as_bytes()) {
                        return Outcome::Success(ud.1.clone())
                    }
                }
            }
        }
        Outcome::Failure((Status::Unauthorized, ()))
    }
}


pub const AUTH_TOKEN_LEN : usize = 16;

#[inline]
fn quad_to_char(b: u8) -> char {
    if b < 10 {
        return (b + 0x30) as char
    }
    (b + 0x57) as char
}

fn hash_pw(password: &str) -> String {
    let mut hasher = sha3::Sha3_256::new();
    hasher.update(password.as_bytes());
    let mut res = String::with_capacity(64);
    for e in hasher.finalize().iter() {
        res.push(quad_to_char(*e >> 4));
        res.push(quad_to_char(*e & 0x0f));
    }

    res
}


/// Sends token on success, else error
#[post("/user/login", data = "<login_data>")]
pub fn login(mut login_data: Json<UserLogin>,
             db: State<SharedDatabase>,
             tokens: State<ActiveTokenStorage>)
    -> Result<Json<UserLoginResponse>, status::Unauthorized<&'static str>> {

    info!("User login: {}", login_data.name);
    let hashed_pw = hash_pw(login_data.password_base64.as_str());
    println!("{}", hashed_pw);
    if let Some(user) = db.get_user(database::GetUserQuery::ByName(&login_data.name)) {
        if user.hashed_pw == hashed_pw {
            return Ok(Json(UserLoginResponse {
                name: std::mem::replace(&mut login_data.name, String::new()),
                profile_picture_url: None,
                auth_token: tokens.new_user_token(user.id).iter().map(|e| *e as char).collect()
            }))
        }
    }

        Err(status::Unauthorized(Some("Username or password unknown")))

}



#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_to_hex() {
        assert_eq!(add(1, 2), 3);
    }


}