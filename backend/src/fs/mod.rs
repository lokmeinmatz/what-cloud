use rocket::http::RawStr;
use crate::auth::UserID;
use log::{info, warn};
use path_slash::PathExt;
use rocket::request::FromFormValue;
use rocket::response::{NamedFile, Responder};
use rocket_contrib::json::Json;
use std::borrow::Borrow;
use std::path::{Path, PathBuf};

pub mod metadata;
pub mod shared;
pub mod zipwriter;
pub mod upload;
mod blocking_buf;
mod async_buf;

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

#[derive(Debug)]
pub struct NetFilePath(String);

impl NetFilePath {
    #[allow(dead_code)]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self(path.as_ref().to_slash_lossy())
    }

    pub fn add_prefix<P: AsRef<Path>>(&mut self, prefix: P) {
        let mut n_base: String = prefix.as_ref().to_slash_lossy();
        if n_base.ends_with('/') && self.0.starts_with('/') {
            n_base.push_str(&self.0[1..]);
        } else if n_base.ends_with('/') || self.0.starts_with('/') {
            n_base.push_str(&self.0);
        } else if self.0.len() > 0 {
            n_base.push('/');
            n_base.push_str(&self.0);
        }
        self.0 = n_base;
    }
}

impl Borrow<str> for NetFilePath {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Borrow<Path> for NetFilePath {
    fn borrow(&self) -> &Path {
        &Path::new(&self.0)
    }
}

impl<'v> FromFormValue<'v> for NetFilePath {
    type Error = ();
    fn from_form_value(raw: &'v rocket::http::RawStr) -> Result<Self, Self::Error> {
        dbg!(raw);
        let mut raw_path: String = match raw.percent_decode() {
            Ok(s) => s.into_owned(),
            Err(_) => {
                return Err(());
            }
        };

        if raw_path.contains("..") {
            // illegal
            // TODO are there other symbolic links or ways to escape the dir?
            return Err(());
        };
        if raw_path.starts_with('/') {
            raw_path.remove(0);
        }
        Ok(NetFilePath(Path::new(&raw_path).to_slash_lossy()))
    }
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

use rocket::response::Stream;
#[derive(Responder)]
pub enum FileDownloadResponse {
    #[response(status = 200)]
    File(NamedFile),
    #[response(status = 200)]
    Zip(Stream<async_buf::AsyncConsumer>),
    #[response(status = 401)]
    Unauthorized(()),
    #[response(status = 404)]
    NotFound(()),
}

#[get("/download/file?<path>&<token>", rank = 1)]
pub async fn download_file(path: NetFilePath, token: UserID) -> FileDownloadResponse {

    let abs_path = to_abs_data_path(&token, Borrow::<Path>::borrow(&path));

    if abs_path.is_dir() {
        // handle zip file
        let cons = zipwriter::new_zip_writer(abs_path).unwrap();
        FileDownloadResponse::Zip(Stream::chunked(cons, 4096))
    } else {
        match NamedFile::open(&abs_path).await {
            Ok(nf) => FileDownloadResponse::File(nf),
            Err(e) => {
                warn!("Error while reading file {:?} : {:?}", abs_path, e);
                FileDownloadResponse::NotFound(())
            }
        }
    }
}

#[get("/download/file?<path>&<shared_id>", rank = 2)]
pub async fn download_shared_file(mut path: NetFilePath, shared_id: &RawStr, db: State<'_, SharedDatabase>) -> FileDownloadResponse {

    if let Some(se) = db.get_shared_entry(&shared_id) {
        
        path.add_prefix(&se.path);
        
        download_file(path, se.user).await
    } else {
        FileDownloadResponse::Unauthorized(())
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_netfilepath() {
        let mut nfp = NetFilePath::from_path("/folder1/test");
        assert_eq!(Borrow::<str>::borrow(&nfp), "/folder1/test");
        nfp.add_prefix("\\User1\\");
        assert_eq!(Borrow::<str>::borrow(&nfp), "/User1/folder1/test");
    }
}
