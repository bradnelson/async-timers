# Async Timers

[![Crates.io][crates-badge]][crates-url]
[![Unlicensed][licence-badge]][licence-url]
[![Build Status][actions-badge]][actions-url]
[![codecov](https://codecov.io/github/Blyschak/async-timers/branch/main/graph/badge.svg?token=322R7ISIMY)](https://codecov.io/github/Blyschak/async-timers)

[crates-badge]: https://img.shields.io/badge/crates.io-v0.1.0-blue
[crates-url]: https://crates.io/crates/async-timers
[licence-badge]: https://img.shields.io/badge/license-Unlicense-blue.svg
[licence-url]: https://github.com/Blyschak/async-timers/blob/master/LICENSE
[actions-badge]: https://github.com/Blyschak/async-timers/actions/workflows/build.yml/badge.svg
[actions-url]: https://github.com/Blyschak/async-timers/actions?query=workflow%3ACI+branch%3Amaster

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
