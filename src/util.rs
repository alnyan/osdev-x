//! Synchronization utilities
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    panic,
    sync::atomic::{AtomicBool, Ordering},
};

/// Wrapper struct to ensure a value can only be initialized once and used only after that
#[repr(C)]
pub struct OneTimeInit<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    state: AtomicBool,
}

/// Locked struct allowing shared mutable access to the wrapped value
#[repr(C)]
pub struct SpinLock<T> {
    value: UnsafeCell<T>,
    state: AtomicBool,
}

/// Wrapper for a lock()ed [SpinLock] value
#[repr(C)]
pub struct SpinLockGuard<'a, T> {
    value: *mut T,
    lock: &'a SpinLock<T>,
}

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

impl<T> SpinLock<T> {
    /// Wraps the value in a [SpinLock] structure
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: AtomicBool::new(false),
        }
    }

    /// Blocks until no other lock is held on the object, then locks it and returns a
    /// [SpinLockGuard]
    pub fn lock(&self) -> SpinLockGuard<T> {
        while self
            .state
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            aarch64_cpu::asm::nop();
        }

        SpinLockGuard {
            value: self.value.get(),
            lock: self,
        }
    }

    /// Resets the lock.
    ///
    /// # Safety
    ///
    /// Only safe to use from a [SpinLockGuard]'s [Drop] impl.
    pub unsafe fn force_release(&self) {
        self.state.store(false, Ordering::Release);
    }
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.value) }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.value) }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            self.lock.force_release();
        }
    }
}
