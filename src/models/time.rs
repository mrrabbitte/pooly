use std::time::{Duration, SystemTime};

///
/// Please note, if you have to use more functionalities, please start using chrono crate.
///
pub fn now_nanos() -> u128 {
    now().as_nanos()
}

pub fn now_millis() -> u128 {
    now().as_millis()
}

fn now() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system time error")
}
