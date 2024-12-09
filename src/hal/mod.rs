//! Module with thin wrappers around the C STM32 HAL
//! (we are not using the stm32f4xx-hal crate for this)

pub mod time;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PinState {
  Reset = 0,
  Set = 1,
}

#[derive(Clone, Copy)]
#[repr(u32)]
#[non_exhaustive]
pub enum GpioBase {
  C = 0x40020800,
  D = 0x40020C00,
}

#[derive(Clone, Copy)]
#[repr(u16)]
#[non_exhaustive]
pub enum GpioPin {
  Pin13 = 0x2000,
  Pin15 = 0x8000,
}

pub mod raw {
  use super::*;
  pub use core::ffi::{c_char, CStr};

  extern "C" {
    pub fn HAL_GetTick() -> u32;
    pub fn HAL_GPIO_ReadPin(base: GpioBase, pin: GpioPin) -> PinState;
    pub fn HAL_GPIO_WritePin(base: GpioBase, pin: GpioPin, state: PinState);
    pub fn debug_print(msg: *const c_char, number: u32);
  }
}

pub fn button_pressed() -> bool {
  use raw::*;
  let state = unsafe { HAL_GPIO_ReadPin(GpioBase::C, GpioPin::Pin15) };
  state == PinState::Reset
}

/// Print an arbitrary `const char*` and a number
#[allow(dead_code)]
pub fn debug_print(msg: &raw::CStr, number: u32) {
  unsafe { raw::debug_print(msg.as_ptr(), number) }
}

pub fn set_led(led: u8, state: PinState) {
  // 13 - green
  // 14 - red
  // 15 - yellow
  use raw::*;
  unsafe {
    HAL_GPIO_WritePin(
      GpioBase::D,
      match led {
        0 => GpioPin::Pin15,
        1 => GpioPin::Pin13,
        _ => panic!(),
      },
      state,
    );
  }
}
