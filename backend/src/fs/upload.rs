use rocket::Data;
use std::borrow::Borrow;
use std::path::PathBuf;
use super::NetFilePath;
use crate::database::SharedDatabase;
use crate::auth::UserID;
use log::{info, warn};

use rocket::response::status;
use rocket::State;

type UploadResponse = Result<status::Accepted<()>, status::Forbidden<()>>;

#[allow(unreachable_code, unused_variables)]
#[post("/upload?<file_path>&<shared_id>", data = "<data>", rank = 1)]
pub async fn post_upload_shared(
    mut file_path: NetFilePath,
    db: State<'_, SharedDatabase>,
    shared_id: String,
    data: Data
) -> UploadResponse {
    warn!("Upload for shared not implemented");
    return Err(status::Forbidden(None));
    // check if shared id is allowed
    if let Some(se) = db.get_shared_entry(&shared_id) {
        file_path.add_prefix(&se.path);

        return handle_upload(file_path, se.user, data).await;
    }

    // TODO add error details
    Err(status::Forbidden(None))
}

#[post("/upload?<file_path>", data = "<data>", rank = 2)]
pub async fn post_upload(
    file_path: NetFilePath,
    user_id: UserID,
    data: Data
) -> UploadResponse {
    handle_upload(file_path, user_id, data).await
}


#[post("/create_folder?<folder_path>")]
pub async fn post_create_folder(
    folder_path: NetFilePath,
    user_id: UserID
) -> UploadResponse {
    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user_id.0);
    if !root.exists() {
        match std::fs::create_dir(&root) {
            Ok(()) => info!("Created base dir of user {}", user_id.0),
            Err(e) => {
                warn!("Failed to create base dir of user {}: {:?}", user_id.0, e);
                return Err(status::Forbidden(None))
            },
        }
    }
    root.push(Borrow::<str>::borrow(&folder_path));
    if root.exists() {
        // check if user has allready folder or needs to get created
        warn!("User tried to create existin folder");
        return Err(status::Forbidden(None));
    }

    if let Err(e) = std::fs::create_dir_all(&root) {
        warn!("Error while creating folder: {:?}", e);
        return Err(status::Forbidden(None));
    }
    Ok(status::Accepted(None))
}

use rocket::data::ToByteUnit;

async fn handle_upload(
    folder_path: NetFilePath,
    user_id: UserID,
    upload: Data
) -> UploadResponse {
    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user_id.0);
    if !root.exists() {
        match std::fs::create_dir(&root) {
            Ok(()) => info!("Created base dir of user {}", user_id.0),
            Err(e) => {
                warn!("Failed to create base dir of user {}: {:?}", user_id.0, e);
                return Err(status::Forbidden(None))
            },
        }
    }
    root.push(Borrow::<str>::borrow(&folder_path));
    if root.exists() {
        // check if user has allready folder or needs to get created
        info!("User overwriting existing file");
    }
    info!("Streaing to file {:?}", root);
    // stream file to root
    match upload.open(10.gibibytes()).stream_to_file(&root).await {
        Ok(size) => {
            info!("Uploaded {} bytes to {:?}", size, root);
            Ok(status::Accepted(None))
        },
        Err(e) => {
            warn!("Upload failed: {}", e);
            Err(status::Forbidden(None))
        }
    }

}
