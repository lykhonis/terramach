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

use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::time::Duration;

use crate::{AnyWidget, BoxedWidget, BuildContext, LayoutContext, MeasuredSize, MountContext, PaintContext, PartialWidget, Widget, WidgetContext, UpdateContext, EventContext, HitTestContext, Event, Animation, Id, Timer, Animator, Tween, AnimationCurves, KeyAction};

use terramach_graphics::textlayout::{
    FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle as GrTextStyle, TextAlign,
};
use terramach_graphics::{Color, Rect, Color4f, FontMgr, FontStyle, Paint, Point, Size, Typeface};

#[derive(Default, Clone)]
pub struct TextStyle {
    background_color: Option<Color>,
    color: Option<Color>,
    font_size: Option<f32>,
    font_style: Option<FontStyle>,
    typeface: Option<Typeface>,
    font_families: Vec<String>,
    hint_color: Option<Color>,
    highlight_color: Option<Color>,
}

impl TextStyle {
    const DEFAULT_COLOR: Color = Color::BLACK;
    const DEFAULT_HINT_COLOR: Color = Color::new(0xFF808080);
    const DEFAULT_HIGHLIGHT_COLOR: Color = Color::new(0xFF808080);
    const DEFAULT_FONT_SIZE: f32 = 14.0;

    pub fn with_background_color(mut self, background_color: impl Into<Option<Color>>) -> Self {
        self.background_color = background_color.into();
        self
    }

    pub fn with_color(mut self, color: impl Into<Option<Color>>) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_hint_color(mut self, color: impl Into<Option<Color>>) -> Self {
        self.hint_color = color.into();
        self
    }

    pub fn with_highlight_color(mut self, color: impl Into<Option<Color>>) -> Self {
        self.highlight_color = color.into();
        self
    }

    pub fn with_font_size(mut self, font_size: impl Into<Option<f32>>) -> Self {
        self.font_size = font_size.into();
        self
    }

    pub fn with_font_style(mut self, font_style: impl Into<Option<FontStyle>>) -> Self {
        self.font_style = font_style.into();
        self
    }

    pub fn with_typeface(mut self, typeface: impl Into<Option<Typeface>>) -> Self {
        self.typeface = typeface.into();
        self
    }

    pub fn with_font_families(mut self, font_families: &[impl AsRef<str>]) -> Self {
        self.font_families = font_families
            .iter()
            .map(|f| f.as_ref().to_string())
            .collect();
        self
    }

    pub fn background_color(&self) -> Option<Color> {
        self.background_color
    }

    pub fn color(&self) -> Color {
        self.color.unwrap_or(TextStyle::DEFAULT_COLOR)
    }

    pub fn hint_color(&self) -> Color {
        self.hint_color.unwrap_or(TextStyle::DEFAULT_HINT_COLOR)
    }

    pub fn highlight_color(&self) -> Color {
        self.highlight_color.unwrap_or(TextStyle::DEFAULT_HIGHLIGHT_COLOR)
    }

    pub fn font_size(&self) -> f32 {
        self.font_size.unwrap_or(TextStyle::DEFAULT_FONT_SIZE)
    }

    pub fn font_style(&self) -> FontStyle {
        self.font_style.unwrap_or_default()
    }

    pub fn font_families(&self) -> &[String] {
        self.font_families.as_slice()
    }

    pub fn typeface(&self) -> Option<&Typeface> {
        self.typeface.as_ref()
    }

    pub fn merge(&self, other: &TextStyle) -> TextStyle {
        TextStyle {
            background_color: self.background_color.or(other.background_color),
            color: self.color.or(other.color),
            font_size: self.font_size.or(other.font_size),
            font_style: self.font_style.or(other.font_style),
            typeface: self.typeface.clone().or(other.typeface.clone()),
            hint_color: self.hint_color.or(other.hint_color),
            highlight_color: self.highlight_color.or(other.highlight_color),
            font_families: self
                .font_families
                .iter()
                .chain(&other.font_families)
                .cloned()
                .collect(),
        }
    }
}

impl PartialEq for TextStyle {
    fn eq(&self, other: &Self) -> bool {
        let typeface = self.typeface.as_ref().map(|typeface| {
            if let Some(other) = other.typeface.as_ref() {
                typeface.font_style() == other.font_style()
                    && typeface.family_name() == other.family_name()
            } else {
                false
            }
        }).unwrap_or_default();
        self.background_color == other.background_color
            && self.color == other.color
            && self.font_size == other.font_size
            && self.font_style == other.font_style
            && self.font_families == other.font_families
            && self.hint_color == other.hint_color
            && self.highlight_color == other.highlight_color
            && typeface
    }
}

impl Debug for TextStyle {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct("TextStyle")
            .field("background_color", &self.background_color)
            .field("color", &self.color)
            .field("font_size", &self.font_size)
            .field(
                "font_style",
                &self.font_style.map(|s| {
                    format!(
                        "width: {}, weight: {}, slant: {:?}",
                        *s.width(),
                        *s.weight(),
                        s.slant()
                    )
                }),
            )
            .field("font_families", &self.font_families)
            .field(
                "typeface",
                &format_args!("{:?}", self.typeface.as_ref().map(|t| t.family_name())),
            )
            .field("hint_color", &self.hint_color)
            .field("highlight_color", &self.highlight_color)
            .finish()
    }
}

