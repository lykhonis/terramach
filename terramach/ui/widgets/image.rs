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

use crate::widgets::Alignment;
use crate::{
    BoxedWidget, Fit, HitTestContext, LayoutContext, MeasuredSize, PaintContext, PartialWidget,
    Widget, WidgetContext,
};

use terramach_graphics::{
    color_filters, image_filters, Bitmap, BlendMode, Codec, CodecResult, Color, Color4f,
    ColorFilter, Data, Image as GrImage, ImageFilter, Paint, Point, Rect, Size,
};

#[derive(Clone, PartialWidget)]
pub struct Image {
    image: GrImage,
    alignment: Alignment,
    fit: Fit,
    color: Option<Color>,
}

impl Image {
    pub fn from_bytes(
        alignment: impl Into<Option<Alignment>>,
        fit: impl Into<Option<Fit>>,
        color: impl Into<Option<Color>>,
        bytes: &[u8],
    ) -> Self {
        let mut codec = Codec::from_data(Data::new_copy(bytes)).unwrap();

        let info = codec.info();
        let mut bitmap = Bitmap::new();
        bitmap.alloc_pixels_info(&info, None);
        let result = unsafe { codec.get_pixels(&info, bitmap.pixels(), bitmap.row_bytes()) };
        if result != CodecResult::Success {
            // TODO: Error?
        }
        bitmap.notify_pixels_changed();

        let image = GrImage::from_bitmap(&bitmap).unwrap();
        Self {
            alignment: alignment.into().unwrap_or_default(),
            fit: fit.into().unwrap_or_default(),
            color: color.into(),
            image,
        }
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.alignment == other.alignment
            && self.fit == other.fit
            && self.color == other.color
            && self.image.image_info() == other.image.image_info()
    }
}

impl Widget for Image {
    fn layout(&self, _context: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        layout.constraints().maximum_size()
    }

    fn paint(&self, _context: &mut WidgetContext, paint: &mut PaintContext) {
        let size = Size::from_isize(self.image.dimensions()).fit(paint.size(), self.fit);
        let bounds = Rect::from((self.alignment.align(paint.size(), size), size));

        let mut filters = Vec::new();
        if let Some(color) = self.color {
            filters.push(image_filters::color_filter(
                color_filters::blend(color, BlendMode::SrcATop).unwrap(),
                None,
                None,
            ));
        }

        let mut image_paint = Paint::default();
        image_paint.set_image_filter(ImageFilter::merge(filters, None));

        let canvas = paint.canvas();
        canvas.draw_image_rect(&self.image, None, &bounds, &image_paint);
    }

    fn hit_test(&self, _: &WidgetContext, hit_test: &mut HitTestContext) -> bool {
        hit_test.in_bounds()
    }
}
