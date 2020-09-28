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


#[get("/node?<url_encoded_path>&<shared_id>", rank = 1)]
pub fn get_node_data_shared(url_encoded_path: &RawStr, db: State<SharedDatabase>, shared_id: String) -> NodeContentResponse {
    
    // check if shared id is allowed
    if let Some(se) = db.get_shared_entry(&shared_id) {
    
        let folder_path = match url_encoded_to_rel_path(url_encoded_path) {
            Ok(p) => p,
            Err(e) => return NodeContentResponse::WrongDecoding(e.into())
        };

        // TODO why join???
        let share_path = dbg!(se.path.join(folder_path));

        get_node(&share_path, se.user, db, Some(&se.path))
    }
    else {
        NodeContentResponse::PathNotFound("Shared ID doesn't exist".into())
    }
}


#[get("/node?<url_encoded_path>", rank = 2)]
pub fn get_node_data(url_encoded_path: &RawStr, user_id: UserID, db: State<SharedDatabase>) -> NodeContentResponse {
 
    
    let folder_path = match url_encoded_to_rel_path(url_encoded_path) {
        Ok(p) => p,
        Err(e) => return NodeContentResponse::WrongDecoding(e.into())
    };
    
    get_node(&folder_path, user_id, db, None)    
}

fn get_node(folder_path: &Path, user_id: UserID, db: State<SharedDatabase>, base_path: Option<&Path>) -> NodeContentResponse {

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

    info!("get_node on path {:?}", combined);
    let is_dir = combined.is_dir();


    // collects either all components to an Vec<String> or skips the ones that are in the base_path for shared nodes
    let path_from_root = match base_path {
        None => folder_path.components(),
        Some(bp) => {
            let mut fpc = folder_path.components();

            for comp in bp.components() {
                let n_fp = fpc.next();
                if let Some(n_fp) = n_fp {
                    if n_fp.as_os_str() != comp.as_os_str() {
                        eprintln!("folder_path didn't contain base_path");
                        return NodeContentResponse::DirError("Wrong base_path".into());
                    }
                } else {
                    eprintln!("folder_path was shorter than base_path?!");
                    return NodeContentResponse::PathNotFound("Wrong base_path".into());
                }
            }
            fpc
        }
    }.map(|oss| oss.as_os_str().to_string_lossy().to_string()).collect();

    if is_dir {
        match combined.read_dir() {
            Err(e) => {
                return NodeContentResponse::DirError(e.to_string())
            },
            Ok(dir) => {
                let mut cf = Vec::new();
                let mut f = Vec::new();
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
                children_folder = Some(cf);
                files = Some(f);
            }
        }
    }

    let metadata = match metadata::get_metadata(&folder_path, &user_id, db) {
        Some(md) => md,
        None => return NodeContentResponse::DirError("Metadata fetch failed".into())
    };

    NodeContentResponse::NodeData(Json(NetNode {
        name: folder_path.file_name().map(std::ffi::OsStr::to_string_lossy).unwrap_or(std::borrow::Cow::Borrowed("")).to_string(),
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