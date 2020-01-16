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

use crate::{gl, gpu};
use crate::{Size, Surface};

pub trait Display {
    fn size(&self) -> Size;
    fn resize(&mut self, new_size: Size);
    fn new_surface(&mut self) -> Option<Surface>;
    fn new_offscreen_surface(&mut self, size: Size) -> Option<Surface>;
    fn clean_current(&mut self);
    fn make_current(&mut self);
    fn present_current(&mut self);
    fn gpu_context(&mut self) -> Option<&mut gpu::Context> { None }
    fn gl(&mut self) -> Option<gl::Gl> { None }
}

