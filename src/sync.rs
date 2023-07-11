//! Synchronization primitives
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use aarch64_cpu::registers::DAIF;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

/// Simple spinloop-based fence guaranteeing that the execution resumes only after its condition is
/// met.
pub struct SpinFence {
    value: AtomicUsize,
}

/// Token type used to prevent IRQs from firing during some critical section. Normal IRQ operation
/// (if enabled before) is resumed when [IrqGuard]'s lifetime is over.
pub struct IrqGuard(u64);

struct SpinlockInner<T> {
    value: UnsafeCell<T>,
    state: AtomicBool,
}

struct SpinlockInnerGuard<'a, T> {
    lock: &'a SpinlockInner<T>,
}

/// Spinlock implementation which prevents interrupts to avoid deadlocks when an interrupt handler
/// tries to acquire a lock taken before the IRQ fired.
pub struct IrqSafeSpinlock<T> {
    inner: SpinlockInner<T>,
}

/// Token type allowing safe access to the underlying data of the [IrqSafeSpinlock]. Resumes normal
/// IRQ operation (if enabled before acquiring) when the lifetime is over.
pub struct IrqSafeSpinlockGuard<'a, T> {
    // Must come first to ensure the lock is dropped first and only then IRQs are re-enabled
    inner: SpinlockInnerGuard<'a, T>,
    _irq: IrqGuard,
}

// Spinlock impls
impl<T> SpinlockInner<T> {
    const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: AtomicBool::new(false),
        }
    }

    fn lock(&self) -> SpinlockInnerGuard<T> {
        // Loop until the lock can be acquired
        while self
            .state
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        SpinlockInnerGuard { lock: self }
    }
}

impl<'a, T> Deref for SpinlockInnerGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<'a, T> DerefMut for SpinlockInnerGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<'a, T> Drop for SpinlockInnerGuard<'a, T> {
    fn drop(&mut self) {
        self.lock
            .state
            .compare_exchange(true, false, Ordering::Release, Ordering::Relaxed)
            .unwrap();
    }
}

unsafe impl<T> Sync for SpinlockInner<T> {}
unsafe impl<T> Send for SpinlockInner<T> {}

// IrqSafeSpinlock impls
impl<T> IrqSafeSpinlock<T> {
    /// Wraps the value in a spinlock primitive
    pub const fn new(value: T) -> Self {
        Self {
            inner: SpinlockInner::new(value),
        }
    }

    /// Attempts to acquire a lock. IRQs will be disabled until the lock is released.
    pub fn lock(&self) -> IrqSafeSpinlockGuard<T> {
        // Disable IRQs to avoid IRQ handler trying to acquire the same lock
        let irq_guard = IrqGuard::acquire();

        // Acquire the inner lock
        let inner = self.inner.lock();

        IrqSafeSpinlockGuard {
            inner,
            _irq: irq_guard,
        }
    }
}

impl<'a, T> Deref for IrqSafeSpinlockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'a, T> DerefMut for IrqSafeSpinlockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

// IrqGuard impls
impl IrqGuard {
    /// Saves the current IRQ state and masks them
    pub fn acquire() -> Self {
        let this = Self(DAIF.get());
        DAIF.modify(DAIF::I::SET);
        this
    }
}

impl Drop for IrqGuard {
    fn drop(&mut self) {
        DAIF.set(self.0);
    }
}

// SpinFence impls
impl SpinFence {
    /// Constructs a new [SpinFence]
    pub const fn new() -> Self {
        Self {
            value: AtomicUsize::new(0),
        }
    }

    /// Resets a fence back to its original state
    pub fn reset(&self) {
        self.value.store(0, Ordering::Release);
    }

    /// "Signals" a fence, incrementing its internal counter by one
    pub fn signal(&self) {
        self.value.fetch_add(1, Ordering::SeqCst);
    }

    /// Waits until the fence is signalled at least the amount of times specified
    pub fn wait_all(&self, count: usize) {
        while self.value.load(Ordering::Acquire) < count {
            core::hint::spin_loop();
        }
    }

    /// Waits until the fence is signalled at least once
    pub fn wait_one(&self) {
        self.wait_all(1);
    }
}
