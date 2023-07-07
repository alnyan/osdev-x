use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    panic,
    sync::atomic::{AtomicBool, Ordering},
};

#[repr(C)]
pub struct OneTimeInit<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    state: AtomicBool,
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

    // FIXME remove this and add a spinlock instead
    #[allow(clippy::mut_from_ref)]
    #[track_caller]
    pub fn get_mut(&self) -> &mut T {
        if !self.state.load(Ordering::Acquire) {
            panic!(
                "{:?}: Attempt to dereference an uninitialized value",
                panic::Location::caller()
            );
        }

        unsafe { (*self.value.get()).assume_init_mut() }
    }
}
