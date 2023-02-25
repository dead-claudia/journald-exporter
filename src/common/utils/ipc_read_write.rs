use crate::prelude::*;

use crate::ffi::PollFlags;
use crate::ffi::PollResult;
use crate::ffi::Pollable;

#[must_use]
pub enum ReadWriteResult<T> {
    Success(T),
    Terminated,
    Err(Error),
}

pub fn try_read<'a>(
    mut input: impl Pollable + Read,
    done_notify: &Notify,
    read_buf: &'a mut [u8],
) -> ReadWriteResult<&'a [u8]> {
    let mut should_read = true;

    'outer: while !done_notify.has_notified() {
        let e = {
            if should_read {
                match input.read(read_buf) {
                    Ok(0) => continue 'outer,
                    Ok(bytes_read) => return ReadWriteResult::Success(&read_buf[..bytes_read]),
                    Err(e) => e,
                }
            } else {
                match input.poll(PollFlags::IN, Some(Duration::from_secs(1))) {
                    Ok(result) => {
                        should_read = result.intersects(PollResult::IN | PollResult::ERR);
                        continue 'outer;
                    }
                    Err(e) => e,
                }
            }
        };

        match e.kind() {
            // Retry these immediately.
            ErrorKind::Interrupted => {}
            ErrorKind::TimedOut => {}
            // Retry this after a poll
            ErrorKind::WouldBlock => should_read = false,
            _ => return ReadWriteResult::Err(e),
        }
    }

    ReadWriteResult::Terminated
}

pub fn try_read2<'a>(
    mut input: impl Pollable + Read,
    done_notify: &Notify,
    read_buf: &'a mut [u8],
) -> ReadWriteResult<&'a [u8]> {
    let mut should_read = true;

    'outer: while !done_notify.has_notified() {
        let e = {
            if should_read {
                match input.read(read_buf) {
                    Ok(0) => continue 'outer,
                    Ok(bytes_read) => return ReadWriteResult::Success(&read_buf[..bytes_read]),
                    Err(e) => e,
                }
            } else {
                match input.poll(PollFlags::IN, Some(Duration::from_secs(1))) {
                    Ok(result) => {
                        should_read = result.intersects(PollResult::IN | PollResult::ERR);
                        continue 'outer;
                    }
                    Err(e) => e,
                }
            }
        };

        match e.kind() {
            // Retry these immediately.
            ErrorKind::Interrupted => {}
            ErrorKind::TimedOut => {}
            // Retry this after a poll
            ErrorKind::WouldBlock => should_read = false,
            _ => return ReadWriteResult::Err(e),
        }
    }

    ReadWriteResult::Terminated
}

#[must_use]
pub enum WriteOutputRequestResult {
    Written,
    Terminated,
    Err(Error),
}

pub fn try_write(
    terminate_notify: &Notify,
    mut output: impl Pollable + Write,
    mut buf: &[u8],
) -> WriteOutputRequestResult {
    while !buf.is_empty() {
        if terminate_notify.has_notified() {
            return WriteOutputRequestResult::Terminated;
        }

        match output.write(buf) {
            Ok(len) => {
                buf = &buf[len..];
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => {
                // Just retry
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => 'poll: loop {
                if terminate_notify.has_notified() {
                    return WriteOutputRequestResult::Terminated;
                }

                // Poll and retry
                match output.poll(PollFlags::OUT, Some(Duration::from_secs(1))) {
                    Ok(_) => break 'poll,
                    // Tolerate and retry on these.
                    Err(e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) => return WriteOutputRequestResult::Err(e),
                };
            },
            Err(e) => return WriteOutputRequestResult::Err(e),
        }
    }

    match output.flush() {
        Ok(()) => WriteOutputRequestResult::Written,
        Err(e) => WriteOutputRequestResult::Err(e),
    }
}
