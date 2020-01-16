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

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use time_point::TimePoint;

use terramach_graphics::{Point, Size};

use crate::{Id, Touch, Touches, HitKey};

pub type EventId = u8;

#[derive(Debug, Clone)]
pub enum Event {
    TouchBegin(Touch),
    TouchUpdate(Touch),
    TouchEnd(Touch),
    Touch(Touches),
    Scroll(Point),
    Hover(Point),
    Enter,
    Leave,
    Frame(TimePoint),
    Timer(Id),
    Tap(EventId),
    Key(HitKey),
    Focus(bool),
    BecameResponder,
    ResignedResponder,
}

pub type WidgetEvents = Events<Event>;
pub type WidgetEventEmitter = EventEmitter<Event>;

pub struct EventContext {
    need_paint: bool,
    need_layout: bool,
    need_build: bool,
    need_event: bool,
    event: Event,
}

impl EventContext {
    pub fn new(event: Event) -> Self {
        EventContext {
            need_layout: false,
            need_paint: false,
            need_build: false,
            need_event: false,
            event,
        }
    }

    pub fn get(&self) -> &Event {
        &self.event
    }

    pub fn need_paint(&self) -> bool {
        self.need_paint
    }

    pub fn need_layout(&self) -> bool {
        self.need_layout
    }

    pub fn need_build(&self) -> bool {
        self.need_build
    }

    pub fn need_event(&self) -> bool {
        self.need_event
    }

    pub fn mark_need_paint(&mut self) {
        self.need_paint = true;
    }

    pub fn mark_need_layout(&mut self) {
        self.need_layout = true;
    }

    pub fn mark_need_build(&mut self) {
        self.need_build = true;
    }

    pub fn mark_need_event(&mut self) {
        self.need_event = true;
    }
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Quit,
    Focus(bool),
    Resize(Size),
    TouchBegin(Touch),
    TouchUpdate(Touch),
    TouchEnd(Touch),
    Scroll(Point),
    Hover(Point),
    Frame(TimePoint),
    Key(HitKey),
}

pub type AppEvents = Events<AppEvent>;

type EventQueue<T> = Arc<Mutex<VecDeque<T>>>;

#[derive(Default)]
pub struct Events<T> {
    queue: EventQueue<T>,
}

impl<T> Events<T> {
    pub fn new() -> Self {
        Events {
            queue: Default::default(),
        }
    }

    pub fn emitter(&mut self) -> EventEmitter<T> {
        EventEmitter {
            queue: self.queue.clone(),
        }
    }

    pub fn poll(&mut self) -> Option<Vec<T>> {
        if let Ok(mut queue) = self.queue.lock() {
            if queue.is_empty() {
                None
            } else {
                Some(queue.drain(..).collect())
            }
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct EventEmitter<T> {
    queue: EventQueue<T>,
}

impl<T> EventEmitter<T> {
    pub fn emit_event(&mut self, event: T) {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push_back(event);
        }
    }

    pub fn is_empty(&self) -> bool {
        if let Ok(queue) = self.queue.lock() {
            queue.is_empty()
        } else {
            true
        }
    }
}
