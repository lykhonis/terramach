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

use crate::bindings;
use crate::native::*;

impl NativeDrop for bindings::terramach_Scheduler {
    fn drop(&mut self) {
        unsafe {
            bindings::terramach_Scheduler_Scheduler_destructor(self);
        }
    }
}

impl NativeAccess<bindings::terramach_Scheduler> for Scheduler {
    fn native(&self) -> &bindings::terramach_Scheduler {
        self.handle.native()
    }

    fn native_mut(&mut self) -> &mut bindings::terramach_Scheduler {
        self.handle.native_mut()
    }
}

pub struct Scheduler {
    handle: Handle<bindings::terramach_Scheduler>,
}

impl Scheduler {
    pub fn new() -> Self {
        unsafe {
            Scheduler {
                handle: Handle::from_ptr(bindings::C_Scheduler_new()),
            }
        }
    }

    pub(crate) fn into_handle(self) -> Handle<bindings::terramach_Scheduler> {
        self.handle
    }
}
