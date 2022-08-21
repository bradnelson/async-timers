use async_timers::{OneshotTimer, PeriodicTimer};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    let mut start_delay = OneshotTimer::expired();
    let mut stop_delay = OneshotTimer::expired();
    let mut periodic = PeriodicTimer::stopped();

    let mut exit_loop_delay = OneshotTimer::scheduled(Duration::from_secs(10));

    // The following call will block forever
    // start_delay.tick().await;

    // Useful in event loop in select! blocks

    start_delay.schedule(Duration::from_secs(2));
    println!("Periodic timer will start in 2 sec");

    loop {
        tokio::select! {
            _ = start_delay.tick() => {
                // Start periodic timer with period of 500 ms
                periodic.start(Duration::from_millis(500));

                stop_delay.schedule(Duration::from_secs(3));
                println!("Periodic timer will stop in 3 sec");
            }
            _ = stop_delay.tick() => {
                // Stop periodic timer
                periodic.stop();
                exit_loop_delay.schedule(Duration::from_secs(3));
                println!("Periodic timer stopped. Will exit in 3 sec");
            }
            _ = periodic.tick() => {
                println!("Periodic tick!");
            }
            _ = exit_loop_delay.tick() => {
                println!("Bye!");
                break;
            }
        }
    }
}
