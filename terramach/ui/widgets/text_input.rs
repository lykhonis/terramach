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
use std::time::{Duration, Instant};

use crate::{
    BoxedWidget, LayoutContext, MeasuredSize, MountContext, PaintContext, PartialWidget, Widget,
    WidgetContext, UpdateContext, EventContext, HitTestContext, Event, Animation, Timer,
    Animator, Tween, AnimationCurves, Key, KeyAction,
};
use crate::widgets::{TextStyle, DefaultTextStyle};
use crate::platform::{Cursor, Clipboard};

use terramach_graphics::textlayout::{
    FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle as GrTextStyle, TextAlign,
    RectHeightStyle, RectWidthStyle,
};
use terramach_graphics::{Color, Rect, Color4f, FontMgr, Paint, Point, Size};

#[derive(Clone, PartialEq, PartialWidget)]
pub struct TextInput {
    text: String,
    hint: Option<String>,
    text_style: Option<TextStyle>,
    text_align: TextAlign,
    max_lines: usize,
}

impl TextInput {
    const CARET_BLINK: usize = 1;
    const CARET_BLINK_TIMEOUT: Duration = Duration::from_millis(400);
    const CARET_BLINK_FADE: Duration = Duration::from_millis(275);
    const CARET_MOVE: Duration = Duration::from_millis(100);

    pub fn new<T: AsRef<str>, H: AsRef<str>>(
        text: impl Into<Option<T>>,
        text_style: impl Into<Option<TextStyle>>,
        text_align: impl Into<Option<TextAlign>>,
        hint: impl Into<Option<H>>,
        max_lines: impl Into<Option<usize>>,
    ) -> Self {
        Self {
            text: text.into().map(|s| s.as_ref().to_string()).unwrap_or_default(),
            text_style: text_style.into(),
            text_align: text_align.into().unwrap_or(TextAlign::Start),
            hint: hint.into().map(|s| s.as_ref().to_string()),
            max_lines: max_lines.into().unwrap_or(1),
        }
    }

    pub fn new_empty() -> Self {
        Self {
            text: String::default(),
            text_style: None,
            text_align: TextAlign::Start,
            hint: None,
            max_lines: 1,
        }
    }

    pub fn new_text<T: AsRef<str>>(text: impl Into<Option<T>>) -> Self {
        let mut input = TextInput::new_empty();
        input.text = text.into().map(|s| s.as_ref().to_string()).unwrap_or_default();
        input
    }

    pub fn with_hint(mut self, hint: impl AsRef<str>) -> Self {
        self.hint = hint.as_ref().to_string().into();
        self
    }

    pub fn with_text_align(mut self, text_align: TextAlign) -> Self {
        self.text_align = text_align;
        self
    }

    pub fn with_text_style(mut self, text_style: TextStyle) -> Self {
        self.text_style = Some(text_style);
        self
    }

    pub fn with_max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = max_lines;
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

impl Widget for TextInput {
    fn mount(&self, context: &mut WidgetContext, mount: &mut MountContext) {
        let mut text_style = self.text_style.clone().unwrap_or_default();
        if let Some(default_text_style) = mount.ancestor_widget::<DefaultTextStyle>() {
            text_style = text_style.merge(default_text_style.text_style());
        }

        let mut paragraph_style = ParagraphStyle::new();
        paragraph_style.set_text_align(self.text_align);
        paragraph_style.set_max_lines(self.max_lines);

        context.set_cursor(Cursor::Text);
        context.set_state(TextState::new(
            &self.text,
            self.make_font_collection(),
            paragraph_style,
            self.make_text_style(&text_style),
            &self.hint,
            text_style.hint_color(),
            text_style.highlight_color(),
        ));
    }

