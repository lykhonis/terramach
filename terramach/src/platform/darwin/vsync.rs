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

use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use display_link::DisplayLink;
use lazy_static::lazy_static;
use time_point::TimePoint;

lazy_static! {
    static ref SHARED: VSync = VSync::new().expect("Failed to initialize VSync!");
}

#[derive(Clone)]
pub struct VSync {
    link: Arc<DisplayLink>,
    callbacks: Arc<Mutex<Vec<Box<dyn Send + FnMut(TimePoint)>>>>,
}

impl VSync {
    pub fn new() -> Option<Self> {
        let callbacks = Arc::new(Mutex::new(Vec::new()));
        let link_callbacks = callbacks.clone();
        let mut link = DisplayLink::new(move |timestamp| {
            let callbacks: Vec<Box<dyn Send + FnMut(TimePoint)>> =
                if let Ok(mut callbacks) = link_callbacks.lock() {
                    callbacks.drain(..).collect()
                } else {
                    return;
                };
            for mut callback in callbacks {
                callback(timestamp);
            }
        })?;
        if link.is_paused() {
            link.resume().ok()?;
        }
        Some(VSync {
            link: Arc::new(link),
            callbacks,
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
        if let Ok(mut callbacks) = self.callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }
}

unsafe impl Sync for VSync {}

impl Default for VSync {
    fn default() -> Self {
        SHARED.clone()
    }
}
