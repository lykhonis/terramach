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

use std::os::raw::c_void;

use crate::{bindings, Size, RendererBackend};
use crate::native::*;

struct Frontend {
    pixel_ratio: f32,
    invalidate: Box<dyn FnMut()>,
}

impl NativeDrop for bindings::terramach_RendererFrontend {
    fn drop(&mut self) {
        unsafe {
            bindings::terramach_RendererFrontend_RendererFrontend_destructor(self);
        }
    }
}

pub struct RendererFrontend {
    pixel_ratio: f32,
    handle: Handle<bindings::terramach_RendererFrontend>,
}

impl RendererFrontend {
    pub fn new<I>(
        backend: RendererBackend,
        pixel_ratio: f32,
        invalidate: I,
    ) -> Self where I: 'static + FnMut() {
        unsafe {
            let frontend = Box::into_raw(Box::new(Frontend {
                pixel_ratio,
                invalidate: Box::new(invalidate),
            }));
            RendererFrontend {
                pixel_ratio,
                handle: Handle::from_ptr(bindings::C_RendererFrontend_new(
                    backend.into_handle().into_ptr(),
                    bindings::terramach_Frontend {
                        info: frontend as *mut c_void,
                        pixelRatio: Some(c_pixel_ratio),
                        invalidate: Some(c_invalidate),
                        release: Some(c_release),
                    },
                )),
            }
        }
    }

    pub fn pixel_ratio(&self) -> f32 {
        self.pixel_ratio
    }

    pub fn render(&mut self) {
        unsafe {
            self.handle.native_mut().render();
        }
    }

    pub(crate) fn into_handle(self) -> Handle<bindings::terramach_RendererFrontend> {
        self.handle
    }
}

impl NativeAccess<bindings::terramach_RendererFrontend> for RendererFrontend {
    fn native(&self) -> &bindings::terramach_RendererFrontend {
        self.handle.native()
    }

    fn native_mut(&mut self) -> &mut bindings::terramach_RendererFrontend {
        self.handle.native_mut()
    }
}

unsafe extern "C" fn c_pixel_ratio(info: *mut c_void) -> f32 {
    let frontend = &mut *(info as *mut Frontend);
    frontend.pixel_ratio
}

unsafe extern "C" fn c_invalidate(info: *mut c_void) {
    let frontend = &mut *(info as *mut Frontend);
    (frontend.invalidate)();
}

unsafe extern "C" fn c_release(info: *mut c_void) {
    let _ = Box::from_raw(info as *mut Frontend);
}
