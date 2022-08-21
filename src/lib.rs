//! Async timers crate
//!
//! This library provides timers that can be easily scheduled and canceled. For example, the tokio's [`tokio::time::Interval`] has no way of stopping the timer.
//! You could have set the interval duration to a very big value, however that is rather a work around. Also, tokio's [`tokio::time::Sleep`] is a one-time use object,
//! meaning it's .await requires to move the object and requires you to recreated it when you need to sleep again.
//!
//! This crate provides [`PeriodicTimer`] and [`OneshotTimer`] that aim to make the use of timers more pleasant.
//! This timers have methods to cancel and restart timers.
use std::task;

use futures::Future;
use tokio::time::{interval, sleep_until, Duration, Instant, Interval};

/// NeverExpire is a future that never unblocks
#[derive(Default, Debug)]
struct NeverExpire {}

impl Future for NeverExpire {
    type Output = Instant;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        task::Poll::Pending
    }
}

/// PeriodicTimer expires on given interval
///
/// PeriodicTimer is an extension and built on top of [`tokio::time::Interval`].
/// It can be in two states: [`PeriodicTimer::Started`] and [`PeriodicTimer::Stopped`].
/// When in [`PeriodicTimer::Started`] state the timer will expire every interval duration but
/// when in [`PeriodicTimer::Stopped`] it won't expire until the timer is started again.
///
/// ```
/// use async_timers::PeriodicTimer;
/// use tokio::time::{Duration, timeout};
///
/// #[tokio::main]
/// async fn main() {
///     let mut timer = PeriodicTimer::started(Duration::from_millis(10));
///
///     timer.tick().await;
///     timer.tick().await;
///     timer.tick().await;
///
///     // approximately 30ms have elapsed.
///
///     let result = timeout(Duration::from_millis(100), timer.tick()).await;
///     assert!(result.is_ok(), "Timeout should not occur since timer is running");
///
///     timer.stop();
///
///     let result = timeout(Duration::from_millis(100), timer.tick()).await;
///     assert!(result.is_err(), "Timeout should occur since timer is stopped");
/// }
/// ```
#[derive(Default, Debug)]
pub enum PeriodicTimer {
    Started(Interval),
    #[default]
    Stopped,
}

impl PeriodicTimer {
    /// Create started timer with the given `period`
    pub fn started(period: Duration) -> Self {
        Self::Started(interval(period))
    }

    /// Create stopped timer
    pub fn stopped() -> Self {
        Self::Stopped
    }

    /// Start the timer with given `period`
    pub fn start(&mut self, period: Duration) {
        *self = Self::started(period);
    }

    /// Stop the timer
    pub fn stop(&mut self) {
        *self = Self::stopped()
    }

    /// Returns a [`Future`] that will expire based on timer's state
    pub async fn tick(&mut self) -> Instant {
        match self {
            Self::Started(interval) => interval.tick().await,
            Self::Stopped => NeverExpire::default().await,
        }
    }
}

/// OneshotTimer expires once after a given duration
///
/// OneshotTimer is used for tasks that need to be executed once after some delay.
/// OneshotTimer is an extension and built on top of [`tokio::time::Sleep`].
/// In [`OneshotTimer::Scheduled`] state it will expire *once* and transition into
/// [`OneshotTimer::Expired`] state.
///
/// ```
/// use async_timers::OneshotTimer;
/// use tokio::time::{Duration, timeout};
///
/// #[tokio::main]
/// async fn main() {
///     let mut timer = OneshotTimer::scheduled(Duration::from_millis(10));
///
///     timer.tick().await;
///
///     // approximately 10ms have elapsed.
///
///     let result = timeout(Duration::from_millis(100), timer.tick()).await;
///     assert!(result.is_err(), "Timeout should occur since timer is expired");
///
///     timer.schedule(Duration::from_millis(30));
///
///     let result = timeout(Duration::from_millis(100), timer.tick()).await;
///     assert!(result.is_ok(), "Timeout should not occur since timer has been scheduled");
/// }
/// ```
#[derive(Default, Debug)]
pub enum OneshotTimer {
    Scheduled(Instant),
    #[default]
    Expired,
}

impl OneshotTimer {
    /// Create a timer scheduled to be expired after `duration`
    pub fn scheduled(duration: Duration) -> Self {
        Self::Scheduled(Instant::now() + duration)
    }

    /// Create a timer that won't expire
    pub fn expired() -> Self {
        Self::Expired
    }

    /// Schedule a new duration
    pub fn schedule(&mut self, duration: Duration) {
        *self = Self::scheduled(duration);
    }

    /// Cancel the timer
    pub fn cancel(&mut self) {
        *self = Self::expired()
    }

