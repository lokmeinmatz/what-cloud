use rocket::Route;
use rocket::State;
use crate::token_validizer::ActiveTokenStorage;
use rocket_contrib::json::Json;

pub fn mount_admin() -> Vec<Route> {
    routes![get_active_sessions]
}


#[derive(Serialize)]
struct Session {
    token: String,
    last_conn: std::time::SystemTime,
    user_id: String
}

#[get("/admin/active_sessions")]
fn get_active_sessions(tokens: State<ActiveTokenStorage>) -> Json<Vec<Session>> {
    let mut res = Vec::new();

    for (token, data) in tokens.inner().inner().iter() {
        res.push(Session {
            token: String::from_utf8_lossy(token).to_string(),
            last_conn: data.0,
            user_id: (data.1).0.clone()
        })
    }


    Json(res)
}