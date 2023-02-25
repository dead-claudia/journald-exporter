use crate::prelude::*;

#[derive(Debug)]
pub struct Notify {
    notified: AtomicBool,
}

impl Notify {
    pub const fn new() -> Notify {
        Notify {
            notified: AtomicBool::new(false),
        }
    }

    pub fn reset(&self) {
        self.notified.store(false, Ordering::Release);
    }

    pub fn has_notified(&self) -> bool {
        self.notified.load(Ordering::Acquire)
    }

    pub fn create_guard(&self) -> NotifyGuard {
        NotifyGuard(self)
    }

    pub fn notify(&self) {
        self.notified.store(true, Ordering::Release);
    }
}

pub struct NotifyGuard<'a>(&'a Notify);

impl Drop for NotifyGuard<'_> {
    fn drop(&mut self) {
        self.0.notify();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn immediate_notify() {
        static NOTIFY: Notify = Notify::new();

        assert!(!NOTIFY.has_notified());
        NOTIFY.notify();
        assert!(NOTIFY.has_notified());
    }

    #[test]
    fn transition_to_notify() {
        static NOTIFY: Notify = Notify::new();

        let handle = ThreadHandle::spawn(Box::new(|| {
            std::thread::sleep(Duration::from_millis(50));
            NOTIFY.notify();
            Ok(())
        }));

        assert!(!NOTIFY.has_notified());
        handle.join().unwrap();
        assert!(NOTIFY.has_notified());
    }
}
