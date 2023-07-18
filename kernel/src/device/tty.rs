//! Terminal driver implementation
use abi::error::Error;

use crate::{proc::wait::Wait, sync::IrqSafeSpinlock};

use super::serial::SerialDevice;

struct CharRingInner<const N: usize> {
    rd: usize,
    wr: usize,
    data: [u8; N],
    flags: u8,
}

/// Ring buffer for a character device. Handles reads, writes and channel notifications for a
/// terminal device.
pub struct CharRing<const N: usize> {
    wait_read: Wait,
    wait_write: Wait,
    inner: IrqSafeSpinlock<CharRingInner<N>>,
}

/// Terminal device interface
pub trait TtyDevice<const N: usize>: SerialDevice {
    /// Returns the ring buffer associated with the device
    fn ring(&self) -> &CharRing<N>;

    /// Returns `true` if data is ready to be read from or written to the terminal
    fn is_ready(&self, write: bool) -> Result<bool, Error> {
        let ring = self.ring();
        if write {
            todo!();
        } else {
            Ok(ring.is_readable())
        }
    }

    /// Sends a single byte to the terminal
    fn line_send(&self, byte: u8) -> Result<(), Error> {
        self.send(byte)
    }

    /// Receives a single byte from the terminal
    fn recv_byte(&self, byte: u8) {
        let ring = self.ring();
        ring.putc(byte, false).ok();
    }

    /// Reads and processes data from the terminal
    fn line_read(&'static self, data: &mut [u8]) -> Result<usize, Error> {
        let ring = self.ring();

        if data.is_empty() {
            return Ok(0);
        }

        let byte = ring.getc()?;
        data[0] = byte;
        Ok(1)
    }

    /// Processes and writes the data to the terminal
    fn line_write(&self, data: &[u8]) -> Result<usize, Error> {
        for &byte in data {
            self.line_send(byte)?;
        }
        Ok(data.len())
    }

    /// Writes raw data to the terminal bypassing the processing functions
    fn raw_write(&self, _data: &[u8]) -> Result<usize, Error> {
        todo!();
    }
}

impl<const N: usize> CharRingInner<N> {
    #[inline]
    const fn is_readable(&self) -> bool {
        if self.rd <= self.wr {
            (self.wr - self.rd) > 0
        } else {
            (self.wr + (N - self.rd)) > 0
        }
    }

    #[inline]
    unsafe fn read_unchecked(&mut self) -> u8 {
        let res = self.data[self.rd];
        self.rd = (self.rd + 1) % N;
        res
    }

    #[inline]
    unsafe fn write_unchecked(&mut self, ch: u8) {
        self.data[self.wr] = ch;
        self.wr = (self.wr + 1) % N;
    }
}

impl<const N: usize> CharRing<N> {
    /// Constructs an empty ring buffer
    pub const fn new() -> Self {
        Self {
            inner: IrqSafeSpinlock::new(CharRingInner {
                rd: 0,
                wr: 0,
                data: [0; N],
                flags: 0,
            }),
            wait_read: Wait::new("char_ring_read"),
            wait_write: Wait::new("char_ring_write"),
        }
    }

    /// Returns `true` if the buffer has data to read
    pub fn is_readable(&self) -> bool {
        let inner = self.inner.lock();
        inner.is_readable() || inner.flags != 0
    }

    /// Reads a single character from the buffer, blocking until available
    pub fn getc(&'static self) -> Result<u8, Error> {
        let mut lock = self.inner.lock();
        loop {
            if !lock.is_readable() && lock.flags == 0 {
                drop(lock);
                self.wait_read.wait(None)?;
                lock = self.inner.lock();
            } else {
                break;
            }
        }

        let byte = unsafe { lock.read_unchecked() };
        drop(lock);
        self.wait_write.wakeup_one();
        // TODO WAIT_SELECT
        Ok(byte)
    }

    /// Sends a single character to the buffer
    pub fn putc(&self, ch: u8, blocking: bool) -> Result<(), Error> {
        let mut lock = self.inner.lock();
        if blocking {
            todo!();
        }
        unsafe {
            lock.write_unchecked(ch);
        }
        drop(lock);
        self.wait_read.wakeup_one();
        // TODO WAIT_SELECT
        Ok(())
    }
}
