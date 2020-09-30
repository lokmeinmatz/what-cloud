use std::io::Read;
use std::io::Write;
use std::sync::Arc;

use crossbeam::sync;
use ringbuf::{Consumer, Producer, RingBuffer};

pub struct BlockingProducer {
    pub(crate) inner: Producer<u8>,
    pub(crate) produced: sync::Unparker,
    pub(crate) wait_consumed: sync::Parker,
    pub(crate) alive: Arc<()>,
}

impl Write for BlockingProducer {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        
        // wait until space is in consumer
        while self.inner.is_full() && Arc::strong_count(&self.alive) > 1 {
            self.wait_consumed.park();
        }

        // we are the only holder of this Arc -> Consumer was dropped
        // this is an error
        if Arc::strong_count(&self.alive) == 1 {
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

            if Arc::strong_count(&self.alive) > 1 {
                return Err(std::io::Error::from(std::io::ErrorKind::WouldBlock));
            }
        } 

        Ok(())
    }
}

/// A Consumer that blocks until data is in the buffer, and
/// can get terminated if the Producer is dropped
pub struct BlockingConsumer {
    pub(crate) inner: Consumer<u8>,
    pub(crate) consumed: sync::Unparker,
    pub(crate) wait_produced: sync::Parker,
    pub(crate) alive: Arc<()>,
}

impl BlockingConsumer {
    /// spinloop waiting for data
    /// If waiting for longer periods, implement via parking / condvar?
    pub fn read_blocking(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // if no data currently is available to read, park until produced
        // also check if producer end is still alive to terminate if is dead
        while self.inner.is_empty() && Arc::strong_count(&self.alive) > 1 {
            self.wait_produced.park();
        }

        // we are the only holder of this Arc -> Producer was dropped
        if Arc::strong_count(&self.alive) == 1 {
            return Ok(0);
        }
        let read = self.inner.read(buf)?;
        self.consumed.unpark();
        Ok(read)
    }
}

impl Read for BlockingConsumer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_blocking(buf)
    }
}

pub fn split_blocking(rb: RingBuffer<u8>) -> (BlockingProducer, BlockingConsumer) {
    let (prod, cons) = rb.split();

    let wait_consumed = sync::Parker::new();
    let wait_produced = sync::Parker::new();
    let consumed = wait_consumed.unparker().clone();
    let produced = wait_produced.unparker().clone();
    let alive = Arc::new(());
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
