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

#[derive(Debug)]
pub struct ClipboardContent {}

impl ClipboardContent {
    pub fn to_string(&self) -> Option<String> {
        unimplemented!();
    }
}

impl From<String> for ClipboardContent {
    fn from(data: String) -> Self {
        unimplemented!();
    }
}

impl From<&str> for ClipboardContent {
    fn from(data: &str) -> Self {
        ClipboardContent::from(data.to_string())
    }
}

impl From<&String> for ClipboardContent {
    fn from(data: &String) -> Self {
        ClipboardContent::from(data.clone())
    }
}

pub struct Clipboard {}

impl Clipboard {
    fn new() -> Self {
        unimplemented!();
    }

    pub fn clear_content(&mut self) {
        unimplemented!();
    }

    pub fn set_content(&mut self, content: impl Into<ClipboardContent>) -> bool {
        let content = content.into();
        unimplemented!();
    }

    pub fn content(&self) -> Option<ClipboardContent> {
        unimplemented!();
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Clipboard::new()
    }
}
