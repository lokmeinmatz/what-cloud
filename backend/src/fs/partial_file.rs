use std::io::SeekFrom;

use rocket::http;
use rocket::response::{self, Responder};
use rocket::request::Request;

use tokio::io::{AsyncRead, AsyncSeek};

pub struct PartialFileResponse {
    file: PartialFile,
    total_size: u64
}

impl PartialFileResponse {
    pub async fn new(file: tokio::fs::File, range: std::ops::RangeInclusive<u64>, total_size: u64) -> Result<Self, ()> {
        Ok(Self {
            file: dbg!(PartialFile::new(file, range).await),
            total_size
        })
    }
}

impl<'r> Responder<'r, 'static> for PartialFileResponse {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        let start = *self.file.range.start();
        let end = *self.file.range.end();
        let res = response::Response::build()
            .status(http::Status::PartialContent)
            .raw_header("Content-Range", 
            format!("bytes {}-{}/{}", start, end, self.total_size))
            .sized_body(Some((end - start) as usize + 1), self.file)
            .finalize();
        
        Ok(dbg!(res))
    }
}


#[derive(Debug)]
pub struct PartialFile {
    file: tokio::fs::File,
    range: std::ops::RangeInclusive<u64>,
    bytes_left: u64,
    total_size: u64
}

impl PartialFile {
    pub async fn new(mut file: tokio::fs::File, range: std::ops::RangeInclusive<u64>) -> Self {
        file.seek(SeekFrom::Start(*range.start())).await.unwrap();
        let total_size = file.metadata().await.unwrap().len();
        let bytes_left = (range.end() - range.start() + 1).min(total_size - range.start());
        Self {
            file,
            bytes_left,
            range,
            total_size
        }
    }
}

impl AsyncRead for PartialFile {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {

        if self.bytes_left == 0 {
            return std::task::Poll::Ready(Ok(0));
        }

        let file = &mut self.file;
        tokio::pin!(file);
        
        match file.poll_read(cx, buf) {
            std::task::Poll::Ready(Ok(r)) => {
                self.bytes_left = self.bytes_left.saturating_sub(r as u64);
                std::task::Poll::Ready(Ok(r))
            },
            other => other
        }
    }
}

impl AsyncSeek for PartialFile {
    fn start_seek(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        position: SeekFrom,
    ) -> std::task::Poll<std::io::Result<()>> {
        let new_pos = match position {
            SeekFrom::Start(s) => SeekFrom::Start(s + self.range.start()),
            SeekFrom::End(e) => SeekFrom::End(*self.range.end() as i64 - self.total_size as i64 + 1 + e),
            _ => position
        };
        debug!("{:?} -> {:?}", position, new_pos);
        let file = &mut self.file;
        tokio::pin!(file);
        file.start_seek(cx, new_pos)
    }

    fn poll_complete(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<u64>> {
        let file = &mut self.file;
        tokio::pin!(file);
        match file.poll_complete(cx) {
            std::task::Poll::Ready(Ok(ofs)) => std::task::Poll::Ready(Ok(ofs - *self.range.start())),
            other => other
        }
    }
}