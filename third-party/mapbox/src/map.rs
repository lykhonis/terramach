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

use std::ffi::CString;
use std::path::Path;

use crate::{bindings, RendererFrontend, Size, ResourceOptions, Scheduler};
use crate::native::*;
use crate::prelude::*;

impl NativeDrop for bindings::mbgl_MapOptions {
    fn drop(&mut self) {
        unsafe {
            bindings::mbgl_MapOptions_MapOptions_destructor(self);
        }
    }
}

pub struct MapOptions {
    handle: Handle<bindings::mbgl_MapOptions>,
}

impl MapOptions {
    pub fn new() -> Self {
        unsafe {
            Self {
                handle: Handle::new_native(bindings::mbgl_MapOptions_MapOptions)
            }
        }
    }

    pub fn with_size(mut self, size: Size) -> Self {
        unsafe {
            self.handle.native_mut().withSize(size);
        }
        self
    }

    pub fn with_pixel_ratio(mut self, ratio: f32) -> Self {
        unsafe {
            self.handle.native_mut().withPixelRatio(ratio);
        }
        self
    }
}

impl Default for MapOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeAccess<bindings::mbgl_MapOptions> for MapOptions {
    fn native(&self) -> &bindings::mbgl_MapOptions {
        self.handle.native()
    }

    fn native_mut(&mut self) -> &mut bindings::mbgl_MapOptions {
        self.handle.native_mut()
    }
}

impl NativeDrop for bindings::terramach_Map {
    fn drop(&mut self) {
        unsafe {
            bindings::terramach_Map_Map_destructor(self);
        }
    }
}

pub struct Map {
    handle: Handle<bindings::terramach_Map>,
}

impl Map {
    pub fn new(
        frontend: RendererFrontend,
        options: &MapOptions,
        resource_options: &ResourceOptions,
    ) -> Self {
        unsafe {
            Self {
                handle: Handle::from_ptr(bindings::C_Map_new(
                    Scheduler::new().into_handle().into_ptr(),
                    frontend.into_handle().into_ptr(),
                    options.native(),
                    resource_options.native(),
                )),
            }
        }
    }

    pub fn render(&mut self) {
        unsafe {
            self.handle.native_mut().render();
        }
    }

    pub fn size(&self) -> Size {
        unsafe {
            let mut options = self.handle.native().getMapOptions();
            let size = options.size();
            options.destruct();
            size
        }
    }

    pub fn set_size(&mut self, size: impl Into<Size>) {
        unsafe {
            self.handle.native_mut().setSize(size.into());
        }
    }

    pub fn jump_to(&mut self, camera: &CameraOptions) {
        unsafe {
            self.handle.native_mut().jumpTo(camera.native());
        }
    }

    pub fn ease_to(&mut self, camera: &CameraOptions, animation: &AnimationOptions) {
        unsafe {
            self.handle.native_mut().easeTo(camera.native(), animation.native());
        }
    }

    pub fn move_by<'a>(
        &mut self,
        coordinate: impl Into<ScreenCoordinate>,
        animation: impl Into<Option<&'a AnimationOptions>>,
    ) {
        unsafe {
            self.handle.native_mut().moveBy(
                &mut coordinate.into().into(),
                animation.into().ptr_or_null(),
            );
        }
    }

    pub fn scale_by<'a>(
        &mut self,
        scale: f32,
        anchor: impl Into<Option<ScreenCoordinate>>,
        animation: impl Into<Option<&'a AnimationOptions>>,
    ) {
        unsafe {
            self.handle.native_mut().scaleBy(
                scale as f64,
                anchor.into().ptr_or_null(),
                animation.into().ptr_or_null(),
            );
        }
    }
}
