
use crate::fs::SharedDatabase;
use rocket::State;
use rocket::http::RawStr;
use std::path::Path;
use std::borrow::Borrow;
use crate::fs::to_abs_data_path;
use crate::fs::UserID;
use crate::fs::NetFilePath;
use rocket::response::NamedFile;
use rocket::response::Stream;

use log::{info, warn};

#[derive(Responder)]
pub enum FileDownloadResponse {
    #[response(status = 200)]
    File(NamedFile),
    #[response(status = 200)]
    Zip(Stream<super::blocking_buf::BlockingConsumer>),
    #[response(status = 401)]
    Unauthorized(()),
    #[response(status = 404)]
    NotFound(()),
}

#[get("/download/file?<path>&<token>", rank = 1)]
pub fn download_file(path: NetFilePath, token: UserID) -> FileDownloadResponse {
    info!("User {:?} requested download of {:?}", token, path);



    let abs_path = to_abs_data_path(&token, Borrow::<Path>::borrow(&path));

    if abs_path.is_dir() {
        // handle zip file
        let cons = super::zipwriter::new_zip_writer(abs_path).unwrap();
        FileDownloadResponse::Zip(Stream::chunked(cons, 4096))
    } else {
        match NamedFile::open(&abs_path) {
            Ok(nf) => FileDownloadResponse::File(nf),
            Err(e) => {
                warn!("Error while reading file {:?} : {:?}", abs_path, e);
                FileDownloadResponse::NotFound(())
            }
        }
    }
}

#[get("/download/file?<path>&<shared_id>", rank = 2)]
pub fn download_shared_file(mut path: NetFilePath, shared_id: &RawStr, db: State<SharedDatabase>) -> FileDownloadResponse {
    info!("Shared download of {:?}", path);

    if let Some(se) = db.get_shared_entry(&shared_id) {
        
        path.add_prefix(&se.path);
        
        download_file(path, se.user)
    } else {
        FileDownloadResponse::NotFound(())
    }
}