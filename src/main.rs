use core::pin::pin;
use std::time::{Duration, Instant};

mod rt;

async fn loud_wait(tag: &str, duration: Duration) {
  println!("{tag}: wait {duration:?}.");
  let started = Instant::now();
  rt::sleep_for(duration).await;
  println!(
    "{tag}: done {duration:?}, really slept {:?}.",
    Instant::now() - started
  );
}

pub fn main() {
  let simple_wait = pin!(loud_wait("A", Duration::from_millis(150)));
  let looped_wait = pin!(async {
    const N: u64 = 5;
    for i in 1..=N {
      loud_wait(
        format!("{i}/{N}").as_str(),
        Duration::from_millis(20 + 10 * i),
      )
      .await;
    }
  });

  rt::Executor::new()
    .with_task(simple_wait)
    .with_task(looped_wait)
    .run();

  println!("All tasks have finished.");
}
