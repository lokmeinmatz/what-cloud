use crate::database::SharedDatabase;
use crate::fs::{to_abs_data_path, /* zipwriter,*/ NetFilePath, UserID};
use log::warn;
use rocket::{Request, State};
use rocket::fs::NamedFile;
use rocket::response::Responder;
use rocket::request::FromRequest;
use std::borrow::Borrow;
use std::path::Path;
use regex::Regex;

use super::partial_file::PartialFileResponse;

#[derive(Responder)]
pub enum FileDownloadResponse {
    #[response(status = 200)]
    File(RangeAcceptingFile),
    #[response(status = 206)]
    PartialFile(PartialFileResponse),
    /* #[response(status = 200)]
    Zip(ByteStream), */
    #[response(status = 401)]
    Unauthorized(()),
    #[response(status = 404)]
    NotFound(()),
}


pub struct RequestedRange{
    start: u64,
    end: Option<u64>
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestedRange {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        if let Some(range_raw) = req.headers().get_one("Range") {
            // TODO lazy static
            let re = Regex::new(r#"bytes=(\d+)-(\d+)?"#).unwrap();
            if let Some(cap) = re.captures(range_raw) {
                let from: u64 = if let Some(f) = cap.get(1)
                    .and_then(|rmatch| rmatch.as_str().parse().ok()) { f } 
                else { return rocket::request::Outcome::Forward(()) };

                let to: Option<u64> = cap.get(2).and_then(|rmatch| rmatch.as_str().parse().ok());
                if let Some(true) = to.map(|to| to < from) {
                    return rocket::request::Outcome::Forward(());
                }
                println!("{}-{:?}", from, to);
                return rocket::request::Outcome::Success(RequestedRange {
                    start: from,
                    end: to
                })
            }
        }
        rocket::request::Outcome::Forward(())
    }
}

pub struct RangeAcceptingFile(NamedFile);

impl<'r> Responder<'r, 'static> for RangeAcceptingFile {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        self.0.respond_to(request).map(|mut res| {
            res.set_raw_header("Accept-Ranges", "bytes");
            res
        })
    }
}

const PARTIAL_MAX_SIZE: u64 = 1024 * 1024;

#[get("/download/file?<path>&<token>", rank = 1)]
pub async fn download_file(path: NetFilePath, token: UserID, range: Option<RequestedRange>) -> FileDownloadResponse {
    let abs_path = to_abs_data_path(&token, Borrow::<Path>::borrow(&path));

    if abs_path.is_dir() {
        // handle zip file
        /* 
        let cons = zipwriter::new_zip_writer(abs_path).unwrap();
        FileDownloadResponse::Zip(ByteStream!{

        }) */
        panic!("Zip ByteStream not implemented")
    } else if let Some(req_range) = range {
        let start = req_range.start;
        
        let file = if let Ok(f) = tokio::fs::File::open(abs_path).await {
            f
        } else {
            return FileDownloadResponse::NotFound(())
        };
        let total_size = file.metadata().await.unwrap().len();
        if total_size <= start {
            warn!("start >= total_size");
            // TODO right http return type
            return FileDownloadResponse::NotFound(());
        }

        // limit end to last byte
        let end = req_range.end.unwrap_or(start + PARTIAL_MAX_SIZE - 1).min(total_size - 1);
        
        // partial file
        if let Ok(pfr) = PartialFileResponse::new(file,start..=end, total_size).await {
            FileDownloadResponse::PartialFile(pfr)
        } else {
            // TODO better eerror handling
            FileDownloadResponse::NotFound(())
        }
    } else {
        match NamedFile::open(&abs_path).await {
            Ok(nf) => FileDownloadResponse::File(RangeAcceptingFile(nf)),
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
    shared_id: &str,
    db: &State<SharedDatabase>,
    range: Option<RequestedRange>
) -> FileDownloadResponse {
    if let Some(se) = db.get_shared_entry(&shared_id) {
        path.add_prefix(&se.path);

        download_file(path, se.user, range).await
    } else {
        FileDownloadResponse::Unauthorized(())
    }
}
