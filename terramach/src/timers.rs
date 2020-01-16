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
use std::time::{Duration, Instant};

use crate::{Id, IndexPool};

pub struct Timer {
    delay: Option<Duration>,
    interval: Option<Duration>,
}

impl Timer {
    pub fn new(delay: impl Into<Option<Duration>>, interval: impl Into<Option<Duration>>) -> Self {
        Timer {
            delay: delay.into(),
            interval: interval.into(),
        }
    }

    pub fn delay(&self) -> Option<Duration> {
        self.delay
    }

    pub fn interval(&self) -> Option<Duration> {
        self.interval
    }

    pub fn is_repeated(&self) -> bool {
        self.interval.is_some()
    }
}

struct ScheduledTimer {
    timer: Timer,
    timestamp: Instant,
    ongoing: bool,
}

impl ScheduledTimer {
    pub fn fire_in(&self) -> Duration {
        let elapsed = self.timestamp.elapsed();
        let delay = if self.ongoing {
            self.timer.interval
        } else {
            self.timer.delay
        }
        .unwrap_or_default();
        if elapsed <= delay {
            delay - elapsed
        } else {
            Duration::from_secs(0)
        }
    }

    pub fn is_repeated(&self) -> bool {
        self.timer.is_repeated()
    }

    pub fn fire(&mut self) -> bool {
        let elapsed = self.timestamp.elapsed();
        if self.ongoing {
            if elapsed >= self.timer.interval.unwrap_or_default() {
                self.timestamp = Instant::now();
                true
            } else {
                false
            }
        } else {
            if elapsed >= self.timer.delay.unwrap_or_default() {
                self.timestamp = Instant::now();
                self.ongoing = true;
                true
            } else {
                false
            }
        }
    }
}

impl From<Timer> for ScheduledTimer {
    fn from(timer: Timer) -> Self {
        ScheduledTimer {
            timer,
            timestamp: Instant::now(),
            ongoing: false,
        }
    }
}

pub struct Timers {
    ids: IndexPool,
    timers: HashMap<Id, ScheduledTimer>,
}

impl Timers {
    pub fn new() -> Self {
        Timers {
            ids: IndexPool::new(),
            timers: HashMap::new(),
        }
    }

    pub fn add(&mut self, timer: Timer) -> Id {
        let id = self.ids.take();
        self.timers.insert(id, ScheduledTimer::from(timer));
        id
    }

    pub fn remove(&mut self, id: Id) -> Option<Timer> {
        let scheduled = self.timers.remove(&id)?;
        self.ids.give(id);
        Some(scheduled.timer)
    }

    pub fn clear(&mut self) {
        for (id, _) in self.timers.drain() {
            self.ids.give(id);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.timers.is_empty()
    }

    pub fn next_fire_time(&self) -> Option<Duration> {
        let mut next_time = None;
        for timer in self.timers.values() {
            let fire_time = timer.fire_in();
            if let Some(time) = &next_time {
                if time < &fire_time {
                    next_time = Some(fire_time);
                }
            } else {
                next_time = Some(fire_time);
            }
        }
        next_time
    }

    pub fn fire(&mut self) -> Option<Vec<Id>> {
        if self.timers.is_empty() {
            return None;
        }
        let mut fired = Vec::new();
        let ids: Vec<Id> = self.timers.keys().copied().collect();
        for id in ids {
            let timer = self.timers.get_mut(&id)?;
            if timer.fire() {
                if !timer.is_repeated() {
                    self.remove(id);
                }
                fired.push(id);
            }
        }
        if fired.is_empty() {
            None
        } else {
            Some(fired)
        }
    }
}
