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

use terramach_graphics::Size;

use crate::{
    AnyWidget, BoxedWidget, BuildContext, Constraints, HitTestContext, LayoutContext, MeasuredSize,
    PartialWidget, Widget, WidgetContext,
};
use crate::widgets::{Flex, MainAxisAlignment, CrossAxisAlignment};

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Row {
    children: Vec<Flex>,
    horizontal_alignment: Option<MainAxisAlignment>,
    vertical_alignment: Option<CrossAxisAlignment>,
}

impl Default for Row {
    fn default() -> Self {
        Row::new(None, CrossAxisAlignment::Middle)
    }
}

impl Row {
    pub fn new(
        horizontal_alignment: impl Into<Option<MainAxisAlignment>>,
        vertical_alignment: impl Into<Option<CrossAxisAlignment>>,
    ) -> Self {
        Row {
            children: Vec::new(),
            horizontal_alignment: horizontal_alignment.into(),
            vertical_alignment: vertical_alignment.into(),
        }
    }

    pub fn with_child(self, widget: impl Into<BoxedWidget>) -> Self {
        self.with_flex_child(None, widget)
    }

    pub fn with_flex_child(
        mut self,
        weight: impl Into<Option<usize>>,
        widget: impl Into<BoxedWidget>,
    ) -> Self {
        self.children.push(Flex::new(widget, weight));
        self
    }
}

impl Widget for Row {
    fn layout(&self, _: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        let minimum_size = layout.constraints().minimum_size();
        let maximum_size = layout.constraints().maximum_size();
        let child_count = self.children.len();
        let mut sizes = vec![Size::default(); child_count];
        let mut static_width = 0.0;

        // layout non-flex children
        for child in 0..child_count {
            if self.children[child].weight().is_some() {
                continue;
            }
            let maximum_size = maximum_size.deflate((static_width, 0.0));
            let mut minimum_height = minimum_size.min(maximum_size).height;
            if let Some(alignment) = &self.vertical_alignment {
                if alignment == &CrossAxisAlignment::Stretch {
                    minimum_height = maximum_size.height;
                }
            }
            let child_constraints = Constraints::new(
                Size::new(0.0, minimum_height),
                maximum_size,
            );
            let size = layout
                .layout_child(child, &child_constraints)
                .unwrap()
                .constrain(&child_constraints);
            sizes[child] = size;
            static_width += size.width;
        }

        // layout flex-children
        let total_weight: usize = self
            .children
            .iter()
            .filter_map(|child| child.weight())
            .sum();
        let flex_width = maximum_size.width - static_width;
        for child in 0..child_count {
            let weight = match self.children[child].weight() {
                Some(weight) => weight,
                None => continue,
            };
            let width = flex_width * weight as f32 / total_weight as f32;
            let mut minimum_height = minimum_size.height;
            if let Some(alignment) = &self.vertical_alignment {
                if alignment == &CrossAxisAlignment::Stretch {
                    minimum_height = maximum_size.height;
                }
            }
            let child_constraints = Constraints::new(
                Size::new(width, minimum_height),
                Size::new(width, maximum_size.height),
            );
            sizes[child] = layout
                .layout_child(child, &child_constraints)
                .unwrap()
                .constrain(&child_constraints);
        }

        let content_size = Size::new(
            sizes.iter().map(|s| s.width).sum(),
            sizes.iter().fold(0.0, |v, s| v.max(s.height)),
        );
        let size = content_size.constrain(layout.constraints());

        // position each child
        let mut offset = 0.0;
        for child in 0..child_count {
            let child_size = &sizes[child];
            let child_offset_x = if let Some(alignment) = &self.horizontal_alignment {
                match alignment {
                    MainAxisAlignment::Start => offset,
                    MainAxisAlignment::Middle => (size.width - content_size.width) / 2.0 + offset,
                    MainAxisAlignment::End => size.width - child_size.width - offset,
                }
            } else {
                offset
            };
            let child_offset_y = if let Some(alignment) = &self.vertical_alignment {
                match alignment {
                    CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
                    CrossAxisAlignment::Middle => (size.height - child_size.height) / 2.0,
                    CrossAxisAlignment::End => size.height - child_size.height,
                }
            } else {
                0.0
            };
            layout.set_child_offset(child, (child_offset_x, child_offset_y));
            offset += child_size.width;
        }

        size
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        for child in &self.children {
            build.add_child(child.widget().clone());
        }
    }
}