    fn update(&self, context: &mut WidgetContext, update: &mut UpdateContext) {
        let mut text_style = self.text_style.clone().unwrap_or_default();
        if let Some(default_text_style) = update.ancestor_widget::<DefaultTextStyle>() {
            text_style = text_style.merge(default_text_style.text_style());
        }

        let mut paragraph_style = ParagraphStyle::new();
        paragraph_style.set_text_align(self.text_align);
        paragraph_style.set_max_lines(self.max_lines);

        let state = context.state_mut::<TextState>().unwrap();
        state.text_style = self.make_text_style(&text_style);
        state.paragraph_style = paragraph_style;
        state.hint = self.hint.clone();
        state.hint_color = text_style.hint_color();
        state.highlight_color = text_style.highlight_color();
        state.invalidate();
    }

    fn layout(&self, context: &mut WidgetContext, layout: &mut LayoutContext) -> Size {
        let state = context.state_mut::<TextState>().unwrap();
        // state.max_width = layout.constraints().maximum_size().width;
        state.invalidate();
        Size::new_unbound_width(state.paragraph_height().unwrap())
            .constrain(layout.constraints())
    }

    fn event(&self, context: &mut WidgetContext, event: &mut EventContext) {
        match event.get() {
            Event::BecameResponder => {
                let state = context.state_mut::<TextState>().unwrap();
                state.focused = true;
                state.caret_active = true;
                state.caret_opacity = None;
                state.caret_bounds = None;
                context.schedule_timer(
                    TextInput::CARET_BLINK,
                    Timer::new(TextInput::CARET_BLINK_TIMEOUT, None),
                );
                event.mark_need_paint();
            }
            Event::ResignedResponder => {
                let state = context.state_mut::<TextState>().unwrap();
                state.focused = false;
                state.caret_opacity = None;
                state.caret_bounds = None;
                context.cancel_all_timers();
                event.mark_need_paint();
            }
            Event::Focus(focused) => {
                let focused = *focused;
                let state = context.state_mut::<TextState>().unwrap();
                state.focused = focused;
                state.caret_opacity = None;
                if focused {
                    context.schedule_timer(
                        TextInput::CARET_BLINK,
                        Timer::new(TextInput::CARET_BLINK_TIMEOUT, None),
                    );
                } else {
                    context.cancel_all_timers();
                }
                event.mark_need_paint();
            }
            Event::Timer(timer_id) => {
                let state = context.state_mut::<TextState>().unwrap();
                if timer_id == &TextInput::CARET_BLINK {
                    if state.caret_active {
                        state.caret_opacity = TextInput::CARET_BLINK_FADE
                            .animate(Tween::new(state.caret_opacity(), 0.0))
                            .with(AnimationCurves::ease_out())
                            .into();
                    } else {
                        state.caret_opacity = TextInput::CARET_BLINK_FADE
                            .animate(Tween::new(state.caret_opacity(), 1.0))
                            .with(AnimationCurves::ease_in())
                            .into();
                    }
                    context.request_frame();
                }
            }
            Event::Frame(_) => {
                let state = context.state_mut::<TextState>().unwrap();
                if let Some(caret_bounds) = &mut state.caret_bounds {
                    if caret_bounds.advance() {
                        context.request_frame();
                    } else {
                        state.caret_bounds = None;
                    }
                    event.mark_need_paint();
                }

                let state = context.state_mut::<TextState>().unwrap();
                if let Some(caret_opacity) = &mut state.caret_opacity {
                    if caret_opacity.advance() {
                        context.request_frame();
                    } else {
                        state.caret_active = !state.caret_active;
                        state.caret_opacity = None;
                        let will_activate = state.caret_active;
                        context.schedule_timer(
                            TextInput::CARET_BLINK,
                            Timer::new(
                                if will_activate { TextInput::CARET_BLINK_TIMEOUT } else { Duration::default() },
                                None,
                            ),
                        );
                    }
                    event.mark_need_paint();
                }
            }
            Event::TouchBegin(touch) => {
                let state = context.state_mut::<TextState>().unwrap();
                if let Some(paragraph) = state.paragraph.as_mut() {
                    let position = paragraph.get_glyph_position_at_coordinate(touch.location()).position as usize;
                    if state.selection.try_set_empty(position) {
                        event.mark_need_paint();
                    }
                }
            }
            Event::Key(hit_key) => {
                let state = context.state_mut::<TextState>().unwrap();
                let current_caret_bounds = state.caret_bounds();
                let mut will_invalidate = false;
                match hit_key.action() {
                    KeyAction::Press | KeyAction::Repeat => {
                        match hit_key.key() {
                            Key::BackSpace if state.selection.remove() => {
                                will_invalidate = true;
                            }
                            Key::Delete if state.selection.delete() => {
                                will_invalidate = true;
                            }
                            Key::Enter => {
                                if state.selection.line_count() < self.max_lines {
                                    state.selection.insert('\n');
                                    will_invalidate = true;
                                }
                            }
                            Key::Left => {
                                if hit_key.modifiers().is_alt() {
                                    will_invalidate = state.selection.move_left_fast(hit_key.modifiers().is_shift());
                                } else {
                                    will_invalidate = state.selection.move_left(hit_key.modifiers().is_shift());
                                }
                            }
                            Key::Right => {
                                if hit_key.modifiers().is_alt() {
                                    will_invalidate = state.selection.move_right_fast(hit_key.modifiers().is_shift());
                                } else {
                                    will_invalidate = state.selection.move_right(hit_key.modifiers().is_shift());
                                }
                            }
                            Key::LeftCommand | Key::RightCommand => {
                                state.command_pressed = true;
                            }
                            Key::C if hit_key.modifiers().is_control() || state.command_pressed => {
                                state.clipboard.set_content(state.selection.text());
                            }
                            Key::V if hit_key.modifiers().is_control() || state.command_pressed => {
                                if let Some(content) = state.clipboard.content() {
                                    if let Some(text) = content.to_string() {
                                        state.selection.insert_str(&text);
                                        will_invalidate = true;
                                    }
                                }
                            }
                            Key::X if hit_key.modifiers().is_control() || state.command_pressed => {
                                if let Some(text) = state.selection.cut() {
                                    state.clipboard.set_content(text);
                                    will_invalidate = true;
                                }
                            }
                            _ => {
                                if let Some(character) = hit_key.printable_character() {
                                    state.selection.insert(character);
                                    will_invalidate = true;
                                }
                            }
                        }
                    }
                    KeyAction::Release => {
                        match hit_key.key() {
                            Key::LeftCommand | Key::RightCommand => {
                                state.command_pressed = false;
                            }
                            _ => {}
                        }
                    }
                }
                if will_invalidate {
                    state.invalidate_paragraph();
                    state.caret_active = true;
                    state.caret_opacity = None;
                    state.caret_bounds = TextInput::CARET_MOVE
                        .animate(Tween::new(current_caret_bounds, state.caret_bounds_inner()))
                        .into();
                    context.request_frame();
                    context.schedule_timer(
                        TextInput::CARET_BLINK,
                        Timer::new(TextInput::CARET_BLINK_TIMEOUT, None),
                    );
                    event.mark_need_paint();
                }
            }
            _ => {}
        }
    }

