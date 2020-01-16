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

use core_graphics::display::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref MAIN: DisplayMetrics = unsafe { DisplayMetrics::new(CGMainDisplayID()) };
}

#[derive(Debug, Clone, Copy)]
pub struct DisplayMetrics {
    device_pixel_ratio: f32,
}

impl DisplayMetrics {
    unsafe fn new(display: CGDirectDisplayID) -> Self {
        let physical_size = CGDisplayScreenSize(display);
        let width_pixels = CGDisplayPixelsWide(display);
        let device_pixel_ratio = width_pixels as f32 / physical_size.width as f32;
        DisplayMetrics { device_pixel_ratio }
    }

    pub fn device_pixel_ratio(&self) -> f32 {
        self.device_pixel_ratio
    }
}

impl Default for DisplayMetrics {
    fn default() -> Self {
        *MAIN
    }
}
