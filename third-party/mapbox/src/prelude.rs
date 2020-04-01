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
use std::time::Duration;
use std::marker::PhantomData;

use crate::bindings;
use crate::native::*;

pub type Size = bindings::mbgl_Size;

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Size {
            width,
            height,
        }
    }
}

impl From<(u32, u32)> for Size {
    fn from(size: (u32, u32)) -> Self {
        Size::new(size.0, size.1)
    }
}

impl NativeDrop for bindings::mbgl_ResourceOptions {
    fn drop(&mut self) {
        unsafe {
            bindings::mbgl_ResourceOptions_ResourceOptions_destructor(self);
        }
    }
}

pub struct ResourceOptions {
    handle: Handle<bindings::mbgl_ResourceOptions>,
}

impl ResourceOptions {
    pub fn new() -> Self {
        unsafe {
            ResourceOptions {
                handle: Handle::new_native(bindings::mbgl_ResourceOptions_ResourceOptions),
            }
        }
    }

    pub fn with_cache_path(mut self, path: impl AsRef<Path>) -> Self {
        unsafe {
            let path = path.as_ref().to_str().unwrap();
            let path = CString::new(path).unwrap();
            bindings::C_mbgl_ResourceOptions_withCachePath(self.handle.native_mut(), path.as_ptr());
        }
        self
    }

    pub fn with_access_token(mut self, token: impl AsRef<str>) -> Self {
        unsafe {
            let token = CString::new(token.as_ref()).unwrap();
            bindings::C_mbgl_ResourceOptions_withAccessToken(self.handle.native_mut(), token.as_ptr());
        }
        self
    }
}

impl Default for ResourceOptions {
    fn default() -> Self {
        ResourceOptions::new()
    }
}

impl NativeAccess<bindings::mbgl_ResourceOptions> for ResourceOptions {
    fn native(&self) -> &bindings::mbgl_ResourceOptions {
        self.handle.native()
    }

    fn native_mut(&mut self) -> &mut bindings::mbgl_ResourceOptions {
        self.handle.native_mut()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LatLng(bindings::mbgl_LatLng);

impl LatLng {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        LatLng(bindings::mbgl_LatLng {
            lat: latitude,
            lon: longitude,
        })
    }

    pub fn latitude(&self) -> f64 { self.0.lat }

    pub fn longitude(&self) -> f64 { self.0.lon }
}

impl Default for LatLng {
    fn default() -> Self {
        LatLng::new(0.0, 0.0)
    }
}

impl From<(f64, f64)> for LatLng {
    fn from(values: (f64, f64)) -> Self {
        LatLng::new(values.0, values.1)
    }
}

impl Into<bindings::mbgl_LatLng> for LatLng {
    fn into(self) -> bindings::mbgl_LatLng {
        self.0
    }
}

impl NativeDrop for bindings::mbgl_CameraOptions {
    fn drop(&mut self) {
        unsafe {
            bindings::C_CameraOptions_delete(self);
        }
    }
}

impl NativeCopy<bindings::mbgl_CameraOptions> for bindings::mbgl_CameraOptions {
    fn copy_to(&self, dst: *mut bindings::mbgl_CameraOptions) {
        unsafe {
            std::ptr::copy(self, dst, 1);
        }
    }
}

impl PartialEq<bindings::mbgl_CameraOptions> for bindings::mbgl_CameraOptions {
    fn eq(&self, other: &bindings::mbgl_CameraOptions) -> bool {
        self.zoom == other.zoom &&
            self.center == other.center &&
            self.anchor == other.anchor &&
            self.bearing == other.bearing &&
            self.padding == other.padding &&
            self.pitch == other.pitch
    }
}

#[derive(Clone, PartialEq)]
pub struct CameraOptions {
    handle: Handle<bindings::mbgl_CameraOptions>,
}

impl CameraOptions {
    pub fn new() -> Self {
        unsafe {
            CameraOptions {
                handle: Handle::from_ptr(bindings::C_CameraOptions_new()),
            }
        }
    }