    /// Returns a [`Future`] that will expire based on timer's state
    pub async fn tick(&mut self) {
        match self {
            Self::Scheduled(instant) => {
                sleep_until(*instant).await;
                *self = Self::expired();
            }
            Self::Expired => {
                NeverExpire::default().await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_periodic_timer() {
        let mut timer1 = PeriodicTimer::stopped();
        let mut timer2 = PeriodicTimer::started(Duration::from_secs(2));

        let mut timer1_expired = false;
        let mut timer2_expired = false;

        tokio::select! {
            _ = timer1.tick() => {
                timer1_expired = true;
            }
            _ = timer2.tick() => {
                timer2_expired = true;
            }
        }

        assert!(!timer1_expired, "timer1 should not have expired");
        assert!(timer2_expired, "timer1 should have expired");

        timer1.start(Duration::from_secs(1));
        timer2.stop();

        timer1_expired = false;
        timer2_expired = false;

        tokio::select! {
            _ = timer1.tick() => {
                timer1_expired = true;
            }
            _ = timer2.tick() => {
                timer2_expired = true;
            }
        }

        assert!(timer1_expired, "timer1 should have expired");
        assert!(!timer2_expired, "timer2 should not have expired");
    }

    #[tokio::test]
    async fn test_oneshot_timer() {
        let mut timer1 = OneshotTimer::expired();
        let mut timer2 = OneshotTimer::scheduled(Duration::from_secs(2));

        let mut timer1_expired = false;
        let mut timer2_expired = false;

        tokio::select! {
            _ = timer1.tick() => {
                timer1_expired = true;
            }
            _ = timer2.tick() => {
                timer2_expired = true;
            }
        }

        assert!(!timer1_expired, "timer1 should not have expired");
        assert!(timer2_expired, "timer1 should have expired");

        timer1.schedule(Duration::from_secs(1));

        timer1_expired = false;
        timer2_expired = false;

        tokio::select! {
            _ = timer1.tick() => {
                timer1_expired = true;
            }
            _ = timer2.tick() => {
                timer2_expired = true;
            }
        }

        assert!(timer1_expired, "timer1 should have expired");
        assert!(!timer2_expired, "timer2 should not have expired");

        timer1.schedule(Duration::from_secs(1));
        timer2.schedule(Duration::from_secs(2));

        timer1.cancel();

        timer1_expired = false;
        timer2_expired = false;

        tokio::select! {
            _ = timer1.tick() => {
                timer1_expired = true;
            }
            _ = timer2.tick() => {
                timer2_expired = true;
            }
        }

        assert!(!timer1_expired, "timer1 should not have expired");
        assert!(timer2_expired, "timer2 should have expired");
    }

    #[tokio::test]
    async fn test_oneshot_state() {
        let mut timer1 = OneshotTimer::scheduled(Duration::from_secs(1));
        let result = tokio::time::timeout(Duration::from_millis(1500), timer1.tick()).await;
        assert!(result.is_ok(), "Should not timeout");

        let mut timer1 = OneshotTimer::scheduled(Duration::from_secs(5));
        let mut timer2 = OneshotTimer::scheduled(Duration::from_secs(2));

        tokio::select! {
            _ = timer1.tick() => {}
            _ = timer2.tick() => {}
        }

        match timer1 {
            OneshotTimer::Scheduled(_) => {}
            OneshotTimer::Expired => assert!(false, "Should be in scheduled state"),
        }

        let result = tokio::time::timeout(Duration::from_millis(3500), timer1.tick()).await;
        assert!(result.is_ok(), "Should not timeout");

        match timer1 {
            OneshotTimer::Scheduled(_) => assert!(false, "Timer should be in expired state"),
            OneshotTimer::Expired => {}
        }
    }

    #[tokio::test]
    async fn test_my_task() {
        struct MyTask {
            period: PeriodicTimer,
        }

        impl MyTask {
            fn new() -> Self {
                Self {
                    period: PeriodicTimer::started(Duration::from_secs(1)),
                }
            }

            fn do_work(&mut self) {
                println!("here");
            }
        }

        let mut task = MyTask::new();
        let mut sleep = OneshotTimer::scheduled(Duration::from_secs(3));

        let result = tokio::time::timeout(Duration::from_secs(10), async move {
            for _ in 0..3 {
                tokio::select! {
                    _ = task.period.tick() => {
                        task.do_work();
                        task.period.stop();
                    }
                    _ = sleep.tick() => {
                        println!("sleep");
                        task.period.start(Duration::from_secs(1));
                    }
                }
            }
        })
        .await;

        assert!(result.is_ok(), "Should not timeout");
    }
}
