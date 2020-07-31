use crate::auth::UserID;
use rocket::http::RawStr;
use std::path::{PathBuf, Path};
use std::borrow::Borrow;
use log::info;
use rocket_contrib::json::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct NetFolder {
    name: String,
    #[serde(rename = "childrenFolder")]
    children_folder: Vec<String>,
    files: Vec<String>,
    #[serde(rename = "pathFromRoot")]
    path_from_root: Vec<String>
}

#[get("/folder?<url_encoded_path>")]
pub fn get_folder_content(url_encoded_path: &RawStr, user_id: UserID) -> Result<Json<NetFolder>, ()> {
    let raw_path = url_encoded_path.percent_decode().map_err(|_| ())?;
    let p_borrow: &str = raw_path.borrow();
    if p_borrow.borrow().contains("..") {
        return Err(());
    }
    let rpath = Path::new(p_borrow);

    let mut combined: PathBuf = PathBuf::from(crate::config::data_base_path());
    combined.push(user_id.0);
    combined.push(rpath);
    info!("get_folder_content on path {:?}", combined);
    Err(())
}