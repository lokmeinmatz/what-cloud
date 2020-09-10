use crate::auth::UserID;
use rocket::http::RawStr;
use rocket::response::{Responder, NamedFile};
use std::path::{PathBuf, Path};
use log::{info, warn};
use rocket_contrib::json::Json;

pub mod metadata;
pub mod zipwriter;
pub mod shared;




/// 
#[derive(Serialize, Debug)]
pub struct NetNode {
    name: String,
    #[serde(rename = "type")]
    node_type: &'static str,
    #[serde(rename = "childrenFolder")]
    children_folder: Option<Vec<String>>,
    files: Option<Vec<String>>,
    #[serde(rename = "pathFromRoot")]
    path_from_root: Vec<String>,
    metadata: metadata::NodeMetadata
}





#[derive(Responder, Debug)]
pub enum NodeContentResponse {
    #[response(status = 400)]
    WrongDecoding(String),
    #[response(status = 404)]
    PathNotFound(String),
    #[response(status = 409)]
    DirError(String),
    #[response(status = 200)]
    NodeData(Json<NetNode>)

}


fn to_abs_data_path<P: AsRef<Path>>(user: &UserID, p: P) -> PathBuf {
    let path: &Path = p.as_ref();
    
    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user.0);
    root.push(path);
    root
}

use rocket::State;
use super::database::SharedDatabase;

pub fn url_encoded_to_rel_path(rs: &RawStr) -> Result<PathBuf, &'static str> {
    let mut raw_path: String = match rs.percent_decode() {
        Ok(s) => s.into_owned(),
        Err(_) => { return Err("Not URL-encoded"); }
    };

    if raw_path.contains("..") {
        // illegal 
        // TODO are there other symbolic links or ways to escape the dir?
        return Err(".. not allowed in path");
    };

    if raw_path.starts_with('/') { raw_path.remove(0); }

    Ok(PathBuf::from(raw_path))
}

#[get("/node?<url_encoded_path>")]
pub fn get_node_data(url_encoded_path: &RawStr, user_id: UserID, db: State<SharedDatabase>) -> NodeContentResponse {
 
    

    let folder_path = match url_encoded_to_rel_path(url_encoded_path) {
        Ok(p) => p,
        Err(e) => return NodeContentResponse::WrongDecoding(e.into())
    };
    
    
    let combined = to_abs_data_path(&user_id,&folder_path);
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
        return NodeContentResponse::PathNotFound("Path doesn't exist".into())
    }


    let mut children_folder: Option<Vec<String>> = None;
    let mut files: Option<Vec<String>> = None;

    info!("get_node_data on path {:?}", combined);
    let is_dir = combined.is_dir();
    if is_dir {
        match combined.read_dir() {
            Err(e) => {
                return NodeContentResponse::DirError(e.to_string())
            },
            Ok(dir) => {
                children_folder = Some(Vec::new());
                files = Some(Vec::new());
                let cf = children_folder.as_mut().unwrap();
                let f = files.as_mut().unwrap();
                for maybe_entry in dir {
                    if let Ok(e) = maybe_entry {
                        if let Ok(ft) = e.file_type() {
                            let fname = e.file_name().into_string().unwrap();
                            if ft.is_dir() {
                                cf.push(fname);
                            }
                            else if ft.is_file() {
                                f.push(fname);
                            }
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

    let metadata = match metadata::get_metadata(&folder_path, &user_id, db) {
        Some(md) => md,
        None => return NodeContentResponse::DirError("Metadata fetch failed".into())
    };

    NodeContentResponse::NodeData(Json(NetNode {
        name: if path_from_root.len() == 0 { "".into() } else { name },
        children_folder,
        files,
        path_from_root,
        metadata ,
        node_type: if is_dir { "folder" } else { "file" }
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

    let folder_path = match url_encoded_to_rel_path(path) {
        Ok(p) => p,
        Err(e) => return FileDownloadResponse::NotFound(())
    };

    let abs_path = to_abs_data_path(&token, &folder_path);

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