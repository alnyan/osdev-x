use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    panic,
    sync::atomic::{AtomicBool, Ordering},
};

#[repr(C)]
pub struct OneTimeInit<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    state: AtomicBool,
}

#[repr(C)]
pub struct SpinLock<T> {
    value: UnsafeCell<T>,
    state: AtomicBool,
}

#[repr(C)]
pub struct SpinLockGuard<'a, T> {
    value: *mut T,
    lock: &'a SpinLock<T>,
}

unsafe impl<T> Sync for OneTimeInit<T> {}
unsafe impl<T> Send for OneTimeInit<T> {}

impl<T> OneTimeInit<T> {
    pub const fn new() -> Self {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicBool::new(false),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.state.load(Ordering::Acquire)
    }

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
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<T> {
        while let Err(_) =
            self.state
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        {
            aarch64_cpu::asm::nop();
        }

        SpinLockGuard {
            value: self.value.get(),
            lock: self,
        }
    }

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
