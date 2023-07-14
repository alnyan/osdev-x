pub struct ExitCode(i32);

impl ExitCode {
    pub fn into_system_exit_code(self) -> i32 {
        self.0
    }
}

impl From<i32> for ExitCode {
    fn from(v: i32) -> Self {
        Self(v)
    }
}
