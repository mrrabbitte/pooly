use std::time::SystemTime;

///
/// Please note, if you have to use more functionalities, please start using chrono crate.
///
pub fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system time error")
        .as_nanos()
}
