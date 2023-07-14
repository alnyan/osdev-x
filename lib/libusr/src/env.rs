static mut ARG: usize = 0;

pub fn args() -> impl Iterator<Item = usize> {
    core::iter::once(unsafe { ARG })
}

pub(crate) unsafe fn init_args(arg: usize) {
    ARG = arg;
}
