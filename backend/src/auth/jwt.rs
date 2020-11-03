use medallion::{Token, Header, Payload};
use chrono::{Utc, Duration};


#[derive(Serialize, Deserialize)]
pub struct JWT {
    // == private claims ==
    #[serde(rename = "profilePictureUrl")]
    profile_picture_url: Option<String>,
    #[serde(rename = "userName")]
    user_name: String,
    #[serde(rename = "userId")]
    user_id: String,
    
}



pub fn to_jwt(payload: JWT) -> Result<String, &'static str> {
    // TODO !!!! production secret

    let token: Token<(), JWT> = Token::new(Header::default(), Payload {
        iss: Some("cloud.matthiaskind.com".into()),
        exp: Some((Utc::now() + Duration::days(7)).timestamp() as u64),
        claims: Some(payload),
        aud: None,
        iat: None,
        jti: None,
        nbf: None,
        sub: None
    });

    token.sign(b"test_secret").map_err(|_| "failed to sign token")
}

pub fn validate_and_parse(token: &str) -> medallion::Result<JWT> {
    Token::<(), _>::parse(token)?.payload.claims.ok_or(
        anyhow::Error::msg("No private claims in token"))
}