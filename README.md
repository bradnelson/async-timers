# Async Timers

This crate provides ```PeriodicTimer``` and ```OneshotTimer``` to be used in ```async``` context (tokio).

# Usage

```rust
use async_timers::OneshotTimer;
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() {
    let mut oneshot_timer = OneshotTimer::expired();

    // call will block forever
    // oneshot_timer.tick().await;

    // Useful in event loop in select! blocks when one branch can start OneshotTimer with oneshot_timer.schedule()
    // and the next time select! will call the branch corresponding to that timer.

    let mut periodic_timer = PeriodicTimer::started(Duration::from_secs(3));

    let mut start = Instant::now();
    let mut i = 1;
    loop {
        tokio::select! {
            _ = oneshot_timer.tick() => {
                println!("{}: I am triggered: {}", start.elapsed().as_millis(), i);
            }
            _ = periodic_timer.tick() => {
                if i < 10 {
                    oneshot_timer.schedule(Duration::from_millis(i * 300));
                    i += 1;
                    start = Instant::now();
                } else {
                    periodic_timer.stop()
                }
            }
        }
    }
}
```
