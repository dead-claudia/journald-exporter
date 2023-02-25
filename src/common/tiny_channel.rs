// A small multi-producer, multi-consumer low-throughput channel optimized for simplicity. It's
// locking, but that's the point - it's super simple.

use crate::prelude::*;

use std::collections::VecDeque;

struct RefCount {
    value: AtomicU32,
}

impl fmt::Debug for RefCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // See Boost's explainer for why this uses `Ordering::Acquire`:
        // https://www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html
        self.value.load(Ordering::Acquire).fmt(f)
    }
}

impl RefCount {
    const fn new() -> Self {
        Self {
            value: AtomicU32::new(1),
        }
    }

    fn has_refs(&self) -> bool {
        // See Boost's explainer for why this uses `Ordering::Acquire`:
        // https://www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html
        self.value.load(Ordering::Acquire) != 0
    }

    fn inc(&self) {
        // See Boost's explainer for why this uses `Ordering::Relaxed`:
        // https://www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html
        let prev = self.value.fetch_add(1, Ordering::Relaxed);

        // Defend against potential leaks by panicking.
        const MAX_REF_COUNT: u32 = reinterpret_i32_u32(i32::MAX);

        // Abort on overflow. Should never happen in practice, and it's less code to generate.
        if prev >= MAX_REF_COUNT {
            std::process::abort();
        }
    }

    fn dec(&self) {
        // See Boost's explainer for why this uses `Ordering::Release` on the subtraction and
        // follows it with an `Acquire` fence:
        // https://www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html
        let prev = self.value.fetch_sub(1, Ordering::Release);

        if prev == 0 {
            std::sync::atomic::fence(Ordering::Acquire);
        }
    }
}

#[derive(Debug)]
struct State<T> {
    senders: RefCount,
    receivers: RefCount,
    checkpoint: Checkpoint<T>,
}

#[derive(Debug)]
pub struct ChannelSender<T> {
    inner: Arc<State<VecDeque<T>>>,
}

