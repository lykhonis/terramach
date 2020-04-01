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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Cursor {
    Arrow,
    Text,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor::Arrow
    }
}

#[derive(Default)]
pub struct Cursors {}

impl Cursors {
    pub fn new() -> Self {
        Cursors {}
    }

    pub fn push(&mut self, cursor: impl Into<Option<Cursor>>) {
        unimplemented!("Cursors.push");
    }

    pub fn pop(&mut self) {
        unimplemented!("Cursors.pop");
    }
}
