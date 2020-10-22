use crate::auth::UserID;
use log::{info, warn};
use path_slash::PathExt;
use rocket::request::FromFormValue;
use rocket::response::{Responder};
use rocket_contrib::json::Json;
use std::borrow::Borrow;
use std::path::{Path, PathBuf};

pub mod metadata;
pub mod shared;
pub mod download;
pub mod zipwriter;
pub mod previews;
pub mod upload;
pub mod download;
pub mod netfilepath;
mod blocking_buf;
mod async_buf;

use netfilepath::NetFilePath;

///
#[derive(Serialize, Debug)]
pub struct NetNode {
    name: String,
    #[serde(rename = "childrenFolder")]
    children_folder: Option<Vec<String>>,
    files: Option<Vec<String>>,
    #[serde(rename = "pathFromRoot")]
    path_from_root: Vec<String>,
    metadata: metadata::NodeMetadata,
    #[serde(rename = "ownedBy")]
    owned_by: UserID,
}


#[derive(Responder, Debug)]
pub enum NodeContentResponse {
    #[response(status = 404)]
    PathNotFound(String),
    #[response(status = 409)]
    DirError(String),
    #[response(status = 200)]
    NodeData(Json<NetNode>),
}

fn to_abs_data_path<P: AsRef<Path>>(user: &UserID, p: P) -> PathBuf {
    let path: &Path = p.as_ref();
    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user.0);
    root.push(path);
    root
}

use super::database::SharedDatabase;
use rocket::State;

#[get("/node?<file_path>&<shared_id>", rank = 1)]
pub fn get_node_data_shared(
    mut file_path: NetFilePath,
    db: State<SharedDatabase>,
    shared_id: String,
) -> NodeContentResponse {
    // check if shared id is allowed
    if let Some(se) = db.get_shared_entry(&shared_id) {
        
        file_path.add_prefix(&se.path);
        
        get_node(file_path, se.user, db, Some(&se.path))
    } else {
        NodeContentResponse::PathNotFound("Shared ID doesn't exist".into())
    }
}

#[get("/node?<file_path>", rank = 2)]
pub fn get_node_data(
    file_path: NetFilePath,
    user_id: UserID,
    db: State<SharedDatabase>,
) -> NodeContentResponse {
    get_node(file_path, user_id, db, None)
}

/// folder_path: Path from base folder of user, but WITHOUT user_id prefix!!!
fn get_node(
    folder_path: NetFilePath,
    user_id: UserID,
    db: State<SharedDatabase>,
    base_path: Option<&Path>,
) -> NodeContentResponse {
    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user_id.0);
    if !root.exists() {
        match std::fs::create_dir(&root) {
            Ok(()) => info!("Created base dir of user {}", user_id.0),
            Err(e) => warn!("Failed to create base dir of user {}: {:?}", user_id.0, e),
        }
    }
    root.push(Borrow::<str>::borrow(&folder_path));
    let combined = root;
    if !combined.exists() {
        // check if user has allready folder or needs to get created
        return NodeContentResponse::PathNotFound("Path doesn't exist".into());
    }

    let mut children_folder: Option<Vec<String>> = None;
    let mut files: Option<Vec<String>> = None;

    let is_dir = combined.is_dir();

    // collects either all components to an Vec<String> or skips the ones that are in the base_path for shared nodes
    let folder_path = Borrow::<Path>::borrow(&folder_path);

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
    }
    .map(|oss| oss.as_os_str().to_string_lossy().to_string())
    .collect();

    if is_dir {
        match combined.read_dir() {
            Err(e) => return NodeContentResponse::DirError(e.to_string()),
            Ok(dir) => {
                let mut cf = Vec::new();
                let mut f = Vec::new();
                for maybe_entry in dir {
                    if let Ok(e) = maybe_entry {
                        if let Ok(ft) = e.file_type() {
                            let fname = e.file_name().into_string().unwrap();
                            if ft.is_dir() {
                                cf.push(fname);
                            } else if ft.is_file() {
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
        None => return NodeContentResponse::DirError("Metadata fetch failed".into()),
    };

    NodeContentResponse::NodeData(Json(NetNode {
        name: folder_path
            .file_name()
            .map(std::ffi::OsStr::to_string_lossy)
            .unwrap_or(std::borrow::Cow::Borrowed(""))
            .to_string(),
        children_folder,
        files,
        path_from_root,
        metadata,
        owned_by: user_id,
    }))
}


use rocket::response::status;
#[delete("/node?<path>")]
pub async fn delete_node_data(
    path: NetFilePath,
    user_id: UserID,
    addr: std::net::SocketAddr
) -> Result<status::Accepted<()>, status::Forbidden<()>> {
    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user_id.0);
    root.push(Borrow::<str>::borrow(&path));
    
    if !root.exists() {
        warn!("User tried to delete {:?} which doesn't exist", &root);
        return Err(status::Forbidden(None));
    }

    info!("IP {:?} deletes {:?}", addr, &path);
    if root.is_file() {
        // delete file
        if let Err(_) = std::fs::remove_file(&root) {
            return Err(status::Forbidden(None));
        }
    } else {
        // delete folder and all its children
        if let Err(_) = std::fs::remove_dir_all(&root) {
            return Err(status::Forbidden(None));
        }
    }
    Ok(status::Accepted(None))
}