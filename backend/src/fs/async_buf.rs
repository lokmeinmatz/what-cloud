use std::io::Read;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crossbeam::{atomic, sync};
use ringbuf::{Consumer, Producer, RingBuffer};
use std::task::Waker;

pub struct BlockingProducer {
    pub(crate) inner: Producer<u8>,
    pub(crate) produced: Arc<atomic::AtomicCell<Option<Waker>>>,
    pub(crate) wait_consumed: sync::Parker,
    pub(crate) alive: Arc<AtomicUsize>,
}

impl BlockingProducer {
    pub fn consumer_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst) > 1
    }
}

impl Write for BlockingProducer {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        // wait until space is in consumer
        while self.inner.is_full() && self.consumer_alive() {
            self.wait_consumed.park();
        }

        // we are the only holder of this Arc -> Consumer was dropped
        // this is an error
        if !self.consumer_alive() {
            return Err(std::io::Error::from(std::io::ErrorKind::WouldBlock));
        }

        let written = self.inner.write(b)?;
        self.produced.take().map(|w| w.wake()); // wake consumer because dat was added
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        // wait until buffer is empty
        while !self.inner.is_empty() {
            self.wait_consumed.park();
            if !self.consumer_alive() && !self.inner.is_empty() {
                return Err(std::io::Error::from(std::io::ErrorKind::WouldBlock));
            }
        }

        Ok(())
    }
}

impl Drop for BlockingProducer {
    fn drop(&mut self) {
        self.alive.fetch_sub(1, Ordering::SeqCst);
        self.produced.take().map(|w| w.wake());
    }
}

/// A Consumer that blocks until data is in the buffer, and
/// can get terminated if the Producer is dropped
pub struct AsyncConsumer {
    pub(crate) inner: Consumer<u8>,
    pub(crate) produced_waker: Arc<atomic::AtomicCell<Option<Waker>>>,
    pub(crate) consumed: sync::Unparker,
    pub(crate) alive: Arc<AtomicUsize>,
}

impl AsyncConsumer {
    pub fn producer_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst) > 1
    }
}

impl Drop for AsyncConsumer {
    fn drop(&mut self) {
        self.alive.fetch_sub(1, Ordering::SeqCst);
        // wake producer if sleeping
        self.consumed.unpark();
    }
}

/*
fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    // if no data currently is available to read, park until produced
    // also check if producer end is still alive to terminate if is dead
    while self.inner.is_empty() && self.producer_alive() {
        self.wait_produced.park();
    }


    if self.inner.is_empty() {
        return Ok(0);
    }

    // we are the only holder of this Arc -> Producer was dropped
    let read = self.inner.read(buf)?;
    self.consumed.unpark();
    Ok(read)
}*/

use std::pin::Pin;
use std::task::{Context, Poll};
impl rocket::tokio::io::AsyncRead for AsyncConsumer {
    fn poll_read(
        mut self: Pin<&mut Self>,
        ctx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        // if no data currently is available to read, park until produced
        // also check if producer end is still alive to terminate if is dead
        if self.inner.is_empty() && self.producer_alive() {
            self.consumed.unpark();
            // store waker so the BlockingProducer can call the waker if produced any data
            self.produced_waker.store(Some(ctx.waker().clone()));
            return Poll::Pending;
        }

        if self.inner.is_empty() {
            return Poll::Ready(Ok(0));
        }

        // we are the only holder of this Arc -> Producer was dropped
        let read = self.inner.read(buf)?;

        // wake producer
        self.consumed.unpark();
        Poll::Ready(Ok(read))
    }
}

pub fn split_blocking_async(rb: RingBuffer<u8>) -> (BlockingProducer, AsyncConsumer) {
    let (prod, cons) = rb.split();

    let wait_consumed = sync::Parker::new();
    let consumed = wait_consumed.unparker().clone();
    let produced = Arc::new(atomic::AtomicCell::new(None));
    let alive = Arc::new(AtomicUsize::new(2));
    (
        BlockingProducer {
            inner: prod,
            produced: produced.clone(),
            wait_consumed,
            alive: alive.clone(),
        },
        AsyncConsumer {
            inner: cons,
            produced_waker: produced,
            consumed,
            alive,
        },
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use ringbuf::RingBuffer;
    use tokio::io::AsyncReadExt;

    #[test]
    fn test() {
        let rb = RingBuffer::new(512);
        let (mut prod, mut cons) = split_blocking_async(rb);
        let mut rbuf = vec![0u8; 1024 * 4];

        std::thread::spawn(move || {
            let mut wbuf = vec![1u8; 1024];
            for _ in 0..16 {
                std::thread::yield_now();
                prod.write_all(&mut wbuf).expect("write failed");
            }
        });
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            loop {
                eprintln!("waiting for data");
                let r = cons.read(&mut rbuf).await.expect("read failed");
                if r == 0 {
                    break;
                }
                assert!(&rbuf[..r].iter().all(|e| *e == 1u8));
                eprintln!("read {} bytes", r);
            }
        })
    }
}
