#![no_std]

use core::pin::pin;
use core::sync::atomic::{AtomicU32, Ordering::SeqCst};
use hal::Duration;
use panic_halt as _;

mod hal;
mod rt;

static OVERFLOWS: AtomicU32 = AtomicU32::new(0);

async fn buttons() {}

async fn leds() {}

#[no_mangle]
extern "C" fn real_main() {
  rt::Executor::new()
    .with_task(pin!(buttons()))
    .with_task(pin!(leds()))
    .run();
}
