use rocket::State;
use crate::database::SharedDatabase;
use crate::auth::UserID;
use rocket::http::RawStr;
use rocket_contrib::json::Json;

/// Set folders / files shared state
#[patch("/folder/shared?<url_encoded_path>&<enabled>")]
pub fn update_folder_share(url_encoded_path: &RawStr, enabled: bool, user_id: UserID, db: State<SharedDatabase>)
    -> Result<String, ()> {
    let raw_path: String = match url_encoded_path.percent_decode() {
        Ok(s) => s.into_owned(),
        Err(_) => { return Err(()); }
    };
    let combined = super::to_abs_data_path(&user_id, &raw_path)?;
    if !combined.exists() { return Err(()); }

    // create new share
    db.update_share(&user_id, &std::path::Path::new(&raw_path), enabled).ok_or(())
}

#[derive(Serialize)]
pub struct SharedEntry {
    pub share_id: String,
    pub path: String
}

#[get("/shared")]
pub fn get_my_shared(user_id: UserID, db: State<SharedDatabase>) -> Json<Vec<SharedEntry>> {
    Json(db.get_all_shared(&user_id))
}