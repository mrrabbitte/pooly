use crate::models::time;

#[derive(Clone)]
pub struct Clock;

impl Clock {

    pub fn now_seconds(&self) -> u64 {
        time::now_seconds()
    }

}