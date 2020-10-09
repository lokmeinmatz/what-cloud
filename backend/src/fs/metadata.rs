use crate::auth::UserID;
use crate::database::SharedDatabase;
use log::{info, warn};
use rocket::State;
use std::path::{Path, PathBuf};

#[derive(Serialize, Debug)]
pub struct NodeMetadata {
    #[serde(rename = "type")]
    node_type: &'static str,
    size: f64,
    #[serde(rename = "lastModified")]
    last_modified: chrono::DateTime<chrono::Utc>,
    shared: Option<String>,
}

/// Path from user perspective (not absolute)
pub fn get_metadata(
    path: &Path,
    user_id: &UserID,
    db: State<SharedDatabase>,
) -> Option<NodeMetadata> {
    let shared: Option<String> = db
        .get_share_id(&user_id, path)
        .ok()
        .flatten();


    let combined = super::to_abs_data_path(&user_id, path);
    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user_id.0);
    if !root.exists() {
        match std::fs::create_dir(&root) {
            Ok(()) => info!("Created base dir of user {}", user_id.0),
            Err(e) => warn!("Failed to create base dir of user {}: {:?}", user_id.0, e),
        }
    }

    if !combined.exists() {
        // check if user has allready folder or needs to get created
        return None;
    }

   
    match std::fs::metadata(&combined) {
        Err(_) => None,
        Ok(meta) => {
            let last_mod: chrono::DateTime<chrono::Utc> = {
                let dur: std::time::Duration = meta
                    .modified()
                    .unwrap()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Timestamp of file to DateTime failed");
                use chrono::TimeZone;
                chrono::Utc.timestamp(dur.as_secs() as i64, 0)
            };

            if meta.file_type().is_dir() {
                // calculate folder size
                // First idea: calc size of all files in folder and write >= size?
                // or cache all calculated sizes somewhere
                let size: u64 = std::fs::read_dir(&combined)
                    .unwrap()
                    .map(|f| {
                        let direntry: std::fs::DirEntry = f.unwrap();
                        if direntry.file_type().unwrap().is_file() {
                            return direntry.metadata().unwrap().len();
                        }
                        0
                    })
                    .sum();

                Some(NodeMetadata {
                    node_type: "folder",
                    size: size as f64,
                    last_modified: last_mod,
                    shared,
                })
            } else {
                Some(NodeMetadata {
                    node_type: "file",
                    size: meta.len() as f64,
                    last_modified: last_mod,
                    shared,
                })
            }
        }
    }
}
