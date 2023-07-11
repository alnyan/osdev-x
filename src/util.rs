//! Synchronization utilities
use core::{
    cell::UnsafeCell,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    panic,
    sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
};

use aarch64_cpu::registers::DAIF;
use spinning_top::lock_api::RawMutex;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::arch::aarch64::intrinsics;

pub struct SpinFence {
    value: AtomicUsize,
}

struct SpinlockInner<T> {
    value: UnsafeCell<T>,
    holder: AtomicUsize,
    state: AtomicBool,
}

struct SpinlockInnerGuard<'a, T> {
    lock: &'a SpinlockInner<T>,
}

struct IrqGuard(u64);

pub struct IrqSafeSpinlock<T> {
    inner: SpinlockInner<T>,
}

pub struct IrqSafeSpinlockGuard<'a, T> {
    // Must come first to ensure the lock is dropped first and only then IRQs are re-enabled
    inner: SpinlockInnerGuard<'a, T>,
    irq_state: IrqGuard,
}

/// Statically-allocated "dynamic" vector
pub struct StaticVector<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

/// Wrapper struct to ensure a value can only be initialized once and used only after that
#[repr(C)]
pub struct OneTimeInit<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    state: AtomicBool,
}

unsafe impl<T> Sync for IrqSafeSpinlock<T> {}
unsafe impl<T> Send for IrqSafeSpinlock<T> {}

unsafe impl<T> Sync for OneTimeInit<T> {}
unsafe impl<T> Send for OneTimeInit<T> {}

impl<T> OneTimeInit<T> {
    /// Wraps the value in an [OneTimeInit]
    pub const fn new() -> Self {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicBool::new(false),
        }
    }

    /// Returns `true` if the value has already been initialized
    pub fn is_initialized(&self) -> bool {
        self.state.load(Ordering::Acquire)
    }

    /// Sets the underlying value of the [OneTimeInit]. If already initialized, panics.
    #[track_caller]
    pub fn init(&self, value: T) {
        if self
            .state
            .compare_exchange(false, true, Ordering::Release, Ordering::Relaxed)
            .is_err()
        {
            panic!(
                "{:?}: Double initialization of OneTimeInit<T>",
                panic::Location::caller()
            );
        }

        unsafe {
            (*self.value.get()).write(value);
        }
    }

    /// Returns an immutable reference to the underlying value and panics if it hasn't yet been
    /// initialized
    #[track_caller]
    pub fn get(&self) -> &T {
        if !self.state.load(Ordering::Acquire) {
            panic!(
                "{:?}: Attempt to dereference an uninitialized value",
                panic::Location::caller()
            );
        }

        unsafe { (*self.value.get()).assume_init_ref() }
    }
}

impl<T, const N: usize> StaticVector<T, N> {
    /// Constructs an empty instance of [StaticVector]
    pub const fn new() -> Self
    where
        T: Copy,
    {
        Self {
            data: [MaybeUninit::uninit(); N],
            len: 0,
        }
    }

    /// Appends an item to the vector.
    ///
    /// # Panics
    ///
    /// Will panic if the vector is full.
    pub fn push(&mut self, value: T) {
        if self.len == N {
            panic!("Static vector overflow: reached limit of {}", N);
        }

        self.data[self.len].write(value);
        self.len += 1;
    }

    /// Returns the number of items present in the vector
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T, const N: usize> Deref for StaticVector<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { MaybeUninit::slice_assume_init_ref(&self.data[..self.len]) }
    }
}

impl<T> SpinlockInner<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            holder: AtomicUsize::new(0),
            state: AtomicBool::new(false),
        }
    }

    pub fn lock_track(&self, holder: usize) -> SpinlockInnerGuard<T> {
        // Loop until the lock can be acquired
        while self
            .state
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        self.holder.store(holder, Ordering::SeqCst);

        SpinlockInnerGuard { lock: self }
    }
}

impl<T> IrqSafeSpinlock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: SpinlockInner::new(value),
        }
    }

    pub fn lock(&self) -> IrqSafeSpinlockGuard<T> {
        let mut caller;
        unsafe {
            core::arch::asm!("mov {0}, lr", out(reg) caller);
        }

        // Disable IRQs to avoid IRQ handler trying to acquire the same lock
        let irq_state = IrqGuard(DAIF.get());
        DAIF.modify(DAIF::I::SET);

        // Acquire the inner lock
        let inner = self.inner.lock_track(caller);

        IrqSafeSpinlockGuard { inner, irq_state }
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
        self.lock.holder.store(0, Ordering::SeqCst);
        self.lock
            .state
            .compare_exchange(true, false, Ordering::Release, Ordering::Relaxed)
            .unwrap();
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

impl Drop for IrqGuard {
    fn drop(&mut self) {
        DAIF.set(self.0);
    }
}

impl SpinFence {
    pub const fn new() -> Self {
        Self {
            value: AtomicUsize::new(0),
        }
    }

    pub fn reset(&self) {
        self.value.store(0, Ordering::Release);
    }

    pub fn signal(&self) {
        self.value.fetch_add(1, Ordering::SeqCst);
    }

    pub fn wait_all(&self, count: usize) {
        while self.value.load(Ordering::Acquire) < count {
            aarch64_cpu::asm::nop();
        }
    }

    pub fn wait_one(&self) {
        self.wait_all(1);
    }
}
