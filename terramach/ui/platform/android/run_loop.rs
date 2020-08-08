/*
 * Terra Mach
 * Copyright [2020] Terra Mach Authors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>
 */

use std::collections::HashMap;
use std::ops::{Add, AddAssign, DerefMut};
use std::os::raw::{c_void, c_int};
use std::ptr::null_mut;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::rc::Rc;

use crate::platform::bindings;

const DISTANT_FUTURE_SECS: f64 = 1.0e10;

#[derive(Default, Hash, Copy, Clone, Eq, PartialEq)]
pub struct RunLoopHandle(usize);

impl Add<usize> for RunLoopHandle {
    type Output = RunLoopHandle;

    fn add(self, rhs: usize) -> Self::Output {
        let (id, _) = self.0.overflowing_add(rhs);
        RunLoopHandle(id)
    }
}

impl AddAssign<usize> for RunLoopHandle {
    fn add_assign(&mut self, rhs: usize) {
        self.0 = self.0 + rhs;
    }
}

struct Fd(AtomicI32);

impl Fd {
    pub fn handle(&self) -> i32 {
        self.0.load(Ordering::Relaxed)
    }
}

impl From<i32> for Fd {
    fn from(fd: i32) -> Self {
        Self(AtomicI32::new(fd))
    }
}

impl Clone for Fd {
    fn clone(&self) -> Self {
        Self::from(self.handle())
    }
}

impl Drop for Fd {
    fn drop(&mut self) {
        let fd = self.0.swap(-1, Ordering::Relaxed);
        if fd != -1 {
            unsafe {
                assert_ne!(
                    bindings::close(fd),
                    -1, "Failed to close a file descriptor",
                );
            }
        }
    }
}

#[derive(Clone)]
struct RunLoopTimer {
    fd: Rc<Fd>,
}

impl RunLoopTimer {
    pub fn new() -> Self {
        unsafe {
            let fd = bindings::timerfd_create(bindings::CLOCK_REALTIME as i32, 0);
            assert_ne!(fd, -1, "Failed to create a system timer");
            let new_value = bindings::itimerspec {
                it_value: bindings::timespec {
                    tv_sec: DISTANT_FUTURE_SECS as i64,
                    tv_nsec: 0,
                },
                it_interval: bindings::timespec {
                    tv_sec: i64::MAX,
                    tv_nsec: 0,
                },
            };
            assert_ne!(bindings::timerfd_settime(
                fd,
                bindings::TFD_TIMER_ABSTIME as i32,
                &new_value,
                null_mut(),
            ), -1, "Failed to arm a timer");
            Self {
                fd: Rc::new(Fd::from(fd)),
            }
        }
    }

    pub fn set_next_fire_time(&mut self, time: Duration) {
        unsafe {
            let mut now = bindings::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            assert_ne!(bindings::clock_gettime(bindings::CLOCK_REALTIME as i32, &mut now), -1, "Failed to get current time");
            let new_value = bindings::itimerspec {
                it_value: bindings::timespec {
                    tv_sec: now.tv_sec + time.as_secs() as i64,
                    tv_nsec: now.tv_nsec + time.subsec_nanos() as i64,
                },
                it_interval: bindings::timespec {
                    tv_sec: i64::MAX,
                    tv_nsec: 0,
                },
            };
            assert_ne!(bindings::timerfd_settime(
                self.fd.handle(),
                bindings::TFD_TIMER_ABSTIME as i32,
                &new_value,
                null_mut(),
            ), -1, "Failed to rearm a timer");
        }
    }
}

pub struct SharedRunLoop {
    looper: *mut bindings::ALooper,
    running: Rc<AtomicBool>,
    timer: RunLoopTimer,
}

impl SharedRunLoop {
    fn new(run_loop: &RunLoop) -> Self {
        unsafe {
            bindings::ALooper_acquire(run_loop.inner);
        }
        Self {
            looper: run_loop.inner,
            running: run_loop.running.clone(),
            timer: run_loop.timer.clone(),
        }
    }

