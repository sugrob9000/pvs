use crate::hal::raw::*;
use core::cmp::max;
use core::ops::{Add, Sub};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
  millis_since_epoch: u32,
}

impl Instant {
  pub fn now() -> Self {
    let millis_since_epoch = unsafe { HAL_GetTick() };
    Self { millis_since_epoch }
  }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
  millis: u32,
}

impl Duration {
  pub fn from_millis(millis: u32) -> Self {
    Self { millis }
  }

  pub fn as_millis(self) -> u32 {
    self.millis
  }
}

impl Add<Duration> for Instant {
  type Output = Self;
  fn add(self, rhs: Duration) -> Self::Output {
    Self::Output {
      millis_since_epoch: self.millis_since_epoch + rhs.as_millis(),
    }
  }
}

impl Add<Instant> for Duration {
  type Output = Instant;
  fn add(self, rhs: Instant) -> Self::Output {
    rhs + self
  }
}

impl Sub<Instant> for Instant {
  type Output = Duration;
  fn sub(self, rhs: Instant) -> Self::Output {
    Duration::from_millis(max(0, self.millis_since_epoch - rhs.millis_since_epoch))
  }
}
