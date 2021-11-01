use std::io::SeekFrom;
use std::task::Poll;

use rocket::http;
use rocket::response::{self, Responder};
use rocket::request::Request;

use tokio::io::{AsyncRead, AsyncSeek, AsyncSeekExt, ReadBuf};

#[derive(Debug)]
pub struct PartialFile {
    file: tokio::fs::File,
    range: std::ops::RangeInclusive<u64>,
    total_size: u64,
    bytes_read: u64
}

impl PartialFile {
    pub async fn new(mut file: tokio::fs::File, range: std::ops::RangeInclusive<u64>) -> Self {
        file.seek(SeekFrom::Start(*range.start())).await.unwrap();
        let total_size = file.metadata().await.unwrap().len();
        Self {
            file,
            range: std::ops::RangeInclusive::new(
                *range.start(),
                *range.start() + (range.end() - range.start() + 1).min(total_size - range.start())
            ),
            total_size,
            bytes_read: 0
        }
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
            .sized_body(Some((end - start) as usize + 1), self)
            .finalize();
        
        Ok(dbg!(res))
    }
}


impl AsyncRead for PartialFile {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf,
    ) -> Poll<std::io::Result<()>> {

        let total_range = 1 + self.range.end() - self.range.start();

        if self.bytes_read >= total_range {
            println!("Finished PartialFile async read");
            return std::task::Poll::Ready(Ok(()));
        }

        let total_remaining = (total_range - self.bytes_read) as usize;
        
        let file = &mut self.file;
        
        tokio::pin!(file);

        let mut bytes_in_buf = buf.filled().len() as u64;
        
        let poll_res = if buf.remaining() > total_remaining {
            // only read up to total_remaining bytes into an manual created ReadBuf to not read more than needed
            // TODO: can thsi get refactored to one single poll_read with truncated buf?
            let mut read_buf_raw = vec![0u8; total_remaining];
            let mut read_buf = ReadBuf::new(&mut read_buf_raw); 
            bytes_in_buf = 0;
            let p_res = file.poll_read(cx, &mut read_buf);
            buf.put_slice(read_buf.filled());
            p_res
        } else {
            file.poll_read(cx, buf)
        };
        
        
        match poll_res {
            Poll::Ready(Ok(())) => {
                self.bytes_read += buf.filled().len() as u64 - bytes_in_buf;
                Poll::Ready(Ok(()))
            },
            e => e
        }
    }
}

impl AsyncSeek for PartialFile {
    fn start_seek(
        mut self: std::pin::Pin<&mut Self>,
        position: SeekFrom,
    ) -> std::io::Result<()> {
        let new_pos = match position {
            SeekFrom::Start(s) => SeekFrom::Start(s + self.range.start()),
            SeekFrom::End(e) => SeekFrom::End(*self.range.end() as i64 - self.total_size as i64 + 1 + e),
            _ => position
        };
        debug!("{:?} -> {:?}", position, new_pos);
        let file = &mut self.file;
        tokio::pin!(file);
        file.start_seek(new_pos)
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