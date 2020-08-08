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

use crate::{
    AnyWidget, BoxedWidget, BuildContext, Constraints, LayoutContext, MeasuredSize, PaintContext,
    PartialWidget, Widget, WidgetContext,
};

use terramach_graphics::Size;

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Constrained {
    constraints: Constraints,
    child: Option<BoxedWidget>,
}

impl Constrained {
    pub fn new_empty(constraints: impl Into<Constraints>) -> Self {
        Self {
            constraints: constraints.into(),
            child: None,
        }
    }

    pub fn new(constraints: impl Into<Constraints>, child: impl Into<BoxedWidget>) -> Self {
        Self {
            constraints: constraints.into(),
            child: Some(child.into()),
        }
    }
}

impl Widget for Constrained {
    fn layout(&self, _: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        let child_constraints = self.constraints.constrain(layout.constraints());
        layout
            .layout_child(0, &child_constraints)
            .unwrap_or(child_constraints.minimum_size())
            .constrain(&child_constraints)
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        if let Some(child) = &self.child {
            build.add_child(child.clone());
        }
    }
}
