use crate::database::SharedDatabase;
use crate::fs::{to_abs_data_path, zipwriter, NetFilePath, UserID};
use log::warn;
use rocket::http;
use rocket::{http::RawStr, Request, State};
use rocket::response::{self, NamedFile, Stream, Responder};
use rocket::request::FromRequest;
use std::borrow::Borrow;
use std::path::Path;
use std::io::{SeekFrom};
use regex::Regex;

#[derive(Responder)]
pub enum FileDownloadResponse {
    #[response(status = 200)]
    File(RangeAcceptingFile),
    #[response(status = 206)]
    PartialFile(PartialFile),
    #[response(status = 200)]
    Zip(Stream<super::async_buf::AsyncConsumer>),
    #[response(status = 401)]
    Unauthorized(()),
    #[response(status = 404)]
    NotFound(()),
}

pub struct PartialFile {
    file: tokio::fs::File,
    range: std::ops::RangeInclusive<u64>,
    total_size: u64
}

impl PartialFile {
    pub async fn new(mut file: tokio::fs::File, range: std::ops::RangeInclusive<u64>, total_size: u64) -> Result<Self, ()> {
        file.seek(SeekFrom::Start(*range.start())).await.map_err(drop)?;
        // allready seeked file stored!!!
        Ok(PartialFile {
            file,
            range,
            total_size
        })
    }
}

impl<'r> Responder<'r, 'static> for PartialFile {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        let start = *self.range.start();
        let end = *self.range.end();
        let res = response::Response::build()
            .status(http::Status::PartialContent)
            .raw_header("Content-Range", 
            format!("bytes {}-{}/{}", start, end, self.total_size))
            .sized_body(Some((end - start) as usize + 1), self.file)
            .finalize();

            Ok(res)
    }
}


pub struct RequestedRange{
    start: u64,
    end: Option<u64>
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for RequestedRange {
    type Error = ();

    async fn from_request(req: &'a Request<'r>) -> rocket::request::Outcome<Self, Self::Error> {
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

#[get("/download/file?<path>&<token>", rank = 1)]
pub async fn download_file(path: NetFilePath, token: UserID, range: Option<RequestedRange>) -> FileDownloadResponse {
    let abs_path = to_abs_data_path(&token, Borrow::<Path>::borrow(&path));

    if abs_path.is_dir() {
        // handle zip file
        let cons = zipwriter::new_zip_writer(abs_path).unwrap();
        FileDownloadResponse::Zip(Stream::chunked(cons, 4096))
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
        let end = req_range.end.unwrap_or(start + 1023).min(total_size - 1);
        
        // partial file
        if let Ok(pf) = PartialFile::new(file,start..=end, total_size).await {
            FileDownloadResponse::PartialFile(pf)
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
    shared_id: &RawStr,
    db: State<'_, SharedDatabase>,
    range: Option<RequestedRange>
) -> FileDownloadResponse {
    if let Some(se) = db.get_shared_entry(&shared_id) {
        path.add_prefix(&se.path);

        download_file(path, se.user, range).await
    } else {
        FileDownloadResponse::Unauthorized(())
    }
}
