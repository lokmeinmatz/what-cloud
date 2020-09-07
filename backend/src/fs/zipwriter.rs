use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use streamed_zip_rs;
use std::io::Read;
use ringbuf::{Consumer, RingBuffer};
use log::{info, error, warn};
use lazy_static::lazy_static;

lazy_static! {
    /// Thread handle and a terminated flag
    static ref ZIP_WRITER_THREAD: Mutex<Vec<(JoinHandle<()>, Arc<AtomicBool>)>> = Mutex::new(Vec::new());

}
const MAX_ZIP_WRITERS: usize = 4;

/// A Consumer that blocks until data is in the buffer, and
/// can get terminated from the Producer
pub struct BlockingConsumer {
    inner: Consumer<u8>,
    terminator: Arc<AtomicBool>
}

impl BlockingConsumer {
    /// Returns Self and an Arc<AtomicBool>> to terminate the Consumer
    pub fn new(inner: Consumer<u8>) -> (Self, Arc<AtomicBool>) {
        let terminator = Arc::new(AtomicBool::new(false));
        (Self {
            inner,
            terminator: terminator.clone()
        }, terminator)
    }

    pub fn read_blocking(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while self.inner.is_empty() && !self.terminator.load(Ordering::SeqCst) {
            std::thread::yield_now();
        }
        if self.terminator.load(Ordering::SeqCst) {
            return Ok(0);
        }
        self.inner.read(buf)
    }
}

impl Read for BlockingConsumer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_blocking(buf)
    }
}

pub fn new_zip_writer(path: std::path::PathBuf) -> Result<BlockingConsumer, &'static str> {

    let mut threads = ZIP_WRITER_THREAD.lock().map_err(|_| "Failed to get lock")?;

    // check if any old threads can get cleaned up
    let mut i = 0;
    while i < threads.len() {
        if !threads[i].1.load(Ordering::SeqCst) {
            // thread is terminated, can get replaced
            let jh = threads.remove(i).0;
            if let Err(e) = jh.join() {
                error!("Some ZIP writer thread join returned Err: {:?}", e);
            }
        }
        i += 1;
    }
    let len = threads.len();
    if len >= MAX_ZIP_WRITERS {
        warn!("Tried to create more ZIP writer threads than allowed");
        return Err("Cannot produce more than 4 zip streams at the same time :(");
    }
    let (prod, cons) = RingBuffer::new(4096).split();
    let (cons, term) = BlockingConsumer::new(cons);
    let term2 = term.clone();
    let worker = std::thread::Builder::new()
    .name(format!("ZIP worker #{}", len))
    .spawn(move || {
        info!("Starting zip folder stream of worker #{}", len);
        if let Err(e) = streamed_zip_rs::ZipStream::stream_folder(prod, &path) {
            error!("Error while streaming zip: {:?}", e);
        }
        info!("Finished zip folder stream of worker #{}", len);
        term.store(true, Ordering::SeqCst);
    }).map_err(|_| "Failed to start ZIP worker thread")?;
    threads.push((worker, term2));
    Ok(cons)
}