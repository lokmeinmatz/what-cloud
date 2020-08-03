use crate::auth::UserID;
use rocket::http::RawStr;
use rocket::response::status::NotFound;
use rocket::response::Responder;
use std::path::{PathBuf, Path};
use std::borrow::Borrow;
use log::{info, warn};
use rocket_contrib::json::Json;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct NetFolder {
    name: String,
    #[serde(rename = "childrenFolder")]
    children_folder: Vec<String>,
    files: Vec<String>,
    #[serde(rename = "pathFromRoot")]
    path_from_root: Vec<String>
}


#[derive(Responder, Debug)]
pub enum FolderContentResponse {
    #[response(status = 400)]
    WrongDecoding(String),
    #[response(status = 404)]
    PathNotFound(String),
    #[response(status = 302)]
    PathNotDir(String),
    #[response(status = 409)]
    DirError(String),
    #[response(status = 200)]
    FolderData(Json<NetFolder>)

}

#[get("/folder?<url_encoded_path>")]
pub fn get_folder_content(url_encoded_path: &RawStr, user_id: UserID) -> FolderContentResponse {
    let raw_path = url_encoded_path.percent_decode().map_err(|e| FolderContentResponse::WrongDecoding(e.to_string())).unwrap();
    let p_borrow: &str = raw_path.borrow();
    if p_borrow.borrow().contains("..") {
        return FolderContentResponse::WrongDecoding("no uplink allowed".into());
    }
    let rpath = Path::new(p_borrow);

    

    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user_id.0);
    let mut combined = root.clone();
    combined.push(rpath.strip_prefix("/").unwrap());
    if !root.exists() {
        match std::fs::create_dir(&root) {
            Ok(()) => { info!("Created base dir of user {}", user_id.0) },
            Err(e) => { warn!("Failed to create base dir of user {}: {:?}", user_id.0, e) }
        }
    }
    
    if !combined.exists() {
        // check if user has allready folder or needs to get created
        return FolderContentResponse::PathNotFound("Path doesn't exist".into())
    }
    if !combined.is_dir() {
        return FolderContentResponse::PathNotDir("Path isn't a folder".into())
    }

    let mut children_folder: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();

    info!("get_folder_content on path {:?}", combined);


    match combined.read_dir() {
        Err(e) => {
            return FolderContentResponse::DirError(e.to_string())
        },
        Ok(dir) => {
            for maybe_entry in dir {
                if let Ok(e) = maybe_entry {
                    if let Ok(ft) = e.file_type() {
                        let fname = e.file_name().into_string().unwrap();
                        if ft.is_dir() {
                            children_folder.push(fname);
                        }
                        else if ft.is_file() {
                            files.push(fname);
                        }
                    }
                }
            } 
        }
    }

    let name = combined.file_name().unwrap().to_string_lossy().into_owned();
    let mut path_from_root = Vec::new();

    for seg in raw_path.split("/") {
        if !seg.is_empty() {
            path_from_root.push(seg.into());
        }
    }

    FolderContentResponse::FolderData(Json(NetFolder {
        name,
        children_folder,
        files,
        path_from_root
    }))
}