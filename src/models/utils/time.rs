use std::time::{Duration, SystemTime};

///
/// Please note, if you have to use more functionalities, please start using chrono crate.
///
#[inline]
pub fn now_nanos() -> u128 {
    now().as_nanos()
}

#[inline]
pub fn now_millis() -> u128 {
    now().as_millis()
}

#[inline]
pub fn now_seconds() -> u64 {
    now().as_secs()
}

#[inline]
fn now() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system time error")
}

pub trait NowProvider {
    fn now_millis(&self) -> u128;
}

pub struct Clock;

impl Clock {

    pub fn new() -> Clock {
        Clock {}
    }
}

impl NowProvider for Clock {
    fn now_millis(&self) -> u128 {
        now_millis()
    }
}
