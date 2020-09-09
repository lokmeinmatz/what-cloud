use crate::auth::UserID;
use rocket::http::RawStr;
use rocket_contrib::json::Json;
use std::path::PathBuf;
use log::{info, warn};
use rocket::State;
use crate::database::SharedDatabase;

#[derive(Serialize, Debug)]
pub struct NodeMetadata {
    #[serde(rename = "type")]
    node_type: &'static str,
    size: f64,
    #[serde(rename = "lastModified")]
    last_modified: chrono::DateTime<chrono::Utc>,
    shared: Option<String>
}


#[derive(Responder, Debug)]
pub enum MetadataResponse {
    #[response(status = 400)]
    UnknownPath(String),
    #[response(status = 200)]
    Success(Json<NodeMetadata>)

}

#[get("/metadata?<url_encoded_path>")]
pub fn get_metadata(url_encoded_path: &RawStr, user_id: UserID, db: State<SharedDatabase>) -> MetadataResponse {
    let raw_path = match url_encoded_path.percent_decode() {
        Ok(s) => s.into_owned(),
        Err(e) => { return MetadataResponse::UnknownPath(e.to_string()); }
    };
    let shared: Option<String> = db.get_share_id(&user_id, &std::path::Path::new(&raw_path))
    .ok().flatten();

    //dbg!(&shared);

    let combined = match super::to_abs_data_path(&user_id, &raw_path) {
        Ok(c) => c,
        Err(()) => return MetadataResponse::UnknownPath("error: to_abs_data_path failed".into())
    };
    let mut root: PathBuf = PathBuf::from(crate::config::data_path());
    root.push(&user_id.0);
    if !root.exists() {
        match std::fs::create_dir(&root) {
            Ok(()) => { info!("Created base dir of user {}", user_id.0) },
            Err(e) => { warn!("Failed to create base dir of user {}: {:?}", user_id.0, e) }
        }
    }

    dbg!(&combined);
    
    if !combined.exists() {
        // check if user has allready folder or needs to get created
        return MetadataResponse::UnknownPath("Path doesn't exist".into())
    }



    info!("get_metadata on path {:?}", combined);
    //std::thread::sleep(std::time::Duration::from_secs(5));
    match std::fs::metadata(&combined) {
        Err(e) => MetadataResponse::UnknownPath(e.to_string()),
        Ok(meta) => {

            let last_mod : chrono::DateTime<chrono::Utc> = {
                let dur: std::time::Duration = meta.modified().unwrap().duration_since(std::time::UNIX_EPOCH).expect("Timestamp of file to DateTime failed");
                use chrono::TimeZone;
                chrono::Utc.timestamp(dur.as_secs() as i64, 0)
            };

            if meta.file_type().is_dir() {

                // calculate folder size
                // First idea: calc size of all files in folder and write >= size?
                // or cache all calculated sizes somewhere
                let size: u64 = std::fs::read_dir(&combined).unwrap().map(|f| {
                    let direntry: std::fs::DirEntry = f.unwrap();
                    if direntry.file_type().unwrap().is_file() {
                        return direntry.metadata().unwrap().len()
                    }
                    0
                }).sum();

                MetadataResponse::Success(Json(NodeMetadata {
                    node_type: "folder",
                    size: size as f64,
                    last_modified: last_mod,
                    shared
                }))
            } else {
                MetadataResponse::Success(Json(NodeMetadata {
                    node_type: "file",
                    size: meta.len() as f64,
                    last_modified: last_mod,
                    shared
                }))
            }

        }
    }


}