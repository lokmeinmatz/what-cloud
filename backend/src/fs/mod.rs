use crate::auth::UserID;
use rocket::http::RawStr;
use rocket::response::{Responder, NamedFile};
use std::path::{PathBuf, Path};
use std::borrow::Borrow;
use log::{info, warn};
use rocket_contrib::json::Json;

pub mod metadata;
pub mod zipwriter;



#[derive(Serialize, Debug)]
pub struct NetFolder {
    name: String,
    #[serde(rename = "childrenFolder")]
    children_folder: Vec<String>,
    files: Vec<String>,
    #[serde(rename = "pathFromRoot")]
    path_from_root: Vec<String>,
    shared: Option<String>
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


fn to_abs_data_path(user: &UserID, path: &str) -> Result<PathBuf, ()> {

    if path.contains("..") {
        return Err(());
    }
    let rpath = Path::new(path);

    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user.0);
    root.push(rpath.strip_prefix("/").map_err(|_| ())?);
    Ok(root)
}

use rocket::State;
use super::database::SharedDatabase;

#[patch("/folder/shared?<url_encoded_path>&<enabled>")]
pub fn update_folder_share(url_encoded_path: &RawStr, enabled: bool, user_id: UserID, db: State<SharedDatabase>)
    -> Option<String> {
    let raw_path: String = match url_encoded_path.percent_decode() {
        Ok(s) => s.into_owned(),
        Err(e) => { return None; }
    };
    let combined = to_abs_data_path(&user_id, &raw_path).ok()?;
    if !combined.exists() { return None; }

    // create new share
    db.update_share(&user_id, &std::path::Path::new(&raw_path), enabled)
}

#[get("/folder?<url_encoded_path>")]
pub fn get_folder_content(url_encoded_path: &RawStr, user_id: UserID, db: State<SharedDatabase>) -> FolderContentResponse {
 
    
    let raw_path: String = match url_encoded_path.percent_decode() {
        Ok(s) => s.into_owned(),
        Err(e) => { return FolderContentResponse::WrongDecoding(e.to_string()); }
    };
    let shared: Option<String> = db.get_share_id(&user_id, &std::path::Path::new(&raw_path))
        .ok().flatten();
    
    let combined = match to_abs_data_path(&user_id,&raw_path) {
        Ok(c) => c,
        Err(()) => return FolderContentResponse::WrongDecoding("Error in path".into())
    };
    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user_id.0);
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

    for seg in url_encoded_path.split("%2F") {
        if !seg.is_empty() {
            path_from_root.push(seg.into());
        }
    }

    FolderContentResponse::FolderData(Json(NetFolder {
        name: if path_from_root.len() == 0 { "".into() } else { name },
        children_folder,
        files,
        path_from_root,
        shared
    }))
}


use rocket::response::Stream;
#[derive(Responder)]
pub enum FileDownloadResponse {
    #[response(status = 200)]
    File(NamedFile),
    #[response(status = 200)]
    Zip(Stream<zipwriter::BlockingConsumer>),
    #[response(status = 401)]
    Unauthorized(()),
    #[response(status = 404)]
    NotFound(())
}


#[get("/download/file?<path>&<token>")]
pub fn download_file(path: &RawStr, token: UserID) -> FileDownloadResponse {
    info!("User {:?} requested download of {}", token, path);

    let path = match path.percent_decode() {
        Ok(s) => s.into_owned(),
        Err(e) => { return FileDownloadResponse::NotFound(()); }
    };

    let abs_path = match to_abs_data_path(&token, &path) {
        Ok(p) => p,
        Err(()) => {
            println!("to_abs_data_path failed");
            return FileDownloadResponse::NotFound(())}
    };

    if abs_path.is_dir() {
        // handle zip file
        let cons = zipwriter::new_zip_writer(abs_path).unwrap();
        FileDownloadResponse::Zip(Stream::chunked(cons, 4096))
    }
    else {
        match NamedFile::open(&abs_path) {
            Ok(nf) => FileDownloadResponse::File(nf),
            Err(e) => {
                warn!("Error while reading file {:?} : {:?}", abs_path, e);
                FileDownloadResponse::NotFound(())
            }
        }
    }

}