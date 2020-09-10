use rocket::State;
use crate::database::SharedDatabase;
use crate::auth::UserID;
use rocket::http::RawStr;
use rocket_contrib::json::Json;
use log::info;


/// Set folders / files shared state
#[patch("/folder/shared?<url_encoded_path>&<enabled>")]
pub fn update_folder_share(url_encoded_path: &RawStr, enabled: bool, user_id: UserID, db: State<SharedDatabase>)
    -> Result<String, ()> {
    let folder_path = super::url_encoded_to_rel_path(url_encoded_path).map_err(drop)?;
    let combined = super::to_abs_data_path(&user_id, &folder_path);
    if !combined.exists() { return Err(()); }
    // create new share
    let r = db.update_share(&user_id, &folder_path, enabled);
    info!("User {} set shared of {:?} to {:?}", &user_id, &folder_path, &r);
    r.ok_or(())
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