use crate::database::SharedDatabase;
use crate::fs::{to_abs_data_path, zipwriter, NetFilePath, UserID};
use log::warn;
use rocket::http::RawStr;
use rocket::response::{NamedFile, Stream};
use rocket::State;
use std::borrow::Borrow;
use std::path::Path;

#[derive(Responder)]
pub enum FileDownloadResponse {
    #[response(status = 200)]
    File(NamedFile),
    #[response(status = 200)]
    Zip(Stream<super::async_buf::AsyncConsumer>),
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
pub async fn download_shared_file(
    mut path: NetFilePath,
    shared_id: &RawStr,
    db: State<'_, SharedDatabase>,
) -> FileDownloadResponse {
    if let Some(se) = db.get_shared_entry(&shared_id) {
        path.add_prefix(&se.path);

        download_file(path, se.user).await
    } else {
        FileDownloadResponse::Unauthorized(())
    }
}
