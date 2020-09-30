use rocket_contrib::json::Json;
use rocket::response::status;
use crate::database;
use sha3::Digest;
use rocket::{State, Request};
use log::{info};
use crate::database::SharedDatabase;
use crate::token_validizer::token_storage;
use rocket::request::{FromRequest, Outcome};
use serde_json::json;

#[derive(Deserialize)]
pub struct UserLogin {
    name: String,
    #[serde(rename = "passwordBase64")]
    password_base64: String
}

#[derive(Serialize)]
pub struct UserLoginResponse {
    name: String,
    #[serde(rename = "profilePictureUrl")]
    profile_picture_url: Option<String>,
    #[serde(rename = "authToken")]
    auth_token: String,
    #[serde(rename = "userId")]
    user_id: UserID
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UserID(pub String);

impl std::fmt::Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        write!(f, "UserId:{}", self.0)
     }
}

impl<'a, 'r> FromRequest<'a, 'r> for UserID {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        use rocket::http::Status;
        let token_storage = token_storage();

        if let Some(token) = request.headers().get("Authorization").next() {
            if token.starts_with("Bearer ") {
                let auth_token = &token[7..];

                if auth_token.len() == crate::auth::AUTH_TOKEN_LEN {
                    //info!("auth token req: {}", auth_token);
            
                    if let Some(ud) = token_storage.get_user_data(auth_token.as_bytes()) {
                        return Outcome::Success(ud.1.clone())
                    }
                }
            }
        }
        Outcome::Failure((Status::Unauthorized, ()))
    }
}

use rocket::http::RawStr;
use rocket::request::FromFormValue;

impl<'v> FromFormValue<'v> for UserID {
    type Error = ();

    fn from_form_value(token: &'v RawStr) -> Result<UserID, ()> {
        
        if token.len() == crate::auth::AUTH_TOKEN_LEN {
            //info!("auth token req: {}", auth_token);

            let token_storage = token_storage();
            if let Some(ud) = token_storage.get_user_data(token.as_bytes()) {
                return Ok(ud.1.clone())
            }
        }
        
        Err(())
    }
}


impl UserID {
    pub fn debug_access() -> Self {
        UserID("DEBUG_ID".into())
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
             db: State<SharedDatabase>)
    -> Result<Json<UserLoginResponse>, status::Unauthorized<&'static str>> {

        let hashed_pw = hash_pw(login_data.password_base64.as_str());
        //println!("{}", hashed_pw);
        if let Some(user) = db.get_user(database::GetUserQuery::ByName(&login_data.name)) {
            if user.hashed_pw == hashed_pw {
            info!("User login: {}", user.id);
            return Ok(Json(UserLoginResponse {
                name: std::mem::replace(&mut login_data.name, String::new()),
                profile_picture_url: None,
                auth_token: token_storage().new_user_token(user.id.clone()).iter().map(|e| *e as char).collect(),
                user_id: user.id
            }))
        }
    }

        Err(status::Unauthorized(Some("Username or password unknown")))

}

/// Sends token on success, else error
#[get("/user/logout")]
pub fn logout(user: UserID) {

    info!("User logout: {}", user);
    //println!("{}", hashed_pw);
    token_storage().remove_user(user);

}

/// Sends token on success, else error
#[get("/user", rank = 1)]
pub fn my_user(_user: UserID)
    -> Result<Json<serde_json::Value>, status::BadRequest<&'static str>> {


    Ok(Json(json!({"loggedIn": true})))
}

#[get("/user", rank = 2)]
pub fn my_user_not_loggedin()
    -> status::BadRequest<&'static str> {

    status::BadRequest(None)
}



#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    //use super::*;

    #[test]
    fn test_to_hex() {
    }


}