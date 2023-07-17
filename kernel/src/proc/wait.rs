use core::time::Duration;

use abi::error::Error;
use alloc::{collections::LinkedList, rc::Rc};

use crate::{
    arch::PLATFORM, device::platform::Platform, sync::IrqSafeSpinlock, task::process::Process,
};

#[derive(Clone, Copy, Debug)]
pub enum WaitStatus {
    Interrupted,
    Pending,
    Done,
}

pub struct Wait {
    queue: IrqSafeSpinlock<LinkedList<Rc<Process>>>,
    name: &'static str,
}

struct Timeout {
    process: Rc<Process>,
    deadline: Duration,
}

impl Wait {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            queue: IrqSafeSpinlock::new(LinkedList::new()),
        }
    }

    pub fn wait(&'static self, deadline: Option<Duration>) -> Result<(), Error> {
        let process = Process::current();
        let mut queue_lock = self.queue.lock();
        queue_lock.push_back(process.clone());
        process.setup_wait(self);

        if let Some(deadline) = deadline {
            TICK_LIST.lock().push_back(Timeout {
                process: process.clone(),
                deadline,
            });
        }

        loop {
            match process.wait_status() {
                WaitStatus::Pending => (),
                WaitStatus::Done => todo!(),
                WaitStatus::Interrupted => todo!(),
            }

            drop(queue_lock);
            process.suspend();

            queue_lock = self.queue.lock();

            if let Some(deadline) = deadline {
                let now = PLATFORM.timestamp_source().timestamp()?;

                if now > deadline {
                    let mut cursor = queue_lock.cursor_front_mut();

                    while let Some(item) = cursor.current() {
                        if item.id() == process.id() {
                            cursor.remove_current();
                            return Err(Error::TimedOut);
                        } else {
                            cursor.move_next();
                        }
                    }

                    panic!();
                }
            }
            todo!();
        }
    }
}

static TICK_LIST: IrqSafeSpinlock<LinkedList<Timeout>> = IrqSafeSpinlock::new(LinkedList::new());

pub fn sleep(timeout: Duration, remaining: &mut Duration) -> Result<(), Error> {
    static SLEEP_NOTIFY: Wait = Wait::new("sleep");
    let now = PLATFORM.timestamp_source().timestamp()?;
    let deadline = now + timeout;

    match SLEEP_NOTIFY.wait(Some(deadline)) {
        // Just what we expected
        Err(Error::TimedOut) => {
            *remaining = Duration::ZERO;
            Ok(())
        }

        Ok(_) => panic!("This should not happen"),
        Err(e) => Err(e),
    }
}

pub fn tick() {
    let now = PLATFORM.timestamp_source().timestamp().unwrap();
    let mut list = TICK_LIST.lock();
    let mut cursor = list.cursor_front_mut();

    while let Some(item) = cursor.current() {
        if now > item.deadline {
            let t = cursor.remove_current().unwrap();

            t.process.enqueue_somewhere();
        } else {
            cursor.move_next();
        }
    }
}
