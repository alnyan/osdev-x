use core::{mem::MaybeUninit, sync::atomic::{AtomicBool, Ordering}, ops::{Deref, DerefMut}, cell::UnsafeCell};


#[repr(C)]
pub struct OneTimeInit<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    state: AtomicBool
}

unsafe impl<T> Sync for OneTimeInit<T> {}
unsafe impl<T> Send for OneTimeInit<T> {}

impl<T> OneTimeInit<T> {
    pub const fn new() -> Self {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicBool::new(false)
        }
    }

    pub fn init(&self, value: T) {
        if self.state.compare_exchange(false, true, Ordering::Release, Ordering::Relaxed).is_err() {
            loop {}
        }

        unsafe {
            (*self.value.get()).write(value);
        }
    }

    pub fn get(&self) -> &T {
        if !self.state.load(Ordering::Acquire) {
            // TODO handle this
            loop {}
        }

        unsafe {
            (*self.value.get()).assume_init_ref()
        }
    }

    pub fn get_mut(&self) -> &mut T {
        if !self.state.load(Ordering::Acquire) {
            loop {}
        }

        unsafe {
            (*self.value.get()).assume_init_mut()
        }
    }
}
