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

use crate::gpu::Pipeline;
use crate::platform::{DisplayMetrics, RunLoop, SharedRunLoop, VSync, Cursors};
use crate::{
    AppEvent, AppEvents, DrawContext, Event, EventResponder, RenderTree, Touches, Widget,
};
use terramach_graphics::{Canvas, Display, PictureRecorder, Rect, Size};

pub struct App {
    state: AppState,
    run_loop: RunLoop,
}

impl App {
    pub fn new<D, W>(
        run_loop: impl Into<Option<RunLoop>>,
        events: impl Into<Option<AppEvents>>,
        display: D,
        content: W,
    ) -> Self where D: 'static + Display,
                    W: 'static + Widget {
        let run_loop = run_loop.into().unwrap_or_else(|| RunLoop::new());
        let shared_run_loop = run_loop.share();
        let vsync = VSync::default();
        let size = display.size();
        let pipeline = Pipeline::new(vsync.clone(), display);
        let tree = RenderTree::new(pipeline.share(), Box::new(content));
        App {
            run_loop,
            state: AppState::new(
                size,
                vsync,
                pipeline,
                events.into().unwrap_or(AppEvents::new()),
                shared_run_loop,
                tree,
            ),
        }
    }

    pub fn run(self) {
        let mut run_loop = self.run_loop;
        let mut state = self.state;
        run_loop.add_observer(move || state.run());
        run_loop.run();
    }
}

struct AppState {
    size: Size,
    vsync: VSync,
    pipeline: Pipeline,
    events: AppEvents,
    tree: RenderTree,
    touches: Touches,
    responder: Option<EventResponder>,
    hover_responders: Vec<EventResponder>,
    display_metrics: DisplayMetrics,
    run_loop: SharedRunLoop,
    cursors: Cursors,
}

impl AppState {
    fn new(
        size: Size,
        vsync: VSync,
        pipeline: Pipeline,
        events: AppEvents,
        run_loop: SharedRunLoop,
        tree: RenderTree,
    ) -> Self {
        AppState {
            size,
            vsync,
            pipeline,
            events,
            tree,
            run_loop,
            touches: Touches::new(),
            responder: None,
            hover_responders: Vec::new(),
            display_metrics: DisplayMetrics::default(),
            cursors: Cursors::default(),
        }
    }

