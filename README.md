# Async Timers

This crate provides ```PeriodicTimer``` and ```OneshotTimer``` to be used in ```async``` context (tokio).

# Usage

```rust
use async_timers::OneshotTimer;
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() {
    let mut timer = OneshotTimer::expired();

    // call will block forever
    // timer.tick().await;

    // Useful in event loop in select! blocks when one branch can start OneshotTimer with timer.schedule()
    // and the next time select! will call the branch corresponding to that timer.

    let mut another_timer = PeriodicTimer::started(Duration::from_secs(3));

    let mut start = Instant::now();
    let mut i = 1;
    loop {
        tokio::select! {
            _ = timer.tick() => {
                println!("{}: I am triggered: {}", start.elapsed().as_millis(), i);
            }
            _ = another_timer.tick() => {
                if i < 10 {
                    timer.schedule(Duration::from_millis(i * 300));
                    i += 1;
                    start = Instant::now();
                } else {
                    another_timer.stop()
                }
            }
        }
    }
}
```
