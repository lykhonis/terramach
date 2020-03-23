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

use std::any::Any;
use std::ptr::null;

use crate::gpu::{SharedPipeline, TextureId, TextureContext};
use crate::platform::DisplayMetrics;

use terramach_graphics::{Size, Image, ISize, Canvas, Point, ColorType, ColorSpace, AlphaType};
use terramach_graphics::{gpu, gl};

pub struct RenderContext<'a> {
    gl: &'a mut gl::Gl,
    size: ISize,
}

impl<'a> RenderContext<'a> {
    fn new(
        gl: &'a mut gl::Gl,
        size: ISize,
    ) -> Self {
        RenderContext {
            gl,
            size,
        }
    }

    pub fn gl(&mut self) -> &mut gl::Gl {
        self.gl
    }

    pub fn size(&self) -> ISize {
        self.size
    }
}

pub struct PrerollContext<'a> {
    context: &'a mut gpu::Context,
    texture: TextureId,
    gl: &'a mut gl::Gl,
    pipeline: &'a mut SharedPipeline,
    size: ISize,
}

impl<'a> PrerollContext<'a> {
    fn new(
        pipeline: &'a mut SharedPipeline,
        gl: &'a mut gl::Gl,
        context: &'a mut gpu::Context,
        texture: TextureId,
        size: ISize,
    ) -> Self {
        PrerollContext {
            pipeline,
            gl,
            context,
            texture,
            size,
        }
    }

    pub fn gpu_context(&mut self) -> &mut gpu::Context {
        self.context
    }

    pub fn gl(&mut self) -> &mut gl::Gl {
        self.gl
    }

    pub fn size(&self) -> ISize {
        self.size
    }

    pub fn texture(&self) -> TextureId {
        self.texture
    }

    pub fn pipeline(&mut self) -> &mut SharedPipeline {
        self.pipeline
    }
}

pub trait RenderTexture: Any {
    fn preroll(&mut self, preroll: &mut PrerollContext);

    fn update(&mut self, _update: &mut TextureContext) {}

    fn render(&mut self, render: &mut RenderContext);
}

pub struct Texture {
    gl: gl::Gl,
    gl_context: gl::Context,
    frame_buffer: gl::types::GLuint,
    texture: gl::types::GLuint,
    context: gpu::Context,
    id: TextureId,
    render_texture: Box<dyn RenderTexture>,
    pipeline: SharedPipeline,
    need_preroll: bool,
    need_render: bool,
    need_update: bool,
    size: Option<ISize>,
    image: Option<Image>,
}

impl Texture {
    pub fn new(
        mut gl: gl::Gl,
        context: &gpu::Context,
        id: TextureId,
        pipeline: SharedPipeline,
        render_texture: Box<dyn RenderTexture>,
    ) -> Self {
        let mut gl_context = gl.context().new_shared();
        let mut frame_buffer: gl::types::GLuint = 0;
        let mut texture: gl::types::GLuint = 0;
        unsafe {
            let _gl_guard = gl_context.make_current();
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::GenFramebuffers(1, &mut frame_buffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture, 0);
        }
        Texture {
            gl,
            gl_context,
            frame_buffer,
            texture,
            context: context.clone(),
            id,
            pipeline,
            render_texture,
            need_preroll: true,
            need_render: true,
            need_update: false,
            size: None,
            image: None,
        }
    }

    pub fn need_render(&self) -> bool {
        self.need_render
    }

    pub fn mark_need_render(&mut self) {
        self.need_render = true;
    }

    pub fn mark_need_preroll(&mut self) {
        self.need_preroll = true;
    }

    pub fn update(&mut self) {
        if self.need_preroll {
            self.need_update = true;
        } else {
            self.need_update = false;
            self.render_texture.update(&mut TextureContext::new());
        }
    }

    pub fn preroll(&mut self, size: impl Into<Size>) {
        let size = size.into().to_ceil();
        let size_changed = self.size != Some(size);
        self.size = Some(size);
        if size_changed {
            self.need_render = true;
        }
        if self.need_preroll || size_changed {
            {
                let pixel_ratio = DisplayMetrics::default().device_pixel_ratio();
                let texture_size = ISize::new(
                    (size.width as f32 * pixel_ratio).ceil() as i32,
                    (size.height as f32 * pixel_ratio).ceil() as i32,
                );
                let _gl_guard = self.gl_context.make_current();
                unsafe {
                    gl::BindTexture(gl::TEXTURE_2D, self.texture);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, texture_size.width, texture_size.height, 0, gl::RGBA, gl::UNSIGNED_BYTE, null());
                    gl::BindTexture(gl::TEXTURE_2D, 0);
                    gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer);
                    gl::Viewport(0, 0, texture_size.width, texture_size.height);
                }
                if self.need_preroll {
                    self.need_preroll = false;
                    self.render_texture.preroll(&mut PrerollContext::new(
                        &mut self.pipeline,
                        &mut self.gl,
                        &mut self.context,
                        self.id,
                        size,
                    ));
                }
            }
            {
                let backend_texture = unsafe {
                    gpu::BackendTexture::new_gl(
                        (size.width, size.height),
                        gpu::MipMapped::No,
                        gpu::gl::TextureInfo {
                            target: gl::TEXTURE_2D,
                            id: self.texture,
                            format: gl::RGBA8,
                        },
                    )
                };
                self.image = Image::from_texture(
                    &mut self.context,
                    &backend_texture,
                    gpu::SurfaceOrigin::BottomLeft,
                    ColorType::RGBA8888,
                    AlphaType::Premul,
                    ColorSpace::new_srgb(),
                );
            }
        }
        if self.need_update {
            self.update();
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas) {
        if self.need_render {
            self.need_render = false;
            debug_assert!(self.size.is_some(), "Drawing texture but it was not prerolled");
            if let Some(size) = self.size {
                let _gl_guard = self.gl_context.make_current();
                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer);
                    gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                self.render_texture.render(&mut RenderContext::new(&mut self.gl, size));
                unsafe {
                    gl::Flush();
                }
            }
        }
        if let Some(image) = &self.image {
            canvas.draw_image(image, Point::default(), None);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        let _gl_guard = self.gl_context.make_current();
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteFramebuffers(1, &self.frame_buffer);
        }
    }
}
