use std::sync::atomic::{AtomicUsize, Ordering};
use std::io::Read;
use std::io::Write;
use std::sync::Arc;

use crossbeam::sync;
use ringbuf::{Consumer, Producer, RingBuffer};

pub struct BlockingProducer {
    pub(crate) inner: Producer<u8>,
    pub(crate) produced: sync::Unparker,
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
        self.produced.unpark();
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
        self.produced.unpark();
    }
}

/// A Consumer that blocks until data is in the buffer, and
/// can get terminated if the Producer is dropped
pub struct BlockingConsumer {
    pub(crate) inner: Consumer<u8>,
    pub(crate) consumed: sync::Unparker,
    pub(crate) wait_produced: sync::Parker,
    pub(crate) alive: Arc<AtomicUsize>,
}

impl BlockingConsumer {

    pub fn producer_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst) > 1
    }

    /// spinloop waiting for data
    /// If waiting for longer periods, implement via parking / condvar?
    pub fn read_blocking(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
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
    }
}

impl Drop for BlockingConsumer {
    fn drop(&mut self) {
        self.alive.fetch_sub(1, Ordering::SeqCst);
        self.consumed.unpark();
    }
}

impl Read for BlockingConsumer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_blocking(buf)
    }
}

#[allow(dead_code)]
pub fn split_blocking(rb: RingBuffer<u8>) -> (BlockingProducer, BlockingConsumer) {
    let (prod, cons) = rb.split();

    let wait_consumed = sync::Parker::new();
    let wait_produced = sync::Parker::new();
    let consumed = wait_consumed.unparker().clone();
    let produced = wait_produced.unparker().clone();
    let alive = Arc::new(AtomicUsize::new(2));
    (
        BlockingProducer {
            inner: prod,
            produced,
            wait_consumed,
            alive: alive.clone(),
        },
        BlockingConsumer {
            inner: cons,
            consumed,
            wait_produced,
            alive,
        },
    )
}


#[cfg(test)]
mod test {
    use ringbuf::RingBuffer;
    use super::*;

    #[test]
    fn test_blocking() {
        let rb = RingBuffer::new(512);
        let (mut prod, mut cons) = split_blocking(rb);
        let mut rbuf = vec![0u8; 1024 * 4];

        std::thread::spawn(move || {
            let mut wbuf = vec![1u8; 1024];
            for _ in 0..1024 {
                std::thread::yield_now();
                prod.write_all(&mut wbuf).expect("write failed");
            }
        });
        loop {
            let r = cons.read(&mut rbuf).expect("read failed");
            if r == 0 {
                break
            }
            assert!(&rbuf[..r].iter().all(|e| *e == 1u8));
            eprintln!("read {} bytes", r);

        }
    }
}