/*
 * Terra Mach
 * Copyright [2020] Volodymyr Lykhonis
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
use std::os::raw::c_void;
use std::ptr::null_mut;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use core_foundation::base::{kCFAllocatorDefault, CFRelease, CFRetain};
use core_foundation::date::CFAbsoluteTimeGetCurrent;
use core_foundation::runloop::*;

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

struct RunLoopObserver {
    inner: CFRunLoopObserverRef,
    callback: *mut Box<dyn FnMut()>,
}

extern "C" fn run_loop_observer_callback(
    _observer: CFRunLoopObserverRef,
    _activity: CFRunLoopActivity,
    info: *mut c_void,
) {
    unsafe {
        let callback = &mut *(info as *mut Box<dyn FnMut()>);
        (callback)();
    }
}

impl RunLoopObserver {
    fn new<F: 'static>(callback: F) -> Self where F: FnMut() {
        unsafe {
            let callback = Box::into_raw(Box::new(Box::new(callback) as Box<dyn FnMut()>));
            let mut context = CFRunLoopObserverContext {
                version: 0,
                info: callback as *mut c_void,
                retain: None,
                release: None,
                copyDescription: None,
            };
            let observer = CFRunLoopObserverCreate(
                kCFAllocatorDefault,
                kCFRunLoopBeforeWaiting,
                1, /*YES*/
                0,
                run_loop_observer_callback,
                &mut context,
            );
            RunLoopObserver {
                inner: observer,
                callback,
            }
        }
    }
}

impl Drop for RunLoopObserver {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.inner as *const c_void);
            let _ = Box::from_raw(self.callback);
        }
    }
}

struct RunLoopSource {
    inner: CFRunLoopSourceRef,
}

extern "C" fn run_loop_source_callback(_info: *const c_void) {}

impl RunLoopSource {
    pub fn new() -> Self {
        unsafe {
            let mut context = CFRunLoopSourceContext {
                version: 0,
                info: null_mut() as *mut c_void,
                retain: None,
                release: None,
                copyDescription: None,
                equal: None,
                hash: None,
                schedule: None,
                cancel: None,
                perform: run_loop_source_callback,
            };
            let source = CFRunLoopSourceCreate(kCFAllocatorDefault, 0, &mut context);
            RunLoopSource { inner: source }
        }
    }

    pub fn signal(&mut self) {
        unsafe {
            CFRunLoopSourceSignal(self.inner);
        }
    }
}

impl Drop for RunLoopSource {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.inner as *const c_void);
        }
    }
}

struct RunLoopTimer {
    inner: CFRunLoopTimerRef,
}

extern "C" fn run_loop_timer_callback(_timer: CFRunLoopTimerRef, _info: *mut c_void) {}

impl RunLoopTimer {
    pub fn new() -> Self {
        unsafe {
            let timer = CFRunLoopTimerCreate(
                kCFAllocatorDefault,
                DISTANT_FUTURE_SECS,
                std::f64::MAX,
                0,
                0,
                run_loop_timer_callback,
                null_mut(),
            );
            RunLoopTimer { inner: timer }
        }
    }

    pub fn set_next_fire_time(&mut self, time: Duration) {
        unsafe {
            let time = CFAbsoluteTimeGetCurrent() + time.as_secs_f64();
            CFRunLoopTimerSetNextFireDate(self.inner, time);
        }
    }
}

impl Drop for RunLoopTimer {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.inner as *const c_void);
        }
    }
}

impl Clone for RunLoopTimer {
    fn clone(&self) -> Self {
        unsafe {
            CFRetain(self.inner as *const c_void);
            RunLoopTimer { inner: self.inner }
        }
    }
}

pub struct SharedRunLoop {
    running: Arc<Mutex<bool>>,
    run_loop: CFRunLoopRef,
    timer: RunLoopTimer,
}

