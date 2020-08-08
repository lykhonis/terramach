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

#[path = "egl.rs"]
pub mod egl;

use std::rc::Rc;

struct EGLContext {
    display: egl::EGLDisplay,
    context: egl::EGLContext,
    read_surface: egl::EGLSurface,
    draw_surface: egl::EGLSurface,
    owner: bool,
}

impl EGLContext {
    pub fn native(&self) -> egl::EGLContext {
        self.context
    }
}

impl Drop for EGLContext {
    fn drop(&mut self) {
        egl::make_current(
            self.display,
            egl::EGL_NO_SURFACE,
            egl::EGL_NO_SURFACE,
            egl::EGL_NO_CONTEXT,
        );
        if self.owner {
            let result = egl::destroy_context(self.display, self.context);
            debug_assert!(result);
        }
    }
}

pub struct Context {
    inner: Rc<EGLContext>,
    guard: Option<ContextGuard>,
}

impl Context {
    pub fn current() -> Option<Self> {
        Some(Context {
            inner: Rc::new(EGLContext {
                owner: false,
                context: egl::get_current_context()?,
                display: egl::get_current_display()?,
                read_surface: egl::get_current_surface(egl::EGL_READ)?,
                draw_surface: egl::get_current_surface(egl::EGL_DRAW)?,
            }),
            guard: None,
        })
    }

    pub fn new_shared(&mut self) -> Self {
        let config = egl::android_choose_config(self.inner.display);
        Self {
            inner: Rc::new(EGLContext {
                owner: true,
                context: egl::android_new_context(self.inner.display, config),
                display: self.inner.display,
                read_surface: self.inner.read_surface,
                draw_surface: self.inner.draw_surface,
            }),
            guard: None,
        }
    }

    pub fn lock_current(&mut self) -> ContextGuard {
        ContextGuard::new(self)
    }

    pub fn make_current(&mut self) {
        if self.guard.is_none() {
            self.guard = self.lock_current().into();
        }
    }

    pub fn clear_current(&mut self) {
        self.guard = None;
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            guard: None,
        }
    }
}

pub struct ContextGuard {
    display: egl::EGLDisplay,
    current_context: Option<egl::EGLContext>,
    current_read_surface: Option<egl::EGLSurface>,
    current_draw_surface: Option<egl::EGLSurface>,
}

impl ContextGuard {
    fn new(context: &Context) -> Self {
        let current_context = egl::get_current_context();
        let current_read_surface = egl::get_current_surface(egl::EGL_READ);
        let current_draw_surface = egl::get_current_surface(egl::EGL_DRAW);
        let result = egl::make_current(
            context.inner.display,
            context.inner.draw_surface,
            context.inner.read_surface,
            context.inner.context,
        );
        debug_assert!(result);
        Self {
            current_context,
            current_read_surface,
            current_draw_surface,
            display: context.inner.display,
        }
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        let result = egl::make_current(
            self.display,
            self.current_draw_surface.unwrap_or(egl::EGL_NO_SURFACE),
            self.current_read_surface.unwrap_or(egl::EGL_NO_SURFACE),
            self.current_context.unwrap_or(egl::EGL_NO_CONTEXT),
        );
        debug_assert!(result);
    }
}
