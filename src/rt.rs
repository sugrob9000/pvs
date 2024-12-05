//! The simplest executor I was able to come up with.
//! - Inplace buffer for fixed number of tasks
//! - No spawner; all tasks must have been spawned before running

use crate::hal::{Duration, Instant};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub type PinnedFuture<'t> = Pin<&'t mut dyn Future<Output = ()>>;

pub struct Executor<'t> {
  task_slots: [Option<PinnedFuture<'t>>; 4],
}

impl<'t> Executor<'t> {
  pub fn new() -> Self {
    Self {
      task_slots: Default::default(),
    }
  }

  pub fn with_task(mut self, task: PinnedFuture<'t>) -> Self {
    let slot = self
      .task_slots
      .iter_mut()
      .find(|slot| slot.is_none())
      .expect("Nowhere to push task");
    *slot = Some(task);
    self
  }

  pub fn run(mut self) {
    loop {
      let mut any_tasks_left = false;

      for task_slot in self.task_slots.iter_mut() {
        if let Some(task) = task_slot {
          // Waker is a formality here, we don't get to actually externally wake anything.
          // Context also is, because in Rust all it does is give you access to a waker.

          let waker = unsafe {
            // SAFETY: TODO ensure it's OK to create this waker (besides the fact that
            // we never wake anything yet). Probably should pin both self and task.
            let raw = RawWaker::new(task as *const PinnedFuture as *const (), &WAKER_VTABLE);
            Waker::from_raw(raw)
          };

          match task.as_mut().poll(&mut Context::from_waker(&waker)) {
            Poll::Ready(()) => *task_slot = None,
            Poll::Pending => any_tasks_left = true,
          }
        }
      }

      if !any_tasks_left {
        break;
      }
    }
  }
}

static WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
  // Input is a mere pointer to a task. Cloned trivially.
  |clone_me| RawWaker::new(clone_me, &WAKER_VTABLE),
  |_| unimplemented!("wake"),
  |_| unimplemented!("wake_by_ref"),
  // Input is a mere pointer to a task. Dropped trivially.
  |_drop_me| (),
);

pub struct SleepFuture {
  wake_at: Instant,
}

impl Future for SleepFuture {
  type Output = ();
  fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
    if Instant::now() >= self.wake_at {
      Poll::Ready(())
    } else {
      Poll::Pending
    }
  }
}

pub fn sleep_for(duration: Duration) -> SleepFuture {
  SleepFuture {
    wake_at: Instant::now() + duration,
  }
}
