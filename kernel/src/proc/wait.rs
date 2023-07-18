//! Wait channel implementation
use core::time::Duration;

use abi::error::Error;
use alloc::{collections::LinkedList, rc::Rc};

use crate::{
    arch::PLATFORM, device::platform::Platform, sync::IrqSafeSpinlock, task::process::Process,
};

/// Defines whether the wait channel is available for a specific task
#[derive(Clone, Copy, Debug)]
pub enum WaitStatus {
    /// Wait on the channel was interrupted
    Interrupted,
    /// Channel did not yet signal availability
    Pending,
    /// Channel has data available
    Done,
}

/// Wait notification channel
pub struct Wait {
    queue: IrqSafeSpinlock<LinkedList<Rc<Process>>>,
    // Used for tracing waits
    #[allow(dead_code)]
    name: &'static str,
}

struct Timeout {
    process: Rc<Process>,
    deadline: Duration,
}

impl Wait {
    /// Constructs a new wait notification channel
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            queue: IrqSafeSpinlock::new(LinkedList::new()),
        }
    }

    /// Wakes up tasks waiting for availability on this channel, but no more than `limit`
    pub fn wakeup_some(&self, mut limit: usize) -> usize {
        let mut queue = self.queue.lock();
        let mut count = 0;
        while limit != 0 && !queue.is_empty() {
            let proc = queue.pop_front().unwrap();

            {
                let mut tick_lock = TICK_LIST.lock();
                let mut cursor = tick_lock.cursor_front_mut();

                while let Some(item) = cursor.current() {
                    if proc.id() == item.process.id() {
                        cursor.remove_current();
                        break;
                    } else {
                        cursor.move_next();
                    }
                }

                drop(tick_lock);

                unsafe {
                    proc.set_wait_status(WaitStatus::Done);
                }
                proc.enqueue_somewhere();
            }

            limit -= 1;
            count += 1;
        }

        count
    }

    /// Wakes up all tasks waiting on this channel
    pub fn wakeup_all(&self) {
        self.wakeup_some(usize::MAX);
    }

    /// Wakes up a single task waiting on this channel
    pub fn wakeup_one(&self) {
        self.wakeup_some(1);
    }

    /// Suspends the task until either the deadline is reached or this channel signals availability
    pub fn wait(&'static self, deadline: Option<Duration>) -> Result<(), Error> {
        let process = Process::current();
        let mut queue_lock = self.queue.lock();
        queue_lock.push_back(process.clone());
        unsafe {
            process.setup_wait(self);
        }

        if let Some(deadline) = deadline {
            TICK_LIST.lock().push_back(Timeout {
                process: process.clone(),
                deadline,
            });
        }

        loop {
            match process.wait_status() {
                WaitStatus::Pending => (),
                WaitStatus::Done => return Ok(()),
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
        }
    }
}

static TICK_LIST: IrqSafeSpinlock<LinkedList<Timeout>> = IrqSafeSpinlock::new(LinkedList::new());

/// Suspends current task until given deadline
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

/// Updates all pending timeouts and wakes up the tasks that have reached theirs
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