    fn paint(&self, context: &mut WidgetContext, paint: &mut PaintContext) {
        let state = context.state_mut::<TextState>().unwrap();
        let size = paint.size();
        let canvas = paint.canvas();
        let mut caret_bounds = state.caret_bounds();
        if state.caret_bounds.is_some() {
            caret_bounds.top -= 1.0;
            caret_bounds.bottom += 1.0;
        }
        canvas.save();
        canvas.translate((
            -state.horizontal_scroll(&caret_bounds, size.width),
            0.0,
        ));
        if state.selection.text().is_empty() {
            if let Some(paragraph) = &mut state.hint_paragraph {
                paragraph.paint(canvas, Point::default());
            }
        } else {
            if let Some(paragraph) = &mut state.paragraph {
                paragraph.paint(canvas, Point::default());
            }
        }
        if state.focused {
            let mut color = Color4f::from(state.text_style.color());
            color.a = state.caret_opacity();
            let mut paint = Paint::new(color, None);
            paint.set_anti_alias(true);
            let radius = caret_bounds.width() / 2.0;
            canvas.draw_round_rect(caret_bounds, radius, radius, &paint);
        }
        canvas.restore();
    }

    fn hit_test(&self, _: &WidgetContext, hit_test: &mut HitTestContext) -> bool {
        hit_test.become_responder()
    }
}

#[derive(Debug)]
struct TextSelection {
    text: String,
    begin: usize,
    end: usize,
    position: usize,
}

impl TextSelection {
    pub fn new(text: impl AsRef<str>) -> Self {
        Self {
            text: text.as_ref().to_string(),
            begin: 0,
            end: 0,
            position: 0,
        }
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) {
        self.text = text.as_ref().to_string();
        if self.position > self.text.len() {
            self.position = self.text.len();
        }
        if self.begin > self.text.len() {
            self.begin = self.text.len();
        }
        if self.end < self.begin {
            self.end = self.begin;
        }
    }

