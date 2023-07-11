//! Synchronization utilities
use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::Deref,
    panic,
    sync::atomic::{AtomicBool, Ordering},
};

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
