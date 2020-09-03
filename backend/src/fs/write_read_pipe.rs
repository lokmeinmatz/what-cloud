use std::sync::mpsc::{Receiver, SyncSender};
/// idea: let the WriterAdapter write an amount of bytes to an channel, which the ReaderAdapter can poll on read

pub struct WRPipeWriterAdapter {
    chan: SyncSender<Box<[u8]>>,
}

pub struct WRPipeReaderAdapter {
    chan: Receiver<Box<[u8]>>,
    local_buf: Vec<u8>
}

impl std::io::Write for WRPipeWriterAdapter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let size = buf.len();
        self.chan.send(buf.into()).map(|_| size).map_err(|_| std::io::Error::from(std::io::ErrorKind::BrokenPipe))
    }
    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        todo!()
    }
}

impl std::io::Read for WRPipeReaderAdapter {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        if self.local_buf.is_empty() {
            // load new data
            let data = match self.chan.recv() {
                Ok(d) => d,
                Err(_) => return Ok(0)
            };

            // received packet fits in buf? copy all
            if data.len() <= buf.len() {
                (&mut buf[0..data.len()]).copy_from_slice(&data);
                return Ok(data.len());
            }

            // packet is larger than buf? write all fitting
            buf.copy_from_slice(&data[0..buf.len()]);

            // copy remaining bytes into local_buf
            self.local_buf = (&data[buf.len()..]).into();
            return Ok(buf.len());
        }

        // use stored data from local_buf
        if self.local_buf.len() <= buf.len() {
            // copy all local_buf
            (&mut buf[0..self.local_buf.len()]).copy_from_slice(&self.local_buf);
            let size = self.local_buf.len();
            self.local_buf.clear();
            return Ok(size);
        }

        // fill buf with next bytes of local_buf
        buf.copy_from_slice(&self.local_buf[0..buf.len()]);
        self.local_buf.splice(0..buf.len(), std::iter::empty());
        Ok(buf.len())
    }
}
