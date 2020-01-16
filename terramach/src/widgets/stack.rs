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

use crate::{
    AnyWidget, BoxedWidget, BuildContext, Constraints, HitTestContext, LayoutContext, MeasuredSize,
    PartialWidget, Widget, WidgetContext,
};

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Stack {
    children: Vec<BoxedWidget>,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            children: Vec::new(),
        }
    }

    pub fn with_child(mut self, widget: impl Into<BoxedWidget>) -> Self {
        self.children.push(widget.into());
        self
    }
}

impl Widget for Stack {
    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        for child in &self.children {
            build.add_child(child.clone());
        }
    }
}