impl SharedRunLoop {
    fn new(run_loop: &RunLoop) -> Self {
        unsafe {
            CFRetain(run_loop.inner as *const c_void);
            SharedRunLoop {
                running: run_loop.running.clone(),
                run_loop: run_loop.inner,
                timer: run_loop.timer.clone(),
            }
        }
    }

    pub fn wakeup(&mut self) {
        unsafe {
            CFRunLoopWakeUp(self.run_loop);
        }
    }

    pub fn stop(&mut self) {
        if let Ok(mut running) = self.running.lock() {
            *running.deref_mut() = false;
        }
        unsafe {
            CFRunLoopStop(self.run_loop);
        }
    }

    pub fn set_next_wakeup_in(&mut self, delay: Duration) {
        self.timer.set_next_fire_time(delay);
    }
}

impl Clone for SharedRunLoop {
    fn clone(&self) -> Self {
        unsafe {
            CFRetain(self.run_loop as *const c_void);
            SharedRunLoop {
                running: self.running.clone(),
                run_loop: self.run_loop,
                timer: self.timer.clone(),
            }
        }
    }
}

impl Drop for SharedRunLoop {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.run_loop as *const c_void);
        }
    }
}

unsafe impl Send for SharedRunLoop {}

pub struct RunLoop {
    inner: CFRunLoopRef,
    running: Arc<Mutex<bool>>,
    last_handle: RunLoopHandle,
    observers: HashMap<RunLoopHandle, RunLoopObserver>,
    timer: RunLoopTimer,
}

impl RunLoop {
    pub fn main() -> Self {
        unsafe { RunLoop::from_run_loop(CFRunLoopGetMain()) }
    }

    pub fn new() -> Self {
        unsafe { RunLoop::from_run_loop(CFRunLoopGetCurrent()) }
    }

    fn from_run_loop(run_loop: CFRunLoopRef) -> Self {
        unsafe {
            CFRetain(run_loop as *const c_void);
            let timer = RunLoopTimer::new();
            CFRunLoopAddTimer(run_loop, timer.inner, kCFRunLoopCommonModes);
            RunLoop {
                inner: run_loop,
                running: Arc::new(Mutex::new(false)),
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
        let observer = RunLoopObserver::new(observer);
        unsafe {
            CFRunLoopAddObserver(self.inner, observer.inner, kCFRunLoopCommonModes);
        }
        self.observers.insert(self.last_handle, observer);
        self.last_handle
    }

    pub fn remove_observer(&mut self, observer: impl AsRef<RunLoopHandle>) {
        if let Some(observer) = self.observers.remove(observer.as_ref()) {
            unsafe {
                CFRunLoopRemoveObserver(self.inner, observer.inner, kCFRunLoopCommonModes);
            }
        }
    }

    pub fn is_running(&self) -> bool {
        if let Ok(running) = self.running.lock() {
            *running
        } else {
            false
        }
    }

    fn set_running(&mut self, running: bool) {
        if let Ok(mut mutext) = self.running.lock() {
            *mutext.deref_mut() = running;
        }
    }

    pub fn run(&mut self) {
        self.set_running(true);
        while self.is_running() {
            let result = unsafe {
                CFRunLoopRunInMode(kCFRunLoopDefaultMode, DISTANT_FUTURE_SECS, 1 /*YES*/)
            };
            if result == kCFRunLoopRunFinished {
                self.set_running(false);
            }
        }
    }

    pub fn stop(&mut self) {
        self.set_running(false);
        unsafe {
            CFRunLoopStop(self.inner);
        }
    }
}

impl Drop for RunLoop {
    fn drop(&mut self) {
        unsafe {
            CFRunLoopRemoveTimer(self.inner, self.timer.inner, kCFRunLoopCommonModes);
            for observer in self.observers.values() {
                CFRunLoopRemoveObserver(self.inner, observer.inner, kCFRunLoopCommonModes);
            }
            CFRelease(self.inner as *const c_void);
        }
    }
}