    pub fn insert(&mut self, character: char) {
        self.clear();
        self.text.insert(self.position, character);
        self.position += 1;
        self.begin = self.position;
        self.end = self.position;
    }

    pub fn insert_str(&mut self, data: impl AsRef<str>) {
        let data = data.as_ref();
        self.clear();
        self.text.insert_str(self.position, data);
        self.position += data.len();
        self.begin = self.position;
        self.end = self.position;
    }

    pub fn remove(&mut self) -> bool {
        if self.clear() {
            true
        } else if self.position > 0 {
            self.position -= 1;
            self.begin = self.position;
            self.end = self.position;
            self.text.remove(self.position);
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self) -> bool {
        if self.clear() {
            true
        } else if self.position < self.text.len() {
            self.text.remove(self.position);
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) -> bool {
        if self.end > self.begin {
            self.text.replace_range(self.begin..self.end, "");
            self.end = self.begin;
            self.position = self.begin;
            true
        } else {
            false
        }
    }

    pub fn cut(&mut self) -> Option<String> {
        if self.end > self.begin {
            let result = self.text[self.begin..self.end].to_string();
            self.clear();
            Some(result)
        } else {
            None
        }
    }

    pub fn move_left(&mut self, selecting: bool) -> bool {
        if self.position == 0 {
            return false;
        }
        if selecting {
            if self.position == self.begin {
                self.position -= 1;
                self.begin = self.position;
            } else {
                self.position -= 1;
                self.end = self.position;
            }
        } else {
            if self.position == self.begin {
                self.position -= 1;
                self.begin = self.position;
                self.end = self.position;
            } else {
                self.position = self.begin;
                self.end = self.position;
            }
        }
        true
    }

    pub fn move_right(&mut self, selecting: bool) -> bool {
        if self.position == self.text().len() {
            return false;
        }
        if selecting {
            if self.position == self.end {
                self.position += 1;
                self.end = self.position;
            } else {
                self.position += 1;
                self.begin = self.position;
            }
        } else {
            if self.position == self.end {
                self.position += 1;
                self.end = self.position;
                self.begin = self.position;
            } else {
                self.position = self.end;
                self.begin = self.position;
            }
        }
        true
    }

    pub fn move_left_fast(&mut self, selecting: bool) -> bool {
        if self.text.is_empty() { return false; }
        if self.position == 0 { return false; }
        let mut chars = self.text
            .char_indices()
            .rev()
            .skip(self.text.len() - self.position)
            .peekable();
        let (_, cur) = chars.next().unwrap();
        while let Some((index, char)) = chars.next() {
            let last_char = chars.peek().is_none();
            if !last_char && (cur.is_whitespace() || cur.is_control()) && (char.is_whitespace() || char.is_control()) {
                continue;
            }
            if !last_char && (cur.is_alphanumeric() && char.is_alphanumeric()) {
                continue;
            }
            let position = if last_char { 0 } else { index + 1 };
            if selecting {
                if self.position == self.begin {
                    self.position = position;
                    self.begin = self.position;
                } else {
                    self.position = position;
                    self.end = self.position;
                    if self.end < self.begin {
                        self.end = self.begin;
                        self.begin = self.position;
                    }
                }
            } else {
                self.position = position;
                self.begin = self.position;
                self.end = self.position;
            }
            break;
        }
        true
    }

    pub fn move_right_fast(&mut self, selecting: bool) -> bool {
        if self.text.is_empty() { return false; }
        if self.position == self.text.len() { return false; }
        let mut chars = self.text.char_indices().skip(self.position).peekable();
        let (_, cur) = chars.peek().copied().unwrap();
        while let Some((index, char)) = chars.next() {
            let last_char = chars.peek().is_none();
            if !last_char && (cur.is_whitespace() || cur.is_control()) && (char.is_whitespace() || char.is_control()) {
                continue;
            }
            if !last_char && (cur.is_alphanumeric() && char.is_alphanumeric()) {
                continue;
            }
            let position = if last_char {
                self.text.len()
            } else if self.position == index {
                index + 1
            } else {
                index
            };
            if selecting {
                if self.position == self.end {
                    self.position = position;
                    self.end = self.position;
                } else {
                    self.position = position;
                    self.begin = self.position;
                    if self.begin > self.end {
                        self.begin = self.end;
                        self.end = self.position;
                    }
                }
            } else {
                self.position = position;
                self.begin = self.position;
                self.end = self.position;
            }
            break;
        }
        true
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn len(&self) -> usize {
        self.end - self.begin
    }

    pub fn is_empty(&self) -> bool {
        self.begin == self.end
    }

    pub fn set_empty(&mut self, position: usize) {
        self.position = position;
        self.begin = position;
        self.end = position;
    }

    pub fn set(&mut self, position: usize, len: usize) {
        self.position = position;
        self.begin = position;
        self.end = position + len;
    }

    pub fn try_set_empty(&mut self, position: usize) -> bool {
        if self.begin != position || self.end != position {
            self.set_empty(position);
            true
        } else {
            false
        }
    }

    pub fn line_count(&self) -> usize {
        1 + self.text.lines().count()
    }
}

struct TextState {
    font_collection: FontCollection,
    paragraph_style: ParagraphStyle,
    text_style: GrTextStyle,
    empty_paragraph: Option<Paragraph>,
    paragraph: Option<Paragraph>,
    caret_opacity: Option<Animation<f32>>,
    caret_bounds: Option<Animation<Rect>>,
    focused: bool,
    caret_active: bool,
    selection: TextSelection,
    command_pressed: bool,
    clipboard: Clipboard,
    hint: Option<String>,
    hint_color: Color,
    hint_paragraph: Option<Paragraph>,
    highlight_color: Color,
    horizontal_scroll: f32,
}

impl TextState {
    pub fn new<'a>(
        text: impl AsRef<str>,
        font_collection: FontCollection,
        paragraph_style: ParagraphStyle,
        text_style: GrTextStyle,
        hint: impl Into<Option<&'a String>>,
        hint_color: Color,
        selection_color: Color,
    ) -> Self {
        Self {
            font_collection,
            paragraph_style,
            text_style,
            hint: hint.into().cloned(),
            hint_paragraph: None,
            hint_color,
            highlight_color: selection_color,
            paragraph: None,
            empty_paragraph: None,
            caret_opacity: None,
            caret_bounds: None,
            caret_active: false,
            selection: TextSelection::new(text),
            focused: false,
            command_pressed: false,
            clipboard: Clipboard::default(),
            horizontal_scroll: 0.0,
        }
    }

    pub fn caret_opacity(&self) -> f32 {
        self.caret_opacity.as_ref()
            .map(|a| a.value())
            .unwrap_or_else(|| if self.caret_active { 1.0 } else { 0.0 })
    }

    pub fn caret_bounds(&mut self) -> Rect {
        if let Some(bounds) = self.caret_bounds.as_ref().map(|a| a.value()) {
            bounds
        } else {
            self.caret_bounds_inner()
        }
    }

    pub fn caret_bounds_inner(&mut self) -> Rect {
        let caret_width = 2.0;
        let paragraph = if self.selection.text().is_empty() {
            self.empty_paragraph.as_mut()
        } else {
            self.paragraph.as_mut()
        }.unwrap();
        let position = self.selection.position();
        let mut new_line = false;
        let index = if position == 0 {
            position
        } else if self.selection.text().chars().nth(position - 1) == Some('\n') {
            new_line = true;
            position
        } else {
            position - 1
        };
        let line_metric = paragraph.get_line_metrics().iter().last().unwrap();
        let line_height = line_metric.height as f32;
        let boxes = paragraph.get_rects_for_range(
            index..index + 1,
            RectHeightStyle::Tight,
            RectWidthStyle::Tight,
        );
        let text_box = boxes.iter().last().unwrap();
        Rect::from_xywh(
            if position == 0 || new_line {
                text_box.rect.left()
            } else {
                text_box.rect.right()
            },
            text_box.rect.top(),
            caret_width,
            line_height,
        ).into()
    }

    pub fn horizontal_scroll(&mut self, caret_bounds: &Rect, width: f32) -> f32 {
        if self.selection.text().is_empty() {
            self.horizontal_scroll = 0.0;
        } else if caret_bounds.left() < self.horizontal_scroll {
            self.horizontal_scroll = caret_bounds.left();
        } else if caret_bounds.right() > self.horizontal_scroll + width {
            self.horizontal_scroll = caret_bounds.right() - width;
        }
        return self.horizontal_scroll;
    }

    pub fn invalidate(&mut self) {
        self.empty_paragraph = Some({
            let mut paragraph = ParagraphBuilder::new(&self.paragraph_style, self.font_collection.clone())
                .push_style(&self.text_style)
                .add_text(" ")
                .build();
            paragraph.layout(f32::INFINITY);
            paragraph
        });
        self.invalidate_paragraph();
        self.hint_paragraph = None;
        if let Some(hint) = &self.hint {
            self.hint_paragraph = Some({
                let mut foreground = Paint::default();
                foreground.set_color(self.hint_color);
                let mut hint_style = self.text_style.clone();
                hint_style.set_foreground_color(foreground);
                let mut paragraph = ParagraphBuilder::new(&self.paragraph_style, self.font_collection.clone())
                    .push_style(&hint_style)
                    .add_text(hint)
                    .build();
                paragraph.layout(f32::INFINITY);
                paragraph
            });
        }
    }

    pub fn invalidate_paragraph(&mut self) {
        self.paragraph = Some({
            let mut builder = ParagraphBuilder::new(&self.paragraph_style, self.font_collection.clone());
            if self.selection.is_empty() {
                builder.push_style(&self.text_style);
                builder.add_text(self.selection.text());
            } else {
                let mut background = Paint::default();
                background.set_color(self.highlight_color);
                let mut selection_style = self.text_style.clone();
                selection_style.set_background_color(background);
                builder.push_style(&self.text_style);
                builder.add_text(&self.selection.text()[..self.selection.begin]);
                builder.push_style(&selection_style);
                builder.add_text(&self.selection.text()[self.selection.begin..self.selection.end]);
                builder.pop();
                builder.add_text(&self.selection.text()[self.selection.end..]);
            }
            let mut paragraph = builder.build();
            paragraph.layout(f32::INFINITY);
            paragraph
        });
    }

    pub fn paragraph_height(&self) -> Option<f32> {
        if self.selection.text().is_empty() {
            self.empty_paragraph.as_ref()?.height().into()
        } else {
            self.paragraph.as_ref()?.height().into()
        }
    }
}
