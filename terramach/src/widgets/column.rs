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
use crate::widgets::{MainAxisAlignment, CrossAxisAlignment};

#[derive(Clone, PartialEq)]
pub(crate) struct Flex {
    widget: BoxedWidget,
    weight: Option<usize>,
}

impl Flex {
    pub fn new(widget: impl Into<BoxedWidget>, weight: impl Into<Option<usize>>) -> Self {
        Flex {
            widget: widget.into(),
            weight: weight.into(),
        }
    }

    pub fn widget(&self) -> &BoxedWidget {
        &self.widget
    }

    pub fn weight(&self) -> Option<usize> {
        self.weight
    }
}

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Column {
    children: Vec<Flex>,
    horizontal_alignment: Option<CrossAxisAlignment>,
    vertical_alignment: Option<MainAxisAlignment>,
}

impl Default for Column {
    fn default() -> Self {
        Column::new(CrossAxisAlignment::Middle, None)
    }
}

impl Column {
    pub fn new(
        horizontal_alignment: impl Into<Option<CrossAxisAlignment>>,
        vertical_alignment: impl Into<Option<MainAxisAlignment>>,
    ) -> Self {
        Column {
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

impl Widget for Column {
    fn layout(&self, _: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        let minimum_size = layout.constraints().minimum_size();
        let maximum_size = layout.constraints().maximum_size();
        let child_count = self.children.len();
        let mut sizes = vec![Size::default(); child_count];
        let mut static_height = 0.0;

        // layout non-flex children
        for child in 0..child_count {
            if self.children[child].weight().is_some() {
                continue;
            }
            let maximum_size = maximum_size.deflate((0.0, static_height));
            let mut minimum_width = minimum_size.min(maximum_size).width;
            if let Some(alignment) = &self.horizontal_alignment {
                if alignment == &CrossAxisAlignment::Stretch {
                    minimum_width = maximum_size.width;
                }
            }
            let child_constraints = Constraints::new(
                Size::new(minimum_width, 0.0),
                maximum_size,
            );
            let size = layout
                .layout_child(child, &child_constraints)
                .unwrap()
                .constrain(&child_constraints);
            static_height += size.height;
            sizes[child] = size;
        }

        // layout flex-children
        let total_weight: usize = self
            .children
            .iter()
            .filter_map(|child| child.weight())
            .sum();
        let flex_height = maximum_size.height - static_height;
        for child in 0..child_count {
            let weight = match self.children[child].weight() {
                Some(weight) => weight,
                None => continue,
            };
            let height = flex_height * weight as f32 / total_weight as f32;
            let mut minimum_width = minimum_size.width;
            if let Some(alignment) = &self.horizontal_alignment {
                if alignment == &CrossAxisAlignment::Stretch {
                    minimum_width = maximum_size.width;
                }
            }
            let child_constraints = Constraints::new(
                Size::new(minimum_width, height),
                Size::new(maximum_size.width, height),
            );
            sizes[child] = layout
                .layout_child(child, &child_constraints)
                .unwrap()
                .constrain(&child_constraints);
        }

        let content_size = Size::new(
            sizes.iter().fold(0.0, |v, s| v.max(s.width)),
            sizes.iter().map(|s| s.height).sum(),
        );
        let size = content_size.constrain(layout.constraints());

        // position each child
        let mut offset = 0.0;
        for child in 0..child_count {
            let child_size = &sizes[child];
            let child_offset_x = if let Some(alignment) = &self.horizontal_alignment {
                match alignment {
                    CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
                    CrossAxisAlignment::Middle => (size.width - content_size.width) / 2.0,
                    CrossAxisAlignment::End => size.width - child_size.width,
                }
            } else {
                0.0
            };
            let child_offset_y = if let Some(alignment) = &self.vertical_alignment {
                match alignment {
                    MainAxisAlignment::Start => offset,
                    MainAxisAlignment::Middle => (size.height - child_size.height) / 2.0 + offset,
                    MainAxisAlignment::End => size.height - child_size.height - offset,
                }
            } else {
                offset
            };
            layout.set_child_offset(child, (child_offset_x, child_offset_y));
            offset += sizes[child].height;
        }

        size
    }

    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        for child in &self.children {
            build.add_child(child.widget().clone());
        }
    }
}
