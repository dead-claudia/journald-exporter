// A small multi-producer, multi-consumer low-throughput channel optimized for simplicity. It's
// locking, but that's the point - it's super simple.

use crate::prelude::*;

#[derive(Debug)]
pub struct Channel<T, const N: usize> {
    closed: Notify,
    checkpoint: Checkpoint<Option<Box<heapless::Vec<T, N>>>>,
}

pub struct ChannelCloseGuard<'a, T, const N: usize> {
    channel: &'a Channel<T, N>,
}

impl<T, const N: usize> Drop for ChannelCloseGuard<'_, T, N> {
    fn drop(&mut self) {
        self.channel.close();
    }
}

impl<T, const N: usize> Channel<T, N> {
    pub const fn new() -> Self {
        Self {
            closed: Notify::new(),
            checkpoint: Checkpoint::new(None),
        }
    }

    pub fn close_notify(&self) -> &Notify {
        &self.closed
    }

    pub fn close(&self) {
        // Don't notify if it's already closed.
        self.checkpoint.try_notify(|guard| {
            *guard = None;
            (self.closed.notify(), ())
        });
    }

    pub fn has_closed(&self) -> bool {
        self.closed.has_notified()
    }

    pub fn close_guard(&self) -> ChannelCloseGuard<T, N> {
        ChannelCloseGuard { channel: self }
    }

    pub fn send(&self, value: T) -> Result<(), T> {
        self.checkpoint.notify(|guard| match guard {
            Some(vec) => vec.push(value),
            None => match try_new_fixed_vec() {
                Some(vec) => guard.insert(vec).push(value),
                None => Err(value),
            },
        })
    }

