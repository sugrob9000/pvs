use core::pin::pin;
use std::time::{Duration, Instant};

mod rt;

async fn loud_wait(tag: String, duration: Duration) {
  println!("{tag}: wait {duration:?}.");
  let start = Instant::now();
  rt::sleep_for(duration).await;
  let really = Instant::now() - start;
  println!("{tag}: done {duration:?}, really slept {really:?}.");
}

async fn loud_wait_n(n: u64) {
  for i in 1..=n {
    loud_wait(format!("{i}/{n}"), Duration::from_millis(10 * i)).await;
  }
}

pub fn main() {
  let a = pin!(loud_wait("A".to_string(), Duration::from_millis(150)));
  let b = pin!(async {
    loud_wait("C1".to_string(), Duration::from_millis(12)).await;
    loud_wait("C2".to_string(), Duration::from_millis(12)).await;
  });
  let c = pin!(loud_wait_n(5));

  rt::Executor::new()
    .with_task(a)
    .with_task(b)
    .with_task(c)
    .run();

  println!("All tasks have finished.");
}