    pub fn wakeup(&mut self) {
        unsafe {
            bindings::ALooper_wake(self.looper);
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.wakeup();
    }

    pub fn set_next_wakeup_in(&mut self, delay: Duration) {
        self.timer.set_next_fire_time(delay);
    }
}

impl Clone for SharedRunLoop {
    fn clone(&self) -> Self {
        unsafe {
            bindings::ALooper_acquire(self.looper);
        }
        Self {
            looper: self.looper,
            running: self.running.clone(),
            timer: self.timer.clone(),
        }
    }
}

impl Drop for SharedRunLoop {
    fn drop(&mut self) {
        unsafe {
            bindings::ALooper_release(self.looper);
        }
    }
}

unsafe impl Send for SharedRunLoop {}

pub struct RunLoop {
    inner: *mut bindings::ALooper,
    running: Rc<AtomicBool>,
    last_handle: RunLoopHandle,
    observers: HashMap<RunLoopHandle, Box<dyn FnMut()>>,
    timer: RunLoopTimer,
}

impl RunLoop {
    pub fn main() -> Self {
        unsafe {
            let looper = bindings::ALooper_forThread();
            assert!(!looper.is_null(), "Looper is not available on current thread");
            Self::from_looper(looper)
        }
    }

    pub fn new() -> Self {
        unsafe {
            let looper = bindings::ALooper_prepare(bindings::ALOOPER_PREPARE_ALLOW_NON_CALLBACKS as c_int);
            assert!(!looper.is_null(), "Failed to prepare a looper on current thread");
            Self::from_looper(looper)
        }
    }

    fn from_looper(looper: *mut bindings::ALooper) -> Self {
        unsafe {
            bindings::ALooper_acquire(looper);
            let timer = RunLoopTimer::new();
            assert_ne!(
                bindings::ALooper_addFd(
                    looper,
                    timer.fd.handle(),
                    0,
                    bindings::ALOOPER_EVENT_INPUT as i32,
                    None,
                    null_mut(),
                ),
                -1, "Failed to add a timer",
            );
            Self {
                inner: looper,
                running: Rc::new(AtomicBool::new(false)),
                last_handle: RunLoopHandle::default(),
                observers: HashMap::new(),
                timer,
            }
        }
    }

    pub fn share(&self) -> SharedRunLoop {
        SharedRunLoop::new(self)
    }

    pub fn add_observer<F: 'static>(&mut self, observer: F) -> RunLoopHandle where F: FnMut() {
        self.last_handle += 1;
        let observer = Box::new(observer);
        self.observers.insert(self.last_handle, observer);
        self.last_handle
    }

    pub fn remove_observer(&mut self, observer: impl AsRef<RunLoopHandle>) {
        let _ = self.observers.remove(observer.as_ref());
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    fn set_running(&mut self, running: bool) {
        self.running.store(running, Ordering::Relaxed);
    }

    pub fn run(&mut self) {
        self.set_running(true);
        while self.is_running() {
            for observer in self.observers.values_mut() {
                (observer)();
            }
            let result = unsafe {
                bindings::ALooper_pollAll(
                    -1 /*indefinite*/,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                )
            };
            match result {
                bindings::ALOOPER_POLL_CALLBACK |
                bindings::ALOOPER_POLL_TIMEOUT |
                bindings::ALOOPER_POLL_WAKE => {}
                _ => self.set_running(false),
            }
        }
    }

    pub fn stop(&mut self) {
        self.set_running(false);
        unsafe {
            bindings::ALooper_wake(self.inner);
        }
    }
}

impl Drop for RunLoop {
    fn drop(&mut self) {
        unsafe {
            assert_ne!(
                bindings::ALooper_removeFd(self.inner, self.timer.fd.handle()),
                -1, "Failed to remove timer",
            );
            bindings::ALooper_release(self.inner);
        }
    }
}