    pub fn with_center(mut self, center: impl Into<LatLng>) -> Self {
        unsafe {
            bindings::C_CameraOptions_withCenter(
                self.handle.native_mut(),
                &mut center.into().into(),
            );
        }
        self
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        unsafe {
            bindings::C_CameraOptions_withZoom(
                self.handle.native_mut(),
                zoom.into(),
            );
        }
        self
    }
}

impl NativeAccess<bindings::mbgl_CameraOptions> for CameraOptions {
    fn native(&self) -> &bindings::mbgl_CameraOptions {
        self.handle.native()
    }

    fn native_mut(&mut self) -> &mut bindings::mbgl_CameraOptions {
        self.handle.native_mut()
    }
}

impl Default for CameraOptions {
    fn default() -> Self {
        CameraOptions::new()
    }
}

impl NativeDrop for bindings::mbgl_AnimationOptions {
    fn drop(&mut self) {
        unsafe {
            bindings::C_AnimationOptions_delete(self);
        }
    }
}

impl NativeCopy<bindings::mbgl_AnimationOptions> for bindings::mbgl_AnimationOptions {
    fn copy_to(&self, dst: *mut bindings::mbgl_AnimationOptions) {
        unsafe {
            std::ptr::copy(self, dst, 1);
        }
    }
}

impl PartialEq<bindings::mbgl_AnimationOptions> for bindings::mbgl_AnimationOptions {
    fn eq(&self, other: &bindings::mbgl_AnimationOptions) -> bool {
        self.duration == other.duration &&
            self.minZoom == other.minZoom &&
            self.easing == other.easing &&
            self.transitionFinishFn == other.transitionFinishFn &&
            self.transitionFrameFn == other.transitionFrameFn &&
            self.velocity == other.velocity
    }
}

#[derive(Clone)]
pub struct AnimationOptions {
    handle: Handle<bindings::mbgl_AnimationOptions>,
}

impl NativeAccess<bindings::mbgl_AnimationOptions> for AnimationOptions {
    fn native(&self) -> &bindings::mbgl_AnimationOptions {
        self.handle.native()
    }

    fn native_mut(&mut self) -> &mut bindings::mbgl_AnimationOptions {
        self.handle.native_mut()
    }
}

impl Default for AnimationOptions {
    fn default() -> Self {
        AnimationOptions::new()
    }
}

impl AnimationOptions {
    pub fn new() -> Self {
        unsafe {
            AnimationOptions {
                handle: Handle::from_ptr(bindings::C_AnimationOptions_new()),
            }
        }
    }

    pub fn with_duration(mut self, duration: impl Into<Duration>) -> Self {
        unsafe {
            bindings::C_AnimationOptions_setDuration(
                self.handle.native_mut(),
                duration.into().as_millis() as u64,
            );
        }
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ScreenCoordinate(bindings::mbgl_ScreenCoordinate);

impl ScreenCoordinate {
    pub fn new(x: f32, y: f32) -> Self {
        ScreenCoordinate(bindings::mbgl_ScreenCoordinate {
            x: x as f64,
            y: y as f64,
            _phantom_0: PhantomData,
        })
    }

    pub fn x(&self) -> f32 { self.0.x as f32 }

    pub fn y(&self) -> f32 { self.0.y as f32 }
}

impl Default for ScreenCoordinate {
    fn default() -> Self {
        ScreenCoordinate::new(0.0, 0.0)
    }
}

impl From<(f32, f32)> for ScreenCoordinate {
    fn from(values: (f32, f32)) -> Self {
        ScreenCoordinate::new(values.0, values.1)
    }
}

impl Into<bindings::mbgl_ScreenCoordinate> for ScreenCoordinate {
    fn into(self) -> bindings::mbgl_ScreenCoordinate {
        self.0
    }
}

impl PtrOrNull<bindings::mbgl_ScreenCoordinate> for Option<ScreenCoordinate> {
    fn ptr_or_null(&self) -> *const bindings::mbgl_ScreenCoordinate {
        match self {
            None => std::ptr::null(),
            Some(value) => &value.0,
        }
    }
}