    fn run(&mut self) {
        if let Some(events) = self.events.poll() {
            let mut issue_touch = false;
            for event in events {
                match event {
                    AppEvent::Quit => {
                        self.run_loop.stop();
                    }
                    AppEvent::Resize(size) => {
                        self.size = size;
                        self.tree.invalidate();
                        self.pipeline.resize(size);
                    }
                    AppEvent::Scroll(delta) => {
                        if let Some(responder) = self.hover_responders.last() {
                            self.tree.emit_event(
                                responder.widget(),
                                Event::Scroll(delta * self.display_metrics.device_pixel_ratio()),
                            );
                        }
                    }
                    AppEvent::Focus(focused) => {
                        if let Some(responder) = &self.responder {
                            self.tree.emit_event(
                                responder.widget(),
                                Event::Focus(focused),
                            );
                        }
                    }
                    AppEvent::TouchBegin(touch) => {
                        let new_responder = self.tree.hit_test(None, touch.location());
                        if let Some(responder) = &self.responder {
                            if let Some(new_responder) = new_responder {
                                if new_responder.widget() != responder.widget() {
                                    self.tree.emit_event(responder.widget(), Event::ResignedResponder);
                                    self.tree.emit_event(new_responder.widget(), Event::BecameResponder);
                                    self.responder = Some(new_responder);
                                }
                            } else {
                                self.tree.emit_event(responder.widget(), Event::ResignedResponder);
                                self.responder = None;
                            }
                        } else if let Some(responder) = new_responder {
                            self.tree.emit_event(responder.widget(), Event::BecameResponder);
                            self.responder = Some(responder);
                        }

                        if let Some(responder) = &self.responder {
                            self.touches.update(responder.transform_touch(&touch));
                            issue_touch = true;
                            self.tree.emit_event(
                                responder.widget(),
                                Event::TouchBegin(responder.transform_touch(&touch)),
                            );
                        }
                    }
                    AppEvent::TouchUpdate(touch) => {
                        if let Some(responder) = &self.responder {
                            self.touches.update(responder.transform_touch(&touch));
                            issue_touch = true;
                            self.tree.emit_event(
                                responder.widget(),
                                Event::TouchUpdate(responder.transform_touch(&touch)),
                            );
                        }
                    }
                    AppEvent::TouchEnd(touch) => {
                        if let Some(responder) = &self.responder {
                            self.touches.remove(touch.id());
                            issue_touch = true;
                            self.tree.emit_event(
                                responder.widget(),
                                Event::TouchEnd(responder.transform_touch(&touch)),
                            );
                        }
                    }
                    AppEvent::Hover(location) => {
                        // scan through hover stack from first, if one is not hover, the rest aren't either
                        for i in 0..self.hover_responders.len() {
                            let responder = &self.hover_responders[i];
                            let hit_responder = self.tree.hit_test(
                                responder.widget(),
                                responder.transform_point(location),
                            );
                            if hit_responder.is_none() {
                                for j in self.hover_responders.len() - 1..=i {
                                    let responder = &self.hover_responders[j];
                                    if responder.has_cursor() {
                                        self.cursors.pop();
                                    }
                                    self.tree.emit_event(responder.widget(), Event::Leave);
                                }
                                self.hover_responders = self.hover_responders.drain(0..i).collect();
                                break;
                            }
                        }

                        // check last hover, whether it hits a child, if so add it to the stack, otherwise hit whole tree
                        if let Some(responder) = self.hover_responders.last() {
                            let hit_responder = self.tree.hit_test(
                                responder.widget(),
                                responder.transform_point(location),
                            );
                            if let Some(hit_responder) = hit_responder {
                                if hit_responder.widget() != responder.widget() {
                                    if let Some(cursor) = hit_responder.cursor() {
                                        self.cursors.push(cursor);
                                    }
                                    self.tree.emit_event(hit_responder.widget(), Event::Enter);
                                    self.hover_responders.push(hit_responder);
                                }
                            }
                        } else if let Some(responder) = self.tree.hit_test(None, location) {
                            if let Some(cursor) = responder.cursor() {
                                self.cursors.push(cursor);
                            }
                            self.tree.emit_event(responder.widget(), Event::Enter);
                            self.hover_responders.push(responder);
                        }

                        if let Some(responder) = self.hover_responders.last() {
                            self.tree.emit_event(
                                responder.widget(),
                                Event::Hover(responder.transform_point(location)),
                            );
                        }
                    }
                    AppEvent::Frame(timestamp) => self.tree.emit_event(None, Event::Frame(timestamp)),
                    AppEvent::Key(key) => {
                        if let Some(responder) = &self.responder {
                            self.tree.emit_event(responder.widget(), Event::Key(key));
                        }
                    }
                }
            }
            if issue_touch {
                if let Some(responder) = &self.responder {
                    self.tree.emit_event(
                        responder.widget(),
                        Event::Touch(self.touches.clone()),
                    );
                }
            }
        }

        self.tree.render(self.size);

        if let Some(timer_time) = self.tree.request_next_timer_time() {
            self.run_loop.set_next_wakeup_in(timer_time);
        }

        if self.tree.needs_frame() {
            let mut event_emitter = self.events.emitter();
            let mut run_loop = self.run_loop.clone();
            self.vsync.request_frame(move |timestamp| {
                event_emitter.emit_event(AppEvent::Frame(timestamp));
                run_loop.wakeup();
            });
        }
    }
}
