use std::path::Path;
use std::borrow::Borrow;
use crate::fs::NetFilePath;
use crate::auth::UserID;
use crate::database::SharedDatabase;
use log::info;
use rocket::http::RawStr;
use rocket::State;
use rocket_contrib::json::Json;

#[derive(Serialize)]
pub struct SharedID(String);

impl SharedID {
    pub fn from_string_unchecked(id: String) -> Self {
        Self(id)
    }

    pub fn from_string_checked(id: String, db: &mut SharedDatabase) -> Option<Self> {
        if db.is_active_shared_id(&id) {
            Some(Self(id))
        }
        else {
            None
        }
    }
}


/// Set folders / files shared state
#[patch("/folder/shared?<path>&<enabled>")]
pub fn update_folder_share(
    path: NetFilePath,
    enabled: bool,
    user_id: UserID,
    db: State<SharedDatabase>,
) -> Result<String, ()> {
    let combined = super::to_abs_data_path(&user_id, Borrow::<Path>::borrow(&path));
    if !combined.exists() {
        return Err(());
    }
    // create new share
    let r = db.update_share(&user_id, Borrow::<Path>::borrow(&path), enabled);
    info!(
        "User {} set shared of {:?} to {:?}",
        &user_id, &path, &r
    );
    r.ok_or(())
}

use std::path::PathBuf;

#[derive(Serialize)]
pub struct SharedEntry {
    pub share_id: SharedID,
    pub user: UserID,
    pub path: PathBuf,
}

#[get("/shared")]
pub fn get_my_shared(user_id: UserID, db: State<SharedDatabase>) -> Json<Vec<SharedEntry>> {
    Json(db.get_all_shared(&user_id))
}
