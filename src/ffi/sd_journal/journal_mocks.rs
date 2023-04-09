use crate::prelude::*;

use super::SystemdMonotonicUsec;
use crate::ffi::Cursor;
use crate::ffi::Id128;
use crate::ffi::JournalRef;
use crate::ffi::SystemdProvider;
#[cfg(not(miri))]
use const_str::cstr;
use std::ffi::CStr;

pub struct FakeJournalRef {
    pub set_data_threshold: CallSpy<usize, io::Result<()>>,
    pub seek_monotonic_usec: CallSpy<(Id128, u64), io::Result<()>>,
    pub seek_cursor: CallSpy<Cursor, io::Result<()>>,
    pub wait: CallSpy<Duration, io::Result<bool>>,
    pub next: CallSpy<(), io::Result<bool>>,
    pub cursor: CallSpy<(), io::Result<Cursor>>,
    pub get_data: CallSpyMap<FixedCString, (), io::Result<&'static [u8]>>,
}

impl FakeJournalRef {
    pub const fn new() -> FakeJournalRef {
        FakeJournalRef {
            set_data_threshold: CallSpy::new("set_data_threshold"),
            seek_monotonic_usec: CallSpy::new("seek_monotonic_usec"),
            seek_cursor: CallSpy::new("seek_cursor"),
            wait: CallSpy::new("wait"),
            next: CallSpy::new("next"),
            cursor: CallSpy::new("cursor"),
            get_data: CallSpyMap::new("get_data"),
        }
    }

    pub fn assert_no_calls_remaining(&self) {
        self.set_data_threshold.assert_no_calls_remaining();
        self.seek_monotonic_usec.assert_no_calls_remaining();
        self.seek_cursor.assert_no_calls_remaining();
        self.wait.assert_no_calls_remaining();
        self.next.assert_no_calls_remaining();
        self.cursor.assert_no_calls_remaining();
        self.get_data.assert_no_calls_remaining();
    }
}

impl JournalRef for &'static FakeJournalRef {
    type Provider = FakeSystemdProvider;

    fn open(provider: &'static Self::Provider) -> io::Result<Self> {
        provider.open.call(())?;
        Ok(&provider.journal)
    }

    fn set_data_threshold(&mut self, threshold: usize) -> io::Result<()> {
        self.set_data_threshold.call(threshold)
    }

    fn seek_monotonic_usec(
        &mut self,
        boot_id: &Id128,
        start_usec: SystemdMonotonicUsec,
    ) -> io::Result<()> {
        self.seek_monotonic_usec
            .call((Id128(boot_id.0), start_usec.0))
    }

    fn seek_cursor(&mut self, cursor: &Cursor) -> io::Result<()> {
        self.seek_cursor.call(cursor.clone())
    }

    fn wait(&mut self, duration: Duration) -> io::Result<bool> {
        self.wait.call(duration)
    }

    fn next(&mut self) -> io::Result<bool> {
        self.next.call(())
    }

    fn cursor(&mut self) -> io::Result<Cursor> {
        self.cursor.call(())
    }

    fn get_data<'a>(&'a mut self, field: &CStr) -> io::Result<&'a [u8]> {
        self.get_data.call(FixedCString::new(field.to_bytes()), ())
    }
}

pub struct FakeSystemdProvider {
    pub boot_id: Id128,
    pub open: CallSpy<(), io::Result<()>>,
    pub watchdog_notify: CallSpy<(), io::Result<()>>,
    pub get_monotonic_time_usec: CallSpy<(), u64>,
    pub journal: FakeJournalRef,
}

impl FakeSystemdProvider {
    pub const fn new(boot_id: Id128) -> FakeSystemdProvider {
        FakeSystemdProvider {
            boot_id,
            open: CallSpy::new("open"),
            watchdog_notify: CallSpy::new("watchdog_notify"),
            get_monotonic_time_usec: CallSpy::new("get_monotonic_time_usec"),
            journal: FakeJournalRef::new(),
        }
    }

