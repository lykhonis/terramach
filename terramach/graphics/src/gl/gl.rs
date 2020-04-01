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
use std::rc::Rc;

use crate::{gl, IRect};

pub type FunctionPointer = *const c_void;
pub type LoadFunction<T> = fn(&mut T, &str) -> FunctionPointer;
pub type Function<T> = fn(&mut T) -> ();

#[derive(Clone)]
pub struct Gl {
    context: gl::Context,
    load_function: Box<Rc<dyn Fn(&str) -> FunctionPointer>>,
}

impl Gl {
    pub fn new<F>(
        context: impl Into<Option<gl::Context>>,
        load_function: F,
    ) -> Option<Self> where F: 'static + Fn(&str) -> FunctionPointer {
        Some(Gl {
            context: context.into().or_else(|| gl::Context::current())?,
            load_function: Box::new(Rc::new(load_function)),
        })
    }

    pub fn view_port(&self) -> IRect {
        gl_view_port()
    }

    pub fn context(&mut self) -> &mut gl::Context {
        &mut self.context
    }

    pub fn load_function(&mut self, name: &str) -> FunctionPointer {
        (self.load_function)(name)
    }
}

pub fn gl_view_port() -> IRect {
    unsafe {
        let mut view_port = [0; 4];
        gl::GetIntegerv(gl::VIEWPORT, view_port.as_mut_ptr());
        IRect::from_xywh(
            view_port[0],
            view_port[1],
            view_port[2],
            view_port[3],
        )
    }
}
