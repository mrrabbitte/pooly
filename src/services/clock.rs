use crate::models::utils::time;

#[derive(Clone)]
pub struct Clock;

impl Clock {

    pub fn new() -> Clock {
        Clock {}
    }

    pub fn now_seconds(&self) -> u64 {
        time::now_seconds()
    }

}