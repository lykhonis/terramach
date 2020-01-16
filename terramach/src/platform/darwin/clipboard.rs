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

#![allow(non_upper_case_globals)]

use std::ptr::null_mut;

use objc_id::Id;
use objc_foundation::{NSData, INSData, NSArray, INSArray, NSString, INSString, INSCopying};
use objc::runtime::{Object, BOOL, YES};

const NSStringPboardType: &str = "NSStringPboardType";

#[derive(Debug)]
pub struct ClipboardContent {
    data: Id<NSData>,
    kind: String,
}

impl ClipboardContent {
    pub fn to_string(&self) -> Option<String> {
        match self.kind.as_str() {
            NSStringPboardType => {
                String::from_utf8_lossy(self.data.bytes()).to_string().into()
            }
            _ => None,
        }
    }
}

impl From<String> for ClipboardContent {
    fn from(data: String) -> Self {
        ClipboardContent {
            data: NSData::with_bytes(data.as_bytes()),
            kind: NSStringPboardType.to_string(),
        }
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

pub struct Clipboard {
    pasteboard: Id<Object>,
}

impl Clipboard {
    fn new() -> Self {
        unsafe {
            Clipboard {
                pasteboard: Id::from_ptr(msg_send![class!(NSPasteboard), generalPasteboard]),
            }
        }
    }

    pub fn clear_content(&mut self) {
        unsafe {
            let () = msg_send![self.pasteboard, clearContents];
        }
    }

    pub fn set_content(&mut self, content: impl Into<ClipboardContent>) -> bool {
        let content = content.into();
        unsafe {
            let kind = NSString::from_str(&content.kind);
            let types = NSArray::from_vec(vec![kind.copy()]);
            let owner: *mut Object = null_mut();
            let () = msg_send![self.pasteboard, declareTypes:types owner:owner];
            let result: BOOL = msg_send![self.pasteboard, setData:content.data forType:kind];
            result == YES
        }
    }

    pub fn content(&self) -> Option<ClipboardContent> {
        unsafe {
            let types = NSArray::from_vec(vec![
                NSString::from_str(NSStringPboardType),
            ]);
            let kind: *mut NSString = msg_send![self.pasteboard, availableTypeFromArray:types];
            if kind.is_null() { return None; }
            let data: *mut NSData = msg_send![self.pasteboard, dataForType:kind];
            if data.is_null() { return None; }
            let kind: Id<NSString> = Id::from_ptr(kind);
            let data = Id::from_ptr(data);
            match kind.as_str() {
                NSStringPboardType => {
                    Some(ClipboardContent {
                        data,
                        kind: kind.as_str().to_string(),
                    })
                }
                _ => None
            }
        }
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Clipboard::new()
    }
}
