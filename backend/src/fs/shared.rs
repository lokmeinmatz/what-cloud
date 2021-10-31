use crate::auth::UserID;
use crate::database::SharedDatabase;
use crate::fs::NetFilePath;
use log::info;
use rocket::State;
use rocket::serde::json::Json;
use std::borrow::Borrow;
use std::path::Path;

#[derive(Serialize, Debug)]
pub struct SharedID(String);

impl SharedID {
    pub fn from_string_unchecked(id: String) -> Self {
        Self(id)
    }

    #[allow(dead_code)]
    pub fn from_string_checked(id: String, db: &SharedDatabase) -> Option<Self> {
        if db.is_active_shared_id(&id) {
            Some(Self(id))
        } else {
            None
        }
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for SharedID {
    fn respond_to(
        self,
        req: &'r rocket::Request<'_>,
    ) -> Result<rocket::Response<'static>, rocket::http::Status> {
        self.0.respond_to(req)
    }
}

impl std::convert::AsRef<str> for SharedID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Set folders / files shared state
/// upload_limit in mb
#[patch("/folder/shared?<path>&<enabled>&<upload_limit>")]
pub fn update_folder_share(
    path: NetFilePath,
    enabled: bool,
    upload_limit: Option<u32>,
    user_id: UserID,
    db: &State<SharedDatabase>,
) -> Result<SharedID, ()> {
    let combined: PathBuf = super::to_abs_data_path(&user_id, Borrow::<Path>::borrow(&path));
    if !combined.exists() || combined.is_file() {
        return Err(());
    }
    // create new share
    let r = db.update_share(
        &user_id,
        Borrow::<Path>::borrow(&path),
        enabled,
        upload_limit,
    );
    info!("User {} set shared of {:?} to {:?}", &user_id, &path, &r);
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
pub fn get_my_shared(user_id: UserID, db: &State<SharedDatabase>) -> Json<Vec<SharedEntry>> {
    Json(db.get_all_shared(&user_id))
}
