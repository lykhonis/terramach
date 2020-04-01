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

use std::sync::{Arc, Mutex};
use std::ptr::null;
use std::ops::DerefMut;

use crate::{gpu, gl};
use crate::{
    AlphaType, Budgeted, ColorSpace, ColorType, Display as GrDisplay, ImageInfo, Rect, Size,
    Surface,
};

pub struct Display<T: 'static> {
    size: Size,
    info: Arc<Mutex<T>>,
    clear_current: gl::Function<T>,
    make_current: gl::Function<T>,
    present_current: gl::Function<T>,
    load_function: gl::LoadFunction<T>,
    gpu_context: gpu::Context,
}

impl<T: 'static> Display<T> {
    pub fn new(
        size: impl Into<Size>,
        mut info: T,
        load_function: gl::LoadFunction<T>,
        clear_current: gl::Function<T>,
        make_current: gl::Function<T>,
        present_current: gl::Function<T>,
    ) -> Option<Self> {
        gl::load_with(|symbol| (load_function)(&mut info, symbol));
        let interface = gpu::gl::Interface::new_load_with(|symbol| load_function(&mut info, symbol));
        let context = gpu::Context::new_gl(interface)?;
        Some(Display {
            size: size.into(),
            info: Arc::new(Mutex::new(info)),
            clear_current,
            make_current,
            present_current,
            load_function,
            gpu_context: context,
        })
    }

    fn fbo(&self) -> u32 {
        unsafe {
            let mut fbo = 0;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fbo as *mut i32);
            fbo as u32
        }
    }
}

impl<T> GrDisplay for Display<T> {
    fn size(&self) -> Size {
        self.size
    }

    fn resize(&mut self, size: Size) {
        self.size = size;
    }

    fn pixel_ratio(&self) -> f32 {
        let view_port = gl::gl_view_port().size();
        (view_port.width as f32 / self.size.width).min(view_port.height as f32 / self.size.height)
    }

    fn new_surface(&mut self) -> Option<Surface> {
        let view_port = gl::gl_view_port().size();
        let pixel_ratio = self.pixel_ratio();
        let fboid = self.fbo();
        let (color_type, format) = {
            if self.gpu_context.color_type_supported_as_surface(ColorType::RGBA8888) {
                (ColorType::RGBA8888, gl::RGBA8)
            } else if self.gpu_context.color_type_supported_as_surface(ColorType::ARGB4444) {
                (ColorType::ARGB4444, gl::RGBA4)
            } else if self.gpu_context.color_type_supported_as_surface(ColorType::RGB565) {
                (ColorType::RGB565, gl::RGB565)
            } else {
                (ColorType::Unknown, 0)
            }
        };
        let frame_buffer_info = gpu::gl::FramebufferInfo { fboid, format };
        let render_target = gpu::BackendRenderTarget::new_gl(
            (view_port.width, view_port.height),
            None,
            0,
            frame_buffer_info,
        );
        let mut surface = Surface::from_backend_render_target(
            &mut self.gpu_context,
            &render_target,
            gpu::SurfaceOrigin::BottomLeft,
            color_type,
            ColorSpace::new_srgb(),
            None,
        )?;
        surface.canvas().scale((pixel_ratio, pixel_ratio));
        Some(surface)
    }

    fn new_offscreen_surface(&mut self, size: Size) -> Option<Surface> {
        let info = ImageInfo::new_n32(
            size.to_ceil(),
            AlphaType::Opaque,
            Some(ColorSpace::new_srgb()),
        );
        Surface::new_render_target(
            &mut self.gpu_context,
            Budgeted::YES,
            &info,
            None,
            gpu::SurfaceOrigin::BottomLeft,
            None,
            None,
        )
    }

    fn clear_current(&mut self) {
        if let Ok(mut info) = self.info.lock() {
            (self.clear_current)(info.deref_mut());
        }
    }

    fn make_current(&mut self) {
        if let Ok(mut info) = self.info.lock() {
            (self.make_current)(info.deref_mut());
        }
    }

    fn present_current(&mut self) {
        if let Ok(mut info) = self.info.lock() {
            (self.present_current)(info.deref_mut());
        }
    }

    fn gpu_context(&mut self) -> Option<&mut gpu::Context> {
        Some(&mut self.gpu_context)
    }

    fn gl(&mut self) -> Option<gl::Gl> {
        let info = self.info.clone();
        let load_function = self.load_function;
        gl::Gl::new(
            None,
            move |name| {
                if let Ok(mut info) = info.lock() {
                    (load_function)(info.deref_mut(), name)
                } else {
                    null()
                }
            })
    }
}
