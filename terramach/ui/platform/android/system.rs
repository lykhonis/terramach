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

use jni::JNIEnv;
use jni::objects::JClass;

#[macro_export]
macro_rules! console_write {
    ($bytes:expr) => {
        $crate::platform::console::write($bytes).expect("Failed to write to a system console");
    }
}

#[macro_export]
macro_rules! console_flush {
    () => {
        $crate::platform::console::flush().expect("Failed to flush a system console");
    }
}

#[macro_export]
macro_rules! console_print {
    ($($args:tt)*) => ($crate::console_write!(std::format!($($args)*).as_bytes()))
}

#[macro_export]
macro_rules! console_println {
    () => {
        $crate::console_print!("\n");
        $crate::console_flush!();
    };
    ($($args:tt)*) => {
        $crate::console_print!($($args)*);
        $crate::console_println!();
    }
}

pub mod console {
    use crate::platform::bindings;
    use std::io;
    use std::ffi::CString;
    use std::os::raw::c_char;

    pub fn write(bytes: &[u8]) -> io::Result<()> {
        let tag = CString::new("TerraMach").unwrap();
        let text = CString::new(bytes).unwrap();
        unsafe {
            bindings::__android_log_write(
                bindings::android_LogPriority_ANDROID_LOG_INFO as i32,
                tag.as_ptr(),
                text.as_ptr(),
            );
        }
        Ok(())
    }

    pub fn flush() -> io::Result<()> {
        Ok(())
    }
}

#[inline]
#[allow(non_snake_case)]
pub fn Java_com_terramach_System_initialize(_: JNIEnv<'static>, _: JClass) {
    std::panic::set_hook(Box::new(|info| {
        console_println!("*****************************************************************");
        console_println!("");
        console_println!("Terra Mach Internal Error");
        if let Some(message) = info.message() {
            console_println!("");
            console_println!("{}", message);
        }
        if let Some(location) = info.location() {
            console_println!("");
            console_println!("{}:{}:{}", location.file(), location.line(), location.column());
        }
        console_println!("");
        console_println!("*****************************************************************");
    }));
}