    pub fn assert_no_calls_remaining(&self) {
        self.open.assert_no_calls_remaining();
        self.watchdog_notify.assert_no_calls_remaining();
        self.get_monotonic_time_usec.assert_no_calls_remaining();
        self.journal.assert_no_calls_remaining();
    }
}

impl SystemdProvider for FakeSystemdProvider
where
    Self: 'static,
{
    fn watchdog_notify(&'static self) -> io::Result<()> {
        self.watchdog_notify.call(())
    }

    fn boot_id(&'static self) -> &Id128 {
        &self.boot_id
    }

    fn get_monotonic_time_usec(&'static self) -> SystemdMonotonicUsec {
        SystemdMonotonicUsec(self.get_monotonic_time_usec.call(()))
    }
}

// Skip in Miri as it's just testing test mocks
#[cfg(not(miri))]
mod tests {
    use super::*;

    #[test]
    fn fake_systemd_provider_has_correct_properties() {
        static PROVIDER1: FakeSystemdProvider = FakeSystemdProvider::new(Id128(123));
        assert_eq!(PROVIDER1.boot_id(), &Id128(123));

        static PROVIDER2: FakeSystemdProvider = FakeSystemdProvider::new(Id128(321));
        assert_eq!(PROVIDER2.boot_id(), &Id128(321));
    }

