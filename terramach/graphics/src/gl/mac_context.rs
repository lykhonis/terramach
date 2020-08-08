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

use std::ptr::null_mut;
use std::rc::Rc;

use cgl;

struct CGLContext {
    context: cgl::CGLContextObj,
    owner: bool,
}

impl CGLContext {
    pub unsafe fn native(&self) -> cgl::CGLContextObj {
        self.context
    }
}

impl Drop for CGLContext {
    fn drop(&mut self) {
        unsafe {
            cgl::CGLSetCurrentContext(null_mut());
            if self.owner {
                let error = cgl::CGLDestroyContext(self.context);
                debug_assert!(error == 0);
            }
        }
    }
}

pub struct Context {
    inner: Rc<CGLContext>,
    guard: Option<ContextGuard>,
}

impl Context {
    pub fn current() -> Option<Self> {
        unsafe {
            let context = cgl::CGLGetCurrentContext();
            if context.is_null() {
                None
            } else {
                Some(Context {
                    inner: Rc::new(CGLContext {
                        context,
                        owner: false,
                    }),
                    guard: None,
                })
            }
        }
    }

    pub fn new_shared(&mut self) -> Self {
        unsafe {
            let pixel_format = cgl::CGLGetPixelFormat(self.inner.native());
            debug_assert!(!pixel_format.is_null());
            let mut context = null_mut();
            let error = cgl::CGLCreateContext(pixel_format, self.inner.native(), &mut context);
            debug_assert!(error == 0);
            debug_assert!(!context.is_null());
            Context {
                inner: Rc::new(CGLContext {
                    context,
                    owner: true,
                }),
                guard: None,
            }
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
    current: cgl::CGLContextObj,
}

impl ContextGuard {
    fn new(context: &Context) -> Self {
        unsafe {
            let current = cgl::CGLGetCurrentContext();
            let error = cgl::CGLSetCurrentContext(context.inner.native());
            debug_assert!(error == 0);
            ContextGuard {
                current,
            }
        }
    }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        unsafe {
            let error = cgl::CGLSetCurrentContext(self.current);
            debug_assert!(error == 0);
        }
    }
}
