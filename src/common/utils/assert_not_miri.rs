#[cfg(not(miri))]
pub fn assert_not_miri() {}

#[cfg(miri)]
#[track_caller]
pub fn assert_not_miri() {
    panic!("This function should not be running under Miri");
}