impl Hash for TextStyle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(color) = self.background_color {
            state.write_u8(color.a());
            state.write_u8(color.r());
            state.write_u8(color.g());
            state.write_u8(color.b());
        }
        if let Some(color) = self.color {
            state.write_u8(color.a());
            state.write_u8(color.r());
            state.write_u8(color.g());
            state.write_u8(color.b());
        }
        if let Some(font_size) = self.font_size {
            state.write_u32(font_size.to_bits());
        }
        if let Some(font_style) = self.font_style {
            font_style.slant().hash(state);
            state.write_i32(*font_style.weight());
            state.write_i32(*font_style.width());
        }
        if let Some(typeface) = &self.typeface {
            typeface.family_name().hash(state);
        }
        if let Some(color) = self.hint_color {
            state.write_u8(color.a());
            state.write_u8(color.r());
            state.write_u8(color.g());
            state.write_u8(color.b());
        }
        if let Some(color) = self.highlight_color {
            state.write_u8(color.a());
            state.write_u8(color.r());
            state.write_u8(color.g());
            state.write_u8(color.b());
        }
    }
}

#[derive(Clone, PartialEq, PartialWidget)]
pub struct DefaultTextStyle {
    text_style: TextStyle,
    child: BoxedWidget,
}

impl DefaultTextStyle {
    pub fn new(text_style: impl Into<Option<TextStyle>>, child: impl Into<BoxedWidget>) -> Self {
        DefaultTextStyle {
            text_style: text_style.into().unwrap_or_default(),
            child: child.into(),
        }
    }

    pub fn text_style(&self) -> &TextStyle {
        &self.text_style
    }
}

impl Widget for DefaultTextStyle {
    fn build(&self, _: &mut WidgetContext, build: &mut BuildContext) {
        build.add_child(self.child.clone());
    }
}

#[derive(Clone, PartialEq, PartialWidget)]
pub struct Text {
    text: String,
    text_style: Option<TextStyle>,
}

impl Text {
    pub fn new<T: AsRef<str>>(
        text: impl Into<Option<T>>,
        text_style: impl Into<Option<TextStyle>>,
    ) -> Self {
        Self {
            text: text.into().map(|s| s.as_ref().to_string()).unwrap_or_default(),
            text_style: text_style.into(),
        }
    }

    pub fn new_empty() -> Self {
        Text {
            text: String::default(),
            text_style: None,
        }
    }

    pub fn new_text<T: AsRef<str>>(text: impl Into<Option<T>>) -> Self {
        Text::new(
            text,
            None,
        )
    }

    pub fn with_text_style(mut self, text_style: TextStyle) -> Self {
        self.text_style = Some(text_style);
        self
    }

    fn make_font_collection(&self) -> FontCollection {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        font_collection
    }

    fn make_text_style(&self, text_style: &TextStyle) -> GrTextStyle {
        let foreground = Paint::new(Color4f::from(text_style.color()), None);

        let mut gr_text_style = GrTextStyle::new();
        gr_text_style.set_foreground_color(foreground);
        gr_text_style.set_font_size(text_style.font_size());
        gr_text_style.set_font_style(text_style.font_style());
        gr_text_style.set_typeface(text_style.typeface().cloned());
        gr_text_style.set_font_families(text_style.font_families());

        if let Some(color) = text_style.background_color() {
            let background = Paint::new(Color4f::from(color), None);
            gr_text_style.set_background_color(background);
        }

        gr_text_style
    }
}

impl Widget for Text {
    fn mount(&self, context: &mut WidgetContext, mount: &mut MountContext) {
        let mut text_style = self.text_style.clone().unwrap_or_default();
        if let Some(default_text_style) = mount.ancestor_widget::<DefaultTextStyle>() {
            text_style = text_style.merge(default_text_style.text_style());
        }

        let mut paragraph_style = ParagraphStyle::new();
        paragraph_style.set_text_align(TextAlign::Start);

        context.set_state(TextState::new(&self.text, paragraph_style, self.make_text_style(&text_style)));
    }

    fn update(&self, context: &mut WidgetContext, update: &mut UpdateContext) {
        let mut text_style = self.text_style.clone().unwrap_or_default();
        if let Some(default_text_style) = update.ancestor_widget::<DefaultTextStyle>() {
            text_style = text_style.merge(default_text_style.text_style());
        }

        let state = context.state_mut::<TextState>().unwrap();
        state.data = self.text.clone();
        state.gr_text_style = self.make_text_style(&text_style);
    }

    fn layout(&self, context: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        let state = context.state_mut::<TextState>().unwrap();

        let mut paragraph_builder =
            ParagraphBuilder::new(&state.paragraph_style, self.make_font_collection());
        paragraph_builder.push_style(&state.gr_text_style);
        paragraph_builder.add_text(state.data());

        let mut paragraph = paragraph_builder.build();
        paragraph.layout(layout.constraints().maximum_size().width);

        let size = Size::new(
            paragraph.max_intrinsic_width(),
            paragraph.height(),
        ).constrain(layout.constraints());
        state.paragraph = Some(paragraph);
        size
    }

    fn paint(&self, context: &mut WidgetContext, paint: &mut PaintContext) {
        let state = context.state_mut::<TextState>().unwrap();
        if let Some(paragraph) = &mut state.paragraph {
            paragraph.paint(paint.canvas(), Point::default());
        }
    }
}

struct TextState {
    data: String,
    paragraph_style: ParagraphStyle,
    gr_text_style: GrTextStyle,
    paragraph: Option<Paragraph>,
}

impl TextState {
    pub fn new(
        data: impl AsRef<str>,
        paragraph_style: ParagraphStyle,
        gr_text_style: GrTextStyle,
    ) -> Self {
        TextState {
            data: data.as_ref().to_string(),
            paragraph_style,
            gr_text_style,
            paragraph: None,
        }
    }

    pub fn data(&self) -> &str {
        self.data.as_str()
    }
}
