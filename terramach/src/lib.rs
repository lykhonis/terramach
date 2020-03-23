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

#![feature(vec_remove_item)]

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[macro_use]
extern crate objc;

mod animation;
mod app;
mod bundle;
mod common;
mod event;
mod gesture;
mod hit;
mod layer;
mod layers;
mod layout;
mod paint;
mod timers;
mod touch;
mod tree;
mod widget;
pub mod widgets;
mod keys;
pub mod platform;
mod target;
pub mod gpu;

pub(crate) use common::*;
pub(crate) use app::*;

pub use animation::*;
pub use bundle::*;
pub use event::*;
pub use gesture::*;
pub use hit::*;
pub use layer::*;
pub use layers::*;
pub use layout::*;
pub use paint::*;
pub use timers::*;
pub use touch::*;
pub use tree::*;
pub use widget::*;
pub use keys::*;
pub use target::*;

pub use terramach_graphics as graphics;
pub use terramach_support::*;