    #[test]
    fn fake_systemd_provider_asserts_initial_state() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "No more `open` calls expected."]
    fn fake_systemd_provider_unexpected_open_call_panics() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        let _open_result = <&FakeJournalRef>::open(&PROVIDER);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `open`: [Ok(())]"]
    fn fake_systemd_provider_extra_open_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `open`: [Ok(())]"]
    fn fake_systemd_provider_extra_open_call_after_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Err(libc::EACCES));
        PROVIDER.open.enqueue_io(Ok(()));
        let _open_result = <&FakeJournalRef>::open(&PROVIDER);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    fn fake_systemd_provider_expected_open_call_works() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        <&FakeJournalRef>::open(&PROVIDER).unwrap();
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    fn fake_systemd_provider_expected_watchdog_notify_call_works() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.watchdog_notify.enqueue_io(Ok(()));
        assert_result_eq(PROVIDER.watchdog_notify(), Ok(()));
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `watchdog_notify`: [Ok(())]"]
    fn fake_systemd_provider_extra_watchdog_notify_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.watchdog_notify.enqueue_io(Ok(()));
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `watchdog_notify`: [Ok(())]"]
    fn fake_systemd_provider_extra_watchdog_notify_call_after_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.watchdog_notify.enqueue_io(Err(libc::EACCES));
        PROVIDER.watchdog_notify.enqueue_io(Ok(()));
        assert_result_eq(
            PROVIDER.watchdog_notify(),
            Err(Error::from_raw_os_error(libc::EACCES)),
        );
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    fn fake_systemd_provider_expected_get_monotonic_time_usec_call_works() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.get_monotonic_time_usec.enqueue(123);
        assert_eq!(
            PROVIDER.get_monotonic_time_usec(),
            SystemdMonotonicUsec(123)
        );
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `get_monotonic_time_usec`: [123]"]
    fn fake_systemd_provider_extra_get_monotonic_time_usec_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.get_monotonic_time_usec.enqueue(123);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `get_monotonic_time_usec`: [123]"]
    fn fake_systemd_provider_extra_get_monotonic_time_usec_call_after_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.get_monotonic_time_usec.enqueue(456);
        PROVIDER.get_monotonic_time_usec.enqueue(123);
        assert_eq!(
            PROVIDER.get_monotonic_time_usec(),
            SystemdMonotonicUsec(456)
        );
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    fn fake_systemd_provider_expected_set_data_threshold_call_works() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.set_data_threshold.enqueue_io(Ok(()));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER)
                .unwrap()
                .set_data_threshold(123),
            Ok(()),
        );
        PROVIDER.journal.set_data_threshold.assert_calls(&[123]);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `set_data_threshold`: [Ok(())]"]
    fn fake_systemd_provider_extra_set_data_threshold_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.set_data_threshold.enqueue_io(Ok(()));
        let _ = <&FakeJournalRef>::open(&PROVIDER).unwrap();
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `set_data_threshold`: [Ok(())]"]
    fn fake_systemd_provider_expected_set_data_threshold_call_after_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER
            .journal
            .set_data_threshold
            .enqueue_io(Err(libc::EACCES));
        PROVIDER.journal.set_data_threshold.enqueue_io(Ok(()));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER)
                .unwrap()
                .set_data_threshold(123),
            Err(Error::from_raw_os_error(libc::EACCES)),
        );
        PROVIDER.journal.set_data_threshold.assert_calls(&[123]);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    fn fake_systemd_provider_expected_seek_monotonic_usec_call_works() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.seek_monotonic_usec.enqueue_io(Ok(()));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER)
                .unwrap()
                .seek_monotonic_usec(&Id128(0), SystemdMonotonicUsec(123)),
            Ok(()),
        );
        PROVIDER
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(0), 123)]);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `seek_monotonic_usec`: [Ok(())]"]
    fn fake_systemd_provider_extra_seek_monotonic_usec_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.seek_monotonic_usec.enqueue_io(Ok(()));
        let _ = <&FakeJournalRef>::open(&PROVIDER).unwrap();
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `seek_monotonic_usec`: [Ok(())]"]
    fn fake_systemd_provider_expected_seek_monotonic_usec_call_after_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER
            .journal
            .seek_monotonic_usec
            .enqueue_io(Err(libc::EACCES));
        PROVIDER.journal.seek_monotonic_usec.enqueue_io(Ok(()));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER)
                .unwrap()
                .seek_monotonic_usec(&Id128(0), SystemdMonotonicUsec(123)),
            Err(Error::from_raw_os_error(libc::EACCES)),
        );
        PROVIDER
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(0), 123)]);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    fn fake_systemd_provider_expected_seek_cursor_call_works() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.seek_cursor.enqueue_io(Ok(()));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER)
                .unwrap()
                .seek_cursor(&Cursor::new(b"0123456789")),
            Ok(()),
        );
        PROVIDER
            .journal
            .seek_cursor
            .assert_calls(&[Cursor::new(b"0123456789")]);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `seek_cursor`: [Ok(())]"]
    fn fake_systemd_provider_extra_seek_cursor_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.seek_cursor.enqueue_io(Ok(()));
        let _ = <&FakeJournalRef>::open(&PROVIDER).unwrap();
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `seek_cursor`: [Ok(())]"]
    fn fake_systemd_provider_expected_seek_cursor_call_after_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.seek_cursor.enqueue_io(Err(libc::EACCES));
        PROVIDER.journal.seek_cursor.enqueue_io(Ok(()));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER)
                .unwrap()
                .seek_cursor(&Cursor::new(b"0123456789")),
            Err(Error::from_raw_os_error(libc::EACCES)),
        );
        PROVIDER
            .journal
            .seek_cursor
            .assert_calls(&[Cursor::new(b"0123456789")]);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    fn fake_systemd_provider_expected_wait_call_works() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.wait.enqueue_io(Ok(true));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER)
                .unwrap()
                .wait(Duration::from_millis(123)),
            Ok(true),
        );
        PROVIDER
            .journal
            .wait
            .assert_calls(&[Duration::from_millis(123)]);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `wait`: [Ok(true)]"]
    fn fake_systemd_provider_extra_wait_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.wait.enqueue_io(Ok(true));
        let _ = <&FakeJournalRef>::open(&PROVIDER).unwrap();
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `wait`: [Ok(false)]"]
    fn fake_systemd_provider_expected_wait_call_after_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.wait.enqueue_io(Ok(true));
        PROVIDER.journal.wait.enqueue_io(Ok(false));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER)
                .unwrap()
                .wait(Duration::from_millis(123)),
            Ok(true),
        );
        PROVIDER
            .journal
            .wait
            .assert_calls(&[Duration::from_millis(123)]);
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    fn fake_systemd_provider_expected_next_call_works() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.next.enqueue_io(Ok(true));
        assert_result_eq(<&FakeJournalRef>::open(&PROVIDER).unwrap().next(), Ok(true));
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `next`: [Ok(true)]"]
    fn fake_systemd_provider_extra_next_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.next.enqueue_io(Ok(true));
        let _ = <&FakeJournalRef>::open(&PROVIDER).unwrap();
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `next`: [Ok(false)]"]
    fn fake_systemd_provider_expected_next_call_after_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER.journal.next.enqueue_io(Ok(true));
        PROVIDER.journal.next.enqueue_io(Ok(false));
        assert_result_eq(<&FakeJournalRef>::open(&PROVIDER).unwrap().next(), Ok(true));
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    fn fake_systemd_provider_expected_cursor_call_works() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER
            .journal
            .cursor
            .enqueue_io(Ok(Cursor::new(b"0123456789")));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER).unwrap().cursor(),
            Ok(Cursor::new(b"0123456789")),
        );
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `cursor`: [Ok(Cursor(\"0123456789\"))]"]
    fn fake_systemd_provider_extra_cursor_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER
            .journal
            .cursor
            .enqueue_io(Ok(Cursor::new(b"0123456789")));
        let _ = <&FakeJournalRef>::open(&PROVIDER).unwrap();
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `cursor`: [Ok(Cursor(\"9876543210\"))]"]
    fn fake_systemd_provider_expected_cursor_call_after_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER
            .journal
            .cursor
            .enqueue_io(Ok(Cursor::new(b"0123456789")));
        PROVIDER
            .journal
            .cursor
            .enqueue_io(Ok(Cursor::new(b"9876543210")));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER).unwrap().cursor(),
            Ok(Cursor::new(b"0123456789")),
        );
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    fn fake_systemd_provider_expected_get_data_call_works() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER
            .journal
            .get_data
            .enqueue_io(FixedCString::new(b"FOO_BAR"), Err(libc::EBADMSG));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER)
                .unwrap()
                .get_data(cstr!("FOO_BAR")),
            Err(Error::from_raw_os_error(libc::EBADMSG)),
        );
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `get_data`: \
    {\"FOO_BAR\": [Err(Os { code: 74, kind: Uncategorized, message: \"Bad message\" })]}"]
    fn fake_systemd_provider_extra_get_data_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER
            .journal
            .get_data
            .enqueue_io(FixedCString::new(b"FOO_BAR"), Err(libc::EBADMSG));
        let _ = <&FakeJournalRef>::open(&PROVIDER).unwrap();
        PROVIDER.assert_no_calls_remaining();
    }

    #[test]
    #[should_panic = "Unexpected calls remaining for `get_data`: {\"FOO_BAR\": [Ok([1, 2, 3])]}"]
    fn fake_systemd_provider_expected_get_data_call_after_call_is_asserted() {
        static PROVIDER: FakeSystemdProvider = FakeSystemdProvider::new(Id128(0));
        PROVIDER.open.enqueue_io(Ok(()));
        PROVIDER
            .journal
            .get_data
            .enqueue_io(FixedCString::new(b"FOO_BAR"), Err(libc::EBADMSG));
        PROVIDER
            .journal
            .get_data
            .enqueue_io(FixedCString::new(b"FOO_BAR"), Ok(&[1, 2, 3]));
        assert_result_eq(
            <&FakeJournalRef>::open(&PROVIDER)
                .unwrap()
                .get_data(cstr!("FOO_BAR")),
            Err(Error::from_raw_os_error(libc::EBADMSG)),
        );
        PROVIDER.assert_no_calls_remaining();
    }
}
