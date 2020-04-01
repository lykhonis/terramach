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

use terramach_graphics::Point;

pub type TouchId = u8;

#[derive(Debug, Copy, Clone)]
pub struct Touch {
    id: TouchId,
    location: Point,
}

impl Touch {
    pub fn new(id: TouchId, location: impl Into<Point>) -> Self {
        Touch {
            id,
            location: location.into(),
        }
    }

    pub fn id(&self) -> TouchId {
        self.id
    }

    pub fn location(&self) -> Point {
        self.location
    }
}

#[derive(Debug, Clone)]
pub struct Touches {
    touches: HashMap<TouchId, Touch>,
}

impl Touches {
    pub fn new() -> Self {
        Touches {
            touches: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.touches.len()
    }

    pub fn update(&mut self, touch: Touch) {
        self.touches.insert(touch.id(), touch);
    }

    pub fn remove(&mut self, id: TouchId) {
        self.touches.remove(&id);
    }

    pub fn is_empty(&self) -> bool {
        self.touches.is_empty()
    }

    pub fn ids(&self) -> Vec<TouchId> {
        self.touches.keys().copied().collect()
    }

    pub fn get(&self, id: TouchId) -> Option<&Touch> {
        self.touches.get(&id)
    }

    pub fn all(&self) -> Vec<&Touch> {
        self.touches.values().collect()
    }
}

struct ActiveTouch {
    touch: Touch,
    active: bool,
}

pub struct TouchTracker {
    touches: HashMap<TouchId, ActiveTouch>,
    current: Option<TouchId>,
    offset: Option<Point>,
}

impl TouchTracker {
    pub fn new() -> Self {
        TouchTracker {
            touches: HashMap::new(),
            current: None,
            offset: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.touches.is_empty()
    }

    pub fn reset(&mut self) {
        self.touches.clear();
        self.current = None;
        self.offset = None;
    }

    pub fn begin_touch(&mut self, id: TouchId) -> Option<Touch> {
        if let Some(touch) = self.touches.get_mut(&id) {
            touch.active = false;
        } else {
            self.touches.insert(
                id,
                ActiveTouch {
                    touch: Touch::new(id, Point::default()),
                    active: false,
                },
            );
        }
        self.current = Some(id);
        if let Some(offset) = self.offset {
            let touch = self.touches.get_mut(&id)?;
            touch.active = true;
            touch.touch.location = offset;
            Some(touch.touch)
        } else {
            None
        }
    }

    pub fn begin_touch_with_offset(&mut self, id: TouchId, offset: impl Into<Point>) -> Touch {
        if !self.touches.contains_key(&id) {
            self.touches.insert(
                id,
                ActiveTouch {
                    touch: Touch::new(id, Point::default()),
                    active: false,
                },
            );
        }
        self.current = Some(id);
        let touch = self.touches.get_mut(&id).unwrap();
        touch.active = true;
        touch.touch.location = offset.into();
        touch.touch
    }

    pub fn end_touch(&mut self, id: TouchId) -> Option<Touch> {
        if self.current == Some(id) {
            self.current = None;
        }
        let touch = self.touches.remove(&id)?;
        if touch.active {
            Some(touch.touch)
        } else {
            None
        }
    }

    pub fn push_offset(&mut self, offset: impl Into<Point>) -> Option<Touch> {
        let offset = offset.into();
        self.offset = Some(offset);
        let touch = self.touches.get_mut(&self.current?)?;
        touch.active = true;
        touch.touch.location = offset;
        Some(touch.touch)
    }

    pub fn touch(&self, id: TouchId) -> Option<Touch> {
        let touch = self.touches.get(&id)?;
        if touch.active {
            Some(touch.touch)
        } else {
            None
        }
    }
}
