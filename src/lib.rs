#![no_std]
#![allow(static_mut_refs)]

use crate::hal::time::Duration;
use core::pin::pin;
use hal::PinState;
use panic_halt as _;

mod hal;
mod rt;

// SAFETY: References to COUNTER and OVERFLOWS never escape, and we are the only thread.
static mut COUNTER: u32 = 0;
static mut OVERFLOWS: u32 = 0;
static mut ANIMATION_REQUEST: Option<u32> = None;

pub fn counter_add(value: i32) -> Result<(), u32> {
  unsafe {
    let wrapped = COUNTER.wrapping_add_signed(value);
    COUNTER = wrapped & 0b11;
    if COUNTER != wrapped {
      OVERFLOWS = OVERFLOWS.saturating_add_signed(value);
      Err(OVERFLOWS)
    } else {
      Ok(())
    }
  }
}

async fn button_main() {
  loop {
    let millis = rt::wait_button_release().await.as_millis();
    let incr = if millis < 500 { 1i32 } else { -1i32 };

    counter_add(incr).unwrap_or_else(|overflows| unsafe {
      if overflows > 0 {
        hal::debug_print(c"Requesting animation...", overflows);
        ANIMATION_REQUEST = Some(overflows);
      }
    });
  }
}

fn set_leds(led_mask: u8, state: PinState) {
  for led in 0..=1 {
    if (led_mask & (1u8 << led)) != 0 {
      hal::set_led(led, state);
    }
  }
}

async fn flash_leds(led_mask: u8, period: Duration) {
  for state in [PinState::Reset, PinState::Set, PinState::Reset] {
    set_leds(led_mask, state);
    rt::sleep_for(period).await;
  }
}

async fn led_main() {
  loop {
    unsafe {
      if let Some(n) = ANIMATION_REQUEST.take() {
        hal::debug_print(c"Got animation request...", n);
        let period = Duration::from_millis(250);
        flash_leds(0b11, period).await;
        for _ in 0..n {
          flash_leds(0b01, period).await;
        }
      }
      for led in 0..=1 {
        hal::set_led(
          led,
          if (COUNTER & (1u32 << led)) != 0 {
            PinState::Set
          } else {
            PinState::Reset
          },
        );
      }
    }
    rt::wait_button_release().await;
  }
}

#[no_mangle]
extern "C" fn real_main() {
  rt::Executor::new()
    .with_task(pin!(button_main()))
    .with_task(pin!(led_main()))
    .run();
}
