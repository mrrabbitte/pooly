use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};
use std::sync::{LockResult, Mutex, MutexGuard};
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use std::time::Duration;

use crate::models::utils::time::{now_millis, NowProvider};

/// A very simplistic leaky bucket implementation.
struct LeakyBucket<T> where T: NowProvider {

    clock: T,

    tickets: AtomicU32,
    max_requests_per_period: u32,

    period_millis: u128,
    last_updated_at_millis: Mutex<u128>

}

impl<T: NowProvider>  LeakyBucket<T> {

    pub fn new(clock: T,
               max_requests_per_period: u32,
               period_millis: u128) -> LeakyBucket<T> {
        let now_millis = clock.now_millis();

        LeakyBucket {
            clock,
            tickets: AtomicU32::new(0),
            max_requests_per_period,
            period_millis,
            last_updated_at_millis: Mutex::new(now_millis)
        }
    }

    pub fn acquire(&self) -> Result<(), LeakyBucketError> {
        if self.has_free_tickets() {
            return Ok(());
        }

        self.update_tickets()?;

        if self.has_free_tickets() {
            return Ok(());
        }

        return Err(LeakyBucketError::TooManyRequests {
            threshold: self.max_requests_per_period,
            period_millis: self.period_millis
        });
    }

    fn has_free_tickets(&self) -> bool {
        self.tickets.fetch_add(1, Ordering::SeqCst) < self.max_requests_per_period
    }

    fn update_tickets(&self) -> Result<(), LeakyBucketError> {
        match self.last_updated_at_millis.lock() {
            Ok(mut current_last) => {
                let now = self.clock.now_millis();

                if *current_last + self.period_millis <= now {
                    self.tickets.store(0, Ordering::SeqCst);
                    *current_last = now;
                }

                Ok(())
            }
            Err(_) => Err(LeakyBucketError::PoisonedLock)
        }
    }


}

#[derive(Debug)]
pub enum LeakyBucketError {

    TooManyRequests{threshold: u32, period_millis: u128},
    PoisonedLock

}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    use crate::models::utils::time::{Clock, now_millis, NowProvider};
    use crate::services::limits::LeakyBucket;

    const PERIOD_MILLIS: u128 = 10_000; // assuming the test will run in < 10 sec
    const MAX_REQUESTS_PER_PERIOD: u32 = 3;

    const NUM_NANOS_IN_MICRO: u32 = 1_000;

    #[test]
    fn test_creates_bucket_correctly() {
        let bucket =
            LeakyBucket::new(Clock::new(), MAX_REQUESTS_PER_PERIOD, PERIOD_MILLIS);

        assert!(bucket.acquire().is_ok());
    }

    #[test]
    fn test_disallows_for_abuse_of_quota() {
        let bucket =
            LeakyBucket::new(Clock::new(), MAX_REQUESTS_PER_PERIOD, PERIOD_MILLIS);

        assert!(bucket.acquire().is_ok());
        assert!(bucket.acquire().is_ok());
        assert!(bucket.acquire().is_ok());

        assert!(bucket.acquire().is_err());
        assert!(bucket.acquire().is_err());
    }

    #[test]
    fn test_allows_for_new_requests_after_appropriate_wait_time() {
        let now = Arc::new(AtomicU64::new(0));

        let bucket =
            LeakyBucket::new(
                MockClock { now: now.clone() }, MAX_REQUESTS_PER_PERIOD, PERIOD_MILLIS);

        assert!(bucket.acquire().is_ok());
        assert!(bucket.acquire().is_ok());
        assert!(bucket.acquire().is_ok());

        assert!(bucket.acquire().is_err());

        now.fetch_add(PERIOD_MILLIS as u64, Ordering::SeqCst);

        assert!(bucket.acquire().is_ok());
        assert!(bucket.acquire().is_ok());
        assert!(bucket.acquire().is_ok());

        assert!(bucket.acquire().is_err());
    }

    #[test]
    fn test_multiple_threads_holds_threshold() {
        let num_successes = Arc::new(AtomicU32::new(0));
        let now = Arc::new(AtomicU64::new(0));

        let bucket = Arc::new(LeakyBucket::new(
            MockClock { now: now.clone() }, MAX_REQUESTS_PER_PERIOD, PERIOD_MILLIS));

        spawn_worker(bucket.clone(), num_successes.clone());
        spawn_worker(bucket.clone(), num_successes.clone());
        spawn_worker(bucket.clone(), num_successes.clone());

        thread::sleep(Duration::from_millis(1));

        assert_eq!(num_successes.load(Ordering::SeqCst), MAX_REQUESTS_PER_PERIOD);

        now.fetch_add(PERIOD_MILLIS as u64, Ordering::SeqCst);

        thread::sleep(Duration::from_millis(1));

        assert_eq!(num_successes.load(Ordering::SeqCst), MAX_REQUESTS_PER_PERIOD * 2);

        thread::sleep(Duration::from_millis(10));

        assert_eq!(num_successes.load(Ordering::SeqCst), MAX_REQUESTS_PER_PERIOD * 2);
    }

    fn spawn_worker(bucket: Arc<LeakyBucket<MockClock>>,
                    num_successes: Arc<AtomicU32>) {
        thread::spawn(move ||
            loop {
                if bucket.acquire().is_ok() {
                    num_successes.fetch_add(1, Ordering::SeqCst);
                    thread::sleep(Duration::new(0, NUM_NANOS_IN_MICRO));
                }
            });
    }

    struct MockClock {
        now: Arc<AtomicU64>
    }

    impl NowProvider for MockClock {
        fn now_millis(&self) -> u128 {
            self.now.load(Ordering::SeqCst) as u128
        }
    }
}
