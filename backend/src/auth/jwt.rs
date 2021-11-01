use chrono::{Duration, Utc};
use medallion::{Header, Payload, Token};
use rocket::{Request, request::{FromRequest, Outcome}};

#[derive(Serialize, Deserialize)]
pub struct JWT {
    // == private claims ==
    #[serde(rename = "profilePictureUrl")]
    pub profile_picture_url: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "userId")]
    pub user_id: super::UserID,
    #[serde(rename = "userRoll")]
    pub user_roll: super::database::UserRoll,
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWT {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        use rocket::http::Status;


        if let Some(token) = request.headers().get("Authorization").next() {
            if token.starts_with("Bearer ") {
                let jwt = &token[7..];

                if let Ok(jwt) = crate::auth::jwt::validate_and_parse(jwt) {
                    return Outcome::Success(jwt);
                }
                
            }
        }
        Outcome::Failure((Status::Unauthorized, ()))
    }
}

pub fn to_jwt(payload: JWT) -> Result<String, &'static str> {
    // TODO !!!! production secret

    let token: Token<(), JWT> = Token::new(
        Header::default(),
        Payload {
            iss: Some("cloud.matthiaskind.com".into()),
            exp: Some((Utc::now() + Duration::days(7)).timestamp() as u64),
            claims: Some(payload),
            aud: None,
            iat: None,
            jti: None,
            nbf: None,
            sub: None,
        },
    );

    token
        .sign(b"test_secret")
        .map_err(|_| "failed to sign token")
}

pub fn validate_and_parse(token: &str) -> medallion::Result<JWT> {
    Token::<(), _>::parse(token)?
        .payload
        .claims
        .ok_or(anyhow::Error::msg("No private claims in token"))
}