impl<T> Clone for ChannelSender<T> {
    fn clone(&self) -> Self {
        self.inner.senders.inc();
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Drop for ChannelSender<T> {
    fn drop(&mut self) {
        self.inner.senders.dec();
    }
}

#[derive(Debug)]
pub struct ChannelReceiver<T> {
    inner: Arc<State<VecDeque<T>>>,
}

impl<T> Clone for ChannelReceiver<T> {
    fn clone(&self) -> Self {
        self.inner.receivers.inc();
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Drop for ChannelReceiver<T> {
    fn drop(&mut self) {
        self.inner.receivers.dec();
    }
}

pub fn make_channel<T>() -> (ChannelSender<T>, ChannelReceiver<T>) {
    let state = Arc::new(State {
        senders: RefCount::new(),
        receivers: RefCount::new(),
        checkpoint: Checkpoint::new(VecDeque::new()),
    });

    (
        ChannelSender {
            inner: Arc::clone(&state),
        },
        ChannelReceiver { inner: state },
    )
}

#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub enum SendResult<T> {
    Sent,
    Disconnected(T),
}

impl<T> ChannelSender<T> {
    pub fn send(&self, value: T) -> SendResult<T> {
        // Hold the guard when notifying the receiver.
        self.inner.checkpoint.try_notify(|guard| {
            if self.inner.receivers.has_refs() {
                guard.push_back(value);
                (true, SendResult::Sent)
            } else {
                (false, SendResult::Disconnected(value))
            }
        })
    }
}

#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub enum ReadTimeoutResult<T> {
    Received(VecDeque<T>),
    TimedOut,
    Disconnected,
}

impl<T> ChannelReceiver<T> {
    /// This reads the whole internal buffer in a single go for simplicity
    pub fn read_timeout(&self, timeout: Duration) -> ReadTimeoutResult<T> {
        // Hold the guard when notifying the receiver.
        let mut guard = self.inner.checkpoint.lock();

        if !guard.is_empty() {
            return ReadTimeoutResult::Received(take(&mut *guard));
        }

        if self.inner.senders.has_refs() {
            let mut guard = self.inner.checkpoint.resume_wait_for(timeout, guard);

            if !guard.is_empty() {
                return ReadTimeoutResult::Received(take(&mut *guard));
            }

            if self.inner.senders.has_refs() {
                return ReadTimeoutResult::TimedOut;
            }
        }

        ReadTimeoutResult::Disconnected
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn works_with_sync_reader_single_message() {
        let (send, recv) = make_channel();
        assert_eq!(send.send(123), SendResult::Sent);
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Received(VecDeque::from_iter([123]))
        );
        assert_eq!(send.send(456), SendResult::Sent);
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Received(VecDeque::from_iter([456]))
        );
    }

    #[test]
    fn works_with_sync_reader_multi_message() {
        let (send, recv) = make_channel();
        assert_eq!(send.send(123), SendResult::Sent);
        assert_eq!(send.send(456), SendResult::Sent);
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Received(VecDeque::from_iter([123, 456]))
        );
        assert_eq!(send.send(555), SendResult::Sent);
        assert_eq!(send.send(222), SendResult::Sent);
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Received(VecDeque::from_iter([555, 222]))
        );
    }

    #[test]
    fn handles_sync_drop() {
        let (send, recv) = make_channel::<i32>();
        drop(send);
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Disconnected
        );
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Disconnected
        );
    }

    #[test]
    fn works_with_async_reader_single_message() {
        // Note: this joins instead of using another checkpoint so Miri can work correctly.
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static START_RESUME: ThreadCheckpoint = ThreadCheckpoint::new();
        static NEXT_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        let _guard = START_CHECKPOINT.drop_guard();
        let _guard = NEXT_CHECKPOINT.drop_guard();
        let (send, recv) = make_channel();
        let handle = ThreadHandle::spawn(Box::new(move || {
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(send.send(123), SendResult::Sent);
            START_RESUME.resume();
            if !NEXT_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(send.send(456), SendResult::Sent);
            Ok(())
        }));

        assert_eq!(
            recv.read_timeout(Duration::from_millis(250)),
            ReadTimeoutResult::TimedOut
        );
        START_CHECKPOINT.resume();
        if !START_RESUME.try_wait() {
            return;
        }
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Received(VecDeque::from_iter([123]))
        );
        NEXT_CHECKPOINT.resume();
        handle.join().unwrap();
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Received(VecDeque::from_iter([456]))
        );
        assert_eq!(
            recv.read_timeout(Duration::from_secs(5)),
            ReadTimeoutResult::Disconnected
        );
    }

    #[test]
    fn works_with_async_reader_multi_message() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static START_RESUME: ThreadCheckpoint = ThreadCheckpoint::new();
        static NEXT_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        let _guard = START_CHECKPOINT.drop_guard();
        let _guard = NEXT_CHECKPOINT.drop_guard();
        let (send, recv) = make_channel();
        let handle = ThreadHandle::spawn(Box::new(move || {
            let _guard = START_RESUME.drop_guard();
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(send.send(123), SendResult::Sent);
            assert_eq!(send.send(456), SendResult::Sent);
            START_RESUME.resume();
            if !NEXT_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(send.send(555), SendResult::Sent);
            assert_eq!(send.send(222), SendResult::Sent);
            Ok(())
        }));

        assert_eq!(
            recv.read_timeout(Duration::from_millis(250)),
            ReadTimeoutResult::TimedOut
        );
        START_CHECKPOINT.resume();
        if !START_RESUME.try_wait() {
            return;
        }
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Received(VecDeque::from_iter([123, 456]))
        );
        NEXT_CHECKPOINT.resume();
        handle.join().unwrap();
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Received(VecDeque::from_iter([555, 222]))
        );
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Disconnected
        );
    }

    #[test]
    fn handles_async_drop() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        let _guard = START_CHECKPOINT.drop_guard();
        let (send, recv) = make_channel::<i32>();
        let handle = ThreadHandle::spawn(Box::new(move || {
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            drop(send);
            Ok(())
        }));

        assert_eq!(
            recv.read_timeout(Duration::from_millis(250)),
            ReadTimeoutResult::TimedOut
        );
        START_CHECKPOINT.resume();
        assert!(matches!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::TimedOut | ReadTimeoutResult::Disconnected
        ));
        handle.join().unwrap();
        assert_eq!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::Disconnected
        );
    }

    #[test]
    fn works_with_concurrent_reader_single_message() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        let _guard = START_CHECKPOINT.drop_guard();
        let (send, recv) = make_channel();
        let handle = ThreadHandle::spawn(Box::new(move || {
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(send.send(123), SendResult::Sent);
            Ok(())
        }));

        assert_eq!(
            recv.read_timeout(Duration::from_millis(250)),
            ReadTimeoutResult::TimedOut
        );
        START_CHECKPOINT.resume();
        loop {
            match recv.read_timeout(Duration::from_secs(1)) {
                ReadTimeoutResult::Received(v) => {
                    assert_eq!(v, VecDeque::from_iter([123]));
                    break;
                }
                ReadTimeoutResult::TimedOut => {}
                ReadTimeoutResult::Disconnected => {
                    handle.join().unwrap();
                    panic!("Thread joined without emitting data!");
                }
            }
        }
        assert!(matches!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::TimedOut | ReadTimeoutResult::Disconnected
        ));
    }

    #[test]
    fn works_with_concurrent_reader_multi_message() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static START_RESUME: ThreadCheckpoint = ThreadCheckpoint::new();
        static NEXT_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        let _guard = START_CHECKPOINT.drop_guard();
        let (send, recv) = make_channel();
        let handle = ThreadHandle::spawn(Box::new(move || {
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(send.send(123), SendResult::Sent);
            assert_eq!(send.send(456), SendResult::Sent);
            START_RESUME.resume();
            if !NEXT_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(send.send(555), SendResult::Sent);
            assert_eq!(send.send(222), SendResult::Sent);
            Ok(())
        }));

        assert_eq!(
            recv.read_timeout(Duration::from_millis(250)),
            ReadTimeoutResult::TimedOut
        );
        START_CHECKPOINT.resume();
        if !START_RESUME.try_wait() {
            return;
        }
        let mut read_first = false;
        loop {
            match recv.read_timeout(Duration::from_secs(1)) {
                ReadTimeoutResult::Received(v) if v.len() > 1 => {
                    assert_eq!(v, VecDeque::from_iter([123, 456]));
                    break;
                }
                ReadTimeoutResult::Received(v) if read_first => {
                    assert_eq!(v, VecDeque::from_iter([456]));
                    break;
                }
                ReadTimeoutResult::Received(v) => {
                    assert_eq!(v, VecDeque::from_iter([123]));
                    read_first = true;
                }
                ReadTimeoutResult::TimedOut => {}
                ReadTimeoutResult::Disconnected => {
                    handle.join().unwrap();
                    panic!("Thread joined without emitting data!");
                }
            }
        }
        NEXT_CHECKPOINT.resume();
        let mut read_first = false;
        loop {
            match recv.read_timeout(Duration::from_secs(1)) {
                ReadTimeoutResult::Received(v) if v.len() > 1 => {
                    assert_eq!(v, VecDeque::from_iter([555, 222]));
                    break;
                }
                ReadTimeoutResult::Received(v) if read_first => {
                    assert_eq!(v, VecDeque::from_iter([222]));
                    break;
                }
                ReadTimeoutResult::Received(v) => {
                    assert_eq!(v, VecDeque::from_iter([555]));
                    read_first = true;
                }
                ReadTimeoutResult::TimedOut => {}
                ReadTimeoutResult::Disconnected => {
                    handle.join().unwrap();
                    panic!("Thread joined without emitting data!");
                }
            }
        }
        assert!(matches!(
            recv.read_timeout(Duration::from_secs(1)),
            ReadTimeoutResult::TimedOut | ReadTimeoutResult::Disconnected
        ));
    }

    #[test]
    fn handles_concurrent_drop() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        let _guard = START_CHECKPOINT.drop_guard();
        let (send, recv) = make_channel::<i32>();
        let handle = ThreadHandle::spawn(Box::new(move || {
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            drop(send);
            Ok(())
        }));

        assert_eq!(
            recv.read_timeout(Duration::from_millis(250)),
            ReadTimeoutResult::TimedOut
        );
        START_CHECKPOINT.resume();
        loop {
            match recv.read_timeout(Duration::from_secs(1)) {
                ReadTimeoutResult::Received(v) => {
                    panic!("Unexpected data received: {v:?}");
                }
                ReadTimeoutResult::TimedOut => {}
                ReadTimeoutResult::Disconnected => {
                    handle.join().unwrap();
                    break;
                }
            }
        }
    }
}
