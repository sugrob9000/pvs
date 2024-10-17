//! The simplest executor I was able to come up with that works with async Rust and
//! requires zero dynamic memory allocation.
//! - Inplace buffer for fixed number of tasks
//! - No spawner; all tasks must have been spawned before running
//! - Only timer supported just yet. This timer is always eagerly polled.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::{Duration, Instant};

pub type PinnedFuture<'t> = Pin<&'t mut dyn Future<Output = ()>>;

const MAX_TASKS: usize = 4;

pub struct Executor<'t> {
  task_slots: [Option<Task<'t>>; MAX_TASKS],
}

struct Task<'t> {
  future: PinnedFuture<'t>,
}

impl<'t> Executor<'t> {
  pub fn new() -> Self {
    Self {
      // None in all slots - no tasks yet
      task_slots: Default::default(),
    }
  }

  pub fn with_task(mut self, task: PinnedFuture<'t>) -> Self {
    for task_slot in self.task_slots.iter_mut() {
      if task_slot.is_none() {
        *task_slot = Some(Task { future: task });
        return self;
      }
    }
    panic!("Nowhere to push this task: max {MAX_TASKS}");
  }

  pub fn run(mut self) {
    loop {
      let mut any_tasks = false;

      for task_slot in self.task_slots.iter_mut() {
        if let Some(task) = task_slot {
          any_tasks = true;

          let waker = unsafe {
            // SAFETY: TODO ensure it's OK to create this waker (besides the fact that
            // we never wake anything yet). Probably should pin both self and task.
            Self::make_waker(task as *const Task)
          };

          let mut context = Context::from_waker(&waker);

          match task.future.as_mut().poll(&mut context) {
            Poll::Ready(()) => *task_slot = None,
            Poll::Pending => continue,
          }
        }
      }

      if !any_tasks {
        break;
      }
    }
  }

  unsafe fn make_waker(task: *const Task) -> Waker {
    let raw = RawWaker::new(task as *const (), &WAKER_VTABLE);
    Waker::from_raw(raw)
  }
}

static WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
  // Input is a mere pointer to a task. Cloned trivially.
  |clone_me| RawWaker::new(clone_me, &WAKER_VTABLE),
  |_| todo!("wake"),
  |_| todo!("wake_by_ref"),
  // Input is a mere pointer to a task. Dropped trivially.
  |_drop_me| (),
);

pub fn sleep_for(duration: Duration) -> SleepFuture {
  SleepFuture {
    wake_at: Instant::now() + duration,
  }
}

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