    /// This reads the whole internal buffer in a single go for simplicity
    pub fn read_timeout(&self, timeout: Duration) -> Option<Box<heapless::Vec<T, N>>> {
        let mut guard = self.checkpoint.lock();

        guard.take().or_else(|| {
            if self.has_closed() {
                None
            } else {
                self.checkpoint.resume_wait_for(timeout, guard).take()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CAPACITY: usize = 16;
    type TestChannel = Channel<usize, TEST_CAPACITY>;
    type TestVec = heapless::Vec<usize, TEST_CAPACITY>;

    #[track_caller]
    fn receive_concurrent(channel: &TestChannel, joined: &Notify) -> Option<Box<TestVec>> {
        loop {
            match channel.read_timeout(Duration::from_millis(10)) {
                Some(v) => break Some(v),
                None if !joined.has_notified() => {}
                None => break None,
            }
        }
    }

    #[test]
    fn works_with_sync_reader_single_message() {
        static CHANNEL: TestChannel = Channel::new();

        let _guard = CHANNEL.close_guard();

        assert_eq!(CHANNEL.send(123), Ok(()));

        assert_eq!(
            CHANNEL.read_timeout(Duration::ZERO),
            Some(Box::new(TestVec::from_iter([123])))
        );

        assert_eq!(CHANNEL.send(456), Ok(()));
        assert_eq!(
            CHANNEL.read_timeout(Duration::ZERO),
            Some(Box::new(TestVec::from_iter([456])))
        );
    }

    #[test]
    fn works_with_sync_reader_multi_message() {
        static CHANNEL: TestChannel = Channel::new();

        let _guard = CHANNEL.close_guard();

        assert_eq!(CHANNEL.send(123), Ok(()));
        assert_eq!(CHANNEL.send(456), Ok(()));

        assert_eq!(
            CHANNEL.read_timeout(Duration::ZERO),
            Some(Box::new(TestVec::from_iter([123, 456])))
        );

        assert_eq!(CHANNEL.send(555), Ok(()));
        assert_eq!(CHANNEL.send(222), Ok(()));
        assert_eq!(
            CHANNEL.read_timeout(Duration::ZERO),
            Some(Box::new(TestVec::from_iter([555, 222])))
        );
    }

    #[test]
    fn works_with_sync_reader_max_messages_read_before_close() {
        static CHANNEL: TestChannel = Channel::new();

        let _guard = CHANNEL.close_guard();

        for i in 1..=TEST_CAPACITY {
            assert_eq!(CHANNEL.send(i), Ok(()));
        }
        assert_eq!(CHANNEL.send(555), Err(555));

        assert_eq!(
            CHANNEL.read_timeout(Duration::ZERO),
            Some(Box::new(TestVec::from_iter(1..=TEST_CAPACITY)))
        );

        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);

        CHANNEL.close();

        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    #[test]
    fn works_with_sync_reader_max_messages_read_after_close() {
        static CHANNEL: TestChannel = Channel::new();

        let _guard = CHANNEL.close_guard();

        for i in 1..=TEST_CAPACITY {
            assert_eq!(CHANNEL.send(i), Ok(()));
        }
        assert_eq!(CHANNEL.send(555), Err(555));

        CHANNEL.close();

        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    #[test]
    fn handles_sync_drop() {
        static CHANNEL: TestChannel = Channel::new();

        let guard = CHANNEL.close_guard();

        assert!(!CHANNEL.has_closed());

        drop(guard);

        assert!(CHANNEL.has_closed());
        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    #[test]
    fn works_with_async_reader_single_message() {
        // Note: this joins instead of using another checkpoint so Miri can work correctly.
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static START_RESUME: ThreadCheckpoint = ThreadCheckpoint::new();
        static NEXT_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static CHANNEL: TestChannel = Channel::new();

        let handle = ThreadHandle::spawn(Box::new(move || {
            let guard = CHANNEL.close_guard();
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(CHANNEL.send(123), Ok(()));
            START_RESUME.resume();
            if !NEXT_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(CHANNEL.send(456), Ok(()));
            #[allow(clippy::mem_forget)]
            std::mem::forget(guard);
            Ok(())
        }));

        let _guard = START_CHECKPOINT.drop_guard();
        let _guard = NEXT_CHECKPOINT.drop_guard();
        let _guard = CHANNEL.close_guard();

        assert_eq!(CHANNEL.read_timeout(Duration::from_millis(250)), None);
        START_CHECKPOINT.resume();
        if !START_RESUME.try_wait() {
            return;
        }

        assert_eq!(
            CHANNEL.read_timeout(Duration::from_secs(1)),
            Some(Box::new(TestVec::from_iter([123])))
        );

        NEXT_CHECKPOINT.resume();

        handle.join().unwrap();

        assert_eq!(
            CHANNEL.read_timeout(Duration::ZERO),
            Some(Box::new(TestVec::from_iter([456])))
        );
        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    #[test]
    fn works_with_async_reader_multi_message() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static START_RESUME: ThreadCheckpoint = ThreadCheckpoint::new();
        static NEXT_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static CHANNEL: TestChannel = Channel::new();

        let handle = ThreadHandle::spawn(Box::new(move || {
            let guard = CHANNEL.close_guard();
            let _guard = START_RESUME.drop_guard();
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(CHANNEL.send(123), Ok(()));
            assert_eq!(CHANNEL.send(456), Ok(()));
            START_RESUME.resume();
            if !NEXT_CHECKPOINT.try_wait() {
                return Ok(());
            }
            assert_eq!(CHANNEL.send(555), Ok(()));
            assert_eq!(CHANNEL.send(222), Ok(()));
            #[allow(clippy::mem_forget)]
            std::mem::forget(guard);
            Ok(())
        }));

        let _guard = START_CHECKPOINT.drop_guard();
        let _guard = NEXT_CHECKPOINT.drop_guard();
        let _guard = CHANNEL.close_guard();

        assert_eq!(CHANNEL.read_timeout(Duration::from_millis(250)), None);

        START_CHECKPOINT.resume();
        if !START_RESUME.try_wait() {
            return;
        }

        assert_eq!(
            CHANNEL.read_timeout(Duration::from_secs(1)),
            Some(Box::new(TestVec::from_iter([123, 456])))
        );

        NEXT_CHECKPOINT.resume();

        handle.join().unwrap();

        assert_eq!(
            CHANNEL.read_timeout(Duration::ZERO),
            Some(Box::new(TestVec::from_iter([555, 222])))
        );
        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    #[test]
    fn works_with_async_reader_max_messages_read_before_close() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static START_RESUME: ThreadCheckpoint = ThreadCheckpoint::new();
        static NEXT_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static CHANNEL: TestChannel = Channel::new();

        let handle = ThreadHandle::spawn(Box::new(move || {
            let channel_guard = CHANNEL.close_guard();
            let resume_guard = START_RESUME.drop_guard();
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            for i in 1..=TEST_CAPACITY {
                assert_eq!(CHANNEL.send(i), Ok(()));
            }
            assert_eq!(CHANNEL.send(555), Err(555));
            drop(resume_guard);
            if !NEXT_CHECKPOINT.try_wait() {
                return Ok(());
            }
            drop(channel_guard);
            Ok(())
        }));

        let _guard = START_CHECKPOINT.drop_guard();
        let _guard = NEXT_CHECKPOINT.drop_guard();
        let _guard = CHANNEL.close_guard();

        assert_eq!(CHANNEL.read_timeout(Duration::from_millis(250)), None);

        START_CHECKPOINT.resume();
        if !START_RESUME.try_wait() {
            return;
        }

        assert_eq!(
            CHANNEL.read_timeout(Duration::from_secs(1)),
            Some(Box::new(TestVec::from_iter(1..=TEST_CAPACITY)))
        );

        NEXT_CHECKPOINT.resume();

        handle.join().unwrap();

        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    #[test]
    fn works_with_async_reader_max_messages_read_after_close() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static CHANNEL: TestChannel = Channel::new();

        let handle = ThreadHandle::spawn(Box::new(move || {
            let _guard = CHANNEL.close_guard();
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            for i in 1..=TEST_CAPACITY {
                assert_eq!(CHANNEL.send(i), Ok(()));
            }
            assert_eq!(CHANNEL.send(555), Err(555));
            Ok(())
        }));

        let _guard = START_CHECKPOINT.drop_guard();
        let _guard = CHANNEL.close_guard();

        assert_eq!(CHANNEL.read_timeout(Duration::from_millis(250)), None);

        START_CHECKPOINT.resume();

        handle.join().unwrap();

        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    #[test]
    fn handles_async_drop() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static CHANNEL: TestChannel = Channel::new();

        let handle = ThreadHandle::spawn(Box::new(move || {
            let channel_guard = CHANNEL.close_guard();
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            drop(channel_guard);
            Ok(())
        }));

        let _guard = START_CHECKPOINT.drop_guard();
        let _guard = CHANNEL.close_guard();

        assert_eq!(CHANNEL.read_timeout(Duration::from_millis(250)), None);
        assert!(!CHANNEL.has_closed());

        START_CHECKPOINT.resume();

        assert_eq!(CHANNEL.read_timeout(Duration::from_secs(1)), None);

        handle.join().unwrap();

        assert!(CHANNEL.has_closed());
        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    #[test]
    fn works_with_concurrent_reader_single_message() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static CHANNEL: TestChannel = Channel::new();
        static JOINED: Notify = Notify::new();

        let handle = ThreadHandle::spawn(Box::new(move || {
            let _guard = JOINED.create_guard();
            let _guard = CHANNEL.close_guard();
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }

            // Sleep 100 milliseconds to give `receive_concurrent` time to spin up.
            std::thread::sleep(Duration::from_millis(100));
            assert_eq!(CHANNEL.send(123), Ok(()));
            // Sleep 100 milliseconds to give the final assertion time to run before joining.
            std::thread::sleep(Duration::from_millis(100));
            Ok(())
        }));

        let _guard = START_CHECKPOINT.drop_guard();
        let _guard = CHANNEL.close_guard();

        assert_eq!(CHANNEL.read_timeout(Duration::from_millis(250)), None);
        assert!(!CHANNEL.has_closed());

        START_CHECKPOINT.resume();

        let v =
            receive_concurrent(&CHANNEL, &JOINED).expect("Thread joined without emitting data!");

        assert_eq!(v, Box::new(TestVec::from_iter([123])));

        handle.join().unwrap();

        assert!(CHANNEL.has_closed());
        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    #[test]
    fn works_with_concurrent_reader_multi_message() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static START_RESUME: ThreadCheckpoint = ThreadCheckpoint::new();
        static NEXT_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static CHANNEL: TestChannel = Channel::new();
        static JOINED: Notify = Notify::new();

        let handle = ThreadHandle::spawn(Box::new(move || {
            let _guard = JOINED.create_guard();
            let _guard = CHANNEL.close_guard();
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            // Sleep 100 milliseconds to give `receive_concurrent` time to spin up.
            std::thread::sleep(Duration::from_millis(100));
            assert_eq!(CHANNEL.send(123), Ok(()));
            assert_eq!(CHANNEL.send(456), Ok(()));
            START_RESUME.resume();
            if !NEXT_CHECKPOINT.try_wait() {
                return Ok(());
            }
            // Sleep 100 milliseconds to give `receive_concurrent` time to spin up.
            std::thread::sleep(Duration::from_millis(100));
            assert_eq!(CHANNEL.send(555), Ok(()));
            assert_eq!(CHANNEL.send(222), Ok(()));
            // Sleep 100 milliseconds to give the final assertion time to run before joining.
            std::thread::sleep(Duration::from_millis(100));
            Ok(())
        }));

        let _guard = START_CHECKPOINT.drop_guard();
        let _guard = NEXT_CHECKPOINT.drop_guard();
        let _guard = CHANNEL.close_guard();

        assert_eq!(CHANNEL.read_timeout(Duration::from_millis(250)), None);
        assert!(!CHANNEL.has_closed());

        START_CHECKPOINT.resume();
        if !START_RESUME.try_wait() {
            return;
        }

        #[track_caller]
        fn check_received_pair(first: usize, second: usize) -> bool {
            let v = match receive_concurrent(&CHANNEL, &JOINED) {
                Some(v) => v,
                None => return false,
            };

            if v.len() > 1 {
                assert_eq!(v, Box::new(TestVec::from_iter([first, second])));
            } else {
                assert_eq!(v, Box::new(TestVec::from_iter([first])));
                let v = match receive_concurrent(&CHANNEL, &JOINED) {
                    Some(v) => v,
                    None => return false,
                };
                assert_eq!(v, Box::new(TestVec::from_iter([second])));
            }

            true
        }

        if !check_received_pair(123, 456) {
            handle.join().unwrap();
            panic!("Thread joined without emitting data!");
        }
        NEXT_CHECKPOINT.resume();
        if !check_received_pair(555, 222) {
            handle.join().unwrap();
            panic!("Thread joined without emitting data!");
        }

        assert_eq!(CHANNEL.read_timeout(Duration::from_secs(1)), None);

        handle.join().unwrap();

        assert!(CHANNEL.has_closed());
        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn works_with_concurrent_reader_many_messages() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static NEXT_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static NEXT_RESUME: ThreadCheckpoint = ThreadCheckpoint::new();
        static CHANNEL: TestChannel = Channel::new();

        const MESSAGE_COUNT: usize = 10000;

        let handle = ThreadHandle::spawn(Box::new(move || {
            let _guard = CHANNEL.close_guard();
            let _guard = NEXT_RESUME.drop_guard();

            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }

            // Sleep 100 milliseconds to give the reader loop time to spin up.
            std::thread::sleep(Duration::from_millis(100));

            'pusher: for i in 1..=MESSAGE_COUNT {
                loop {
                    let result = CHANNEL.send(i);
                    if result.is_ok() {
                        break;
                    }
                    assert_eq!(result, Err(i));
                    if CHANNEL.has_closed() {
                        break 'pusher;
                    }
                }
            }

            NEXT_RESUME.resume();
            if !NEXT_CHECKPOINT.try_wait() {
                return Ok(());
            }

            Ok(())
        }));

        let _guard = START_CHECKPOINT.drop_guard();
        let _guard = NEXT_CHECKPOINT.drop_guard();
        let _guard = CHANNEL.close_guard();

        static RESULT: Uncontended<heapless::Vec<usize, MESSAGE_COUNT>> =
            Uncontended::new(heapless::Vec::new());

        static EXPECTED: [usize; MESSAGE_COUNT] = {
            let mut result = [0; MESSAGE_COUNT];
            let mut i = 0;
            while i < result.len() {
                result[i] = i + 1;
                i += 1;
            }
            result
        };

        let mut result = RESULT.lock();

        assert_eq!(CHANNEL.read_timeout(Duration::from_millis(250)), None);
        assert!(!CHANNEL.has_closed());

        START_CHECKPOINT.resume();

        #[track_caller]
        fn push(result: &mut heapless::Vec<usize, MESSAGE_COUNT>, source: &[usize]) {
            if result.extend_from_slice(source).is_err() {
                panic!(
                    "Excess data detected. Capacity: {}, length: {}, pushed: {:?}",
                    MESSAGE_COUNT,
                    result.len(),
                    source
                )
            }
        }

        while result.len() < MESSAGE_COUNT {
            match CHANNEL.read_timeout(Duration::from_millis(10)) {
                Some(v) => push(&mut result, &v),
                None if CHANNEL.has_closed() => break,
                None => {}
            }
        }

        if !NEXT_RESUME.try_wait() {
            return;
        }

        if let Some(v) = CHANNEL.read_timeout(Duration::ZERO) {
            push(&mut result, &v);
        }

        assert_eq!(result.len(), MESSAGE_COUNT);
        assert_eq!(&result[..], &EXPECTED);
        assert_eq!(CHANNEL.read_timeout(Duration::from_secs(1)), None);
        NEXT_CHECKPOINT.resume();

        handle.join().unwrap();

        assert!(CHANNEL.has_closed());
        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }

    // No specific "max messages" test exists for concurrent readers, since it'd be a challenge to
    // get a queue depth that deep without serious flakiness. Also, it'd be a massive pain to test.
    // In any case, I expect Miri would be able to figure out how to get that failure mode at least
    // sometimes tested for in the above test.

    #[test]
    fn handles_concurrent_drop() {
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static NEXT_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static CHANNEL: TestChannel = Channel::new();
        static JOINED: Notify = Notify::new();

        let _handle = ThreadHandle::spawn(Box::new(move || {
            let _guard = JOINED.create_guard();
            let channel_guard = CHANNEL.close_guard();
            if !START_CHECKPOINT.try_wait() {
                return Ok(());
            }
            // Sleep 100 milliseconds to give the below loop time to spin up.
            std::thread::sleep(Duration::from_millis(100));
            drop(channel_guard);
            if !NEXT_CHECKPOINT.try_wait() {
                return Ok(());
            }
            // Sleep 100 milliseconds to let the below loop detect closure.
            std::thread::sleep(Duration::from_millis(100));
            Ok(())
        }));

        let _guard = CHANNEL.close_guard();

        let mut is_closed = false;

        assert!(!CHANNEL.has_closed());

        START_CHECKPOINT.resume();

        loop {
            match CHANNEL.read_timeout(Duration::from_millis(10)) {
                Some(v) => panic!("Unexpected data received: {v:?}"),
                None if JOINED.has_notified() => break,
                None => {
                    if is_closed {
                        assert!(CHANNEL.has_closed());
                    } else {
                        is_closed = CHANNEL.has_closed();
                        NEXT_CHECKPOINT.resume();
                    }
                }
            }
        }

        assert_eq!(CHANNEL.read_timeout(Duration::ZERO), None);
    }
}
