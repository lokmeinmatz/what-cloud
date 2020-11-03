use chrono::{Duration, Utc};
use medallion::{Header, Payload, Token};

#[derive(Serialize, Deserialize)]
pub struct JWT {
    // == private claims ==
    #[serde(rename = "profilePictureUrl")]
    pub profile_picture_url: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "userId")]
    pub user_id: super::UserID,
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
