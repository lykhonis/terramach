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

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::os::raw::{c_long, c_void};

use lazy_static::lazy_static;
use time_point::TimePoint;

use crate::platform::bindings;
use std::time::Instant;

lazy_static! {
    static ref SHARED: VSync = VSync::new().expect("Failed to initialize VSync!");
}

unsafe extern "C" fn choreographer_frame<F>(frame_time_nanos: c_long, data: *mut c_void)
    where F: FnMut(TimePoint) {
    let timestamp = TimePoint::new(frame_time_nanos);
    let mut f = Box::from_raw(data as *mut F);
    f(timestamp)
}

#[derive(Clone)]
pub struct VSync {
    choreographer: *mut bindings::AChoreographer,
}

impl VSync {
    pub fn new() -> Option<Self> {
        let choreographer = unsafe {
            bindings::AChoreographer_getInstance()
        };
        if choreographer.is_null() {
            return None;
        }
        Some(VSync {
            choreographer,
        })
    }

    pub fn wait(&mut self) -> Option<TimePoint> {
        let (tx, rx) = channel();
        self.request_frame(move |timestamp| {
            let _ = tx.send(timestamp);
        });
        rx.recv().ok()
    }

    pub fn request_frame<F>(&mut self, callback: F) where F: 'static + Send + FnMut(TimePoint) {
        let data = Box::into_raw(Box::new(callback));
        unsafe {
            bindings::AChoreographer_postFrameCallback(
                self.choreographer,
                Some(choreographer_frame::<F>),
                data as *mut c_void,
            );
        }
    }
}

unsafe impl Sync for VSync {}

impl Default for VSync {
    fn default() -> Self {
        SHARED.clone()
    }
}
