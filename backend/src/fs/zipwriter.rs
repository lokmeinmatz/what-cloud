use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};

// use super::async_buf::{split_blocking_async, AsyncConsumer};
use lazy_static::lazy_static;
use log::{error, info};
use ringbuf::RingBuffer;
use streamed_zip_rs;
lazy_static! {
    /// Thread handle and a terminated flag
    static ref ZIP_WRITER_THREAD_COUNT: Mutex<usize> = Mutex::new(0);
    static ref ZIP_WRITER_FINISHED: Condvar = Condvar::new();
}

static WRITER_ID: AtomicUsize = AtomicUsize::new(0);
const MAX_ZIP_WRITERS: usize = 4;

/// Creates a new thread (if not more than MAX_ZIP_WRITERS exist), who will stream the folder into an ringbuffer with 4KiB size.
/// Thread is responsible for decrementing counter if finished and notify change via condvar
pub fn new_zip_writer(path: std::path::PathBuf) -> Result<AsyncConsumer, &'static str> {
    // maybe find a better way to fail if waited too long??
    let mut thread_count = ZIP_WRITER_THREAD_COUNT
        .lock()
        .map_err(|_| "Failed to get lock")?;

    thread_count = ZIP_WRITER_FINISHED
        .wait_while(thread_count, |tc| *tc >= MAX_ZIP_WRITERS)
        .map_err(|_| "Failed to get lock")?;

    *thread_count += 1;
    // unlock
    drop(thread_count);

    let (prod, cons) = split_blocking_async(RingBuffer::new(4096));

    let id = WRITER_ID.fetch_add(1, Ordering::SeqCst);
    std::thread::Builder::new()
        .name(format!("ZIP worker #{}", id))
        .spawn(move || {
            let start = std::time::Instant::now();
            info!("Starting zip folder stream of worker #{}", id);
            if let Err(e) = streamed_zip_rs::ZipStream::stream_folder(prod, &path) {
                error!("Error while streaming zip: {:?}", e);
            }

            // terminate consumer
            // decrease counter and notify others that they can continue
            let mut thread_count = ZIP_WRITER_THREAD_COUNT
                .lock()
                .expect("Zip worker failed to get lock, its poisoned");
            *thread_count -= 1;
            info!(
                "Finished zip folder stream of worker #{} | ZIP worker active: {} | took {}s",
                id,
                thread_count,
                start.elapsed().as_secs_f64()
            );
            ZIP_WRITER_FINISHED.notify_one();
        })
        .map_err(|_| "Failed to start ZIP worker thread")?;
    Ok(cons)
}
