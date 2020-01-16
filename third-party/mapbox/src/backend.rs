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

use std::os::raw::{c_void, c_char};
use std::ffi::CStr;
use std::io::Write;
use std::ptr::null_mut;

use crate::{bindings, Size};
use crate::native::*;
use crate::bindings::terramach_RendererBackend;

struct Backend {
    extension_function_point: Box<dyn FnMut(&str) -> *const c_void>,
    framebuffer_size: Box<dyn FnMut() -> Size>,
    make_current: Box<dyn FnMut()>,
    clear_current: Box<dyn FnMut()>,
    present_current: Box<dyn FnMut()>,
}

impl NativeDrop for bindings::terramach_RendererBackend {
    fn drop(&mut self) {
        unsafe {
            bindings::terramach_RendererBackend_RendererBackend_destructor(self);
        }
    }
}

pub struct RendererBackend {
    handle: Handle<bindings::terramach_RendererBackend>,
}

impl RendererBackend {
    pub fn new<E, F, M, C, P>(
        extension_function_point: E,
        framebuffer_size: F,
        make_current: M,
        clear_current: C,
        present_current: P,
    ) -> Self where E: 'static + FnMut(&str) -> *const c_void,
                    F: 'static + FnMut() -> Size,
                    M: 'static + FnMut(),
                    C: 'static + FnMut(),
                    P: 'static + FnMut() {
        unsafe {
            let backend = Box::into_raw(Box::new(Backend {
                extension_function_point: Box::new(extension_function_point),
                framebuffer_size: Box::new(framebuffer_size),
                make_current: Box::new(make_current),
                clear_current: Box::new(clear_current),
                present_current: Box::new(present_current),
            }));
            RendererBackend {
                handle: Handle::from_ptr(bindings::C_RendererBackend_new(bindings::terramach_Backend {
                    info: backend as *mut c_void,
                    getExtensionFunctionPointer: Some(c_extension_function_point),
                    getFramebufferSize: Some(c_framebuffer_size),
                    makeCurrent: Some(c_make_current),
                    clearCurrent: Some(c_clear_current),
                    presentCurrent: Some(c_present_current),
                    release: Some(c_release),
                })),
            }
        }
    }

    pub fn size(&self) -> Size {
        unsafe {
            self.handle.native().getSize()
        }
    }

    pub(crate) fn into_handle(self) -> Handle<bindings::terramach_RendererBackend> {
        self.handle
    }
}

impl NativeAccess<bindings::terramach_RendererBackend> for RendererBackend {
    fn native(&self) -> &bindings::terramach_RendererBackend {
        self.handle.native()
    }

    fn native_mut(&mut self) -> &mut bindings::terramach_RendererBackend {
        self.handle.native_mut()
    }
}

unsafe extern "C" fn c_extension_function_point(info: *mut c_void, name: *const c_char) -> bindings::mbgl_gl_ProcAddress {
    let backend = &mut *(info as *mut Backend);
    let name = CStr::from_ptr(name).to_str().unwrap();
    let ptr = (backend.extension_function_point)(name);
    if ptr.is_null() {
        None
    } else {
        Some(std::mem::transmute::<*const c_void, extern "C" fn()>(ptr))
    }
}

unsafe extern "C" fn c_framebuffer_size(info: *mut c_void) -> Size {
    let backend = &mut *(info as *mut Backend);
    (backend.framebuffer_size)()
}

unsafe extern "C" fn c_make_current(info: *mut c_void) {
    let backend = &mut *(info as *mut Backend);
    (backend.make_current)();
}

unsafe extern "C" fn c_clear_current(info: *mut c_void) {
    let backend = &mut *(info as *mut Backend);
    (backend.clear_current)();
}

unsafe extern "C" fn c_present_current(info: *mut c_void) {
    let backend = &mut *(info as *mut Backend);
    (backend.present_current)();
}

unsafe extern "C" fn c_release(info: *mut c_void) {
    let _ = Box::from_raw(info as *mut Backend);
}
