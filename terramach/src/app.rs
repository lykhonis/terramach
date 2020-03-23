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
use crate::{AppEvent, AppEvents, DrawContext, Event, EventResponder, RenderTree, Touches, BoxedWidget};

use terramach_graphics::{Canvas, Display, PictureRecorder, Rect, Size};

pub(crate) fn run_app<D: 'static + Display>(
    mut run_loop: RunLoop,
    mut events: AppEvents,
    display: D,
    content: impl Into<BoxedWidget>,
) {
    let display = Box::new(display);
    let content = content.into();
    let mut app_run_loop = run_loop.share();
    let mut current_size = display.size();
    let mut vsync = VSync::default();
    let mut pipeline = Pipeline::new(vsync.clone(), display);
    let mut tree = RenderTree::new(pipeline.share(), content);
    let mut hover_responders: Vec<EventResponder> = Vec::new();
    let display_metrics = DisplayMetrics::default();
    let mut current_responder: Option<EventResponder> = None;
    let mut touches = Touches::new();
    let mut cursors = Cursors::new();
    run_loop.add_observer(move || {
        if let Some(events) = events.poll() {
            let mut issue_touch = false;
            for event in events {
                match event {
                    AppEvent::Quit => {
                        app_run_loop.stop();
                    }
                    AppEvent::Resize(size) => {
                        current_size = size;
                        tree.invalidate();
                        pipeline.resize(size);
                    }
                    AppEvent::Scroll(delta) => {
                        if let Some(responder) = hover_responders.last() {
                            tree.emit_event(
                                responder.widget(),
                                Event::Scroll(delta * display_metrics.device_pixel_ratio()),
                            );
                        }
                    }
                    AppEvent::Focus(focused) => {
                        if let Some(responder) = &current_responder {
                            tree.emit_event(
                                responder.widget(),
                                Event::Focus(focused),
                            );
                        }
                    }
                    AppEvent::TouchBegin(touch) => {
                        let new_responder = tree.hit_test(None, touch.location());
                        if let Some(responder) = &current_responder {
                            if let Some(new_responder) = new_responder {
                                if new_responder.widget() != responder.widget() {
                                    tree.emit_event(responder.widget(), Event::ResignedResponder);
                                    tree.emit_event(new_responder.widget(), Event::BecameResponder);
                                    current_responder = Some(new_responder);
                                }
                            } else {
                                tree.emit_event(responder.widget(), Event::ResignedResponder);
                                current_responder = None;
                            }
                        } else if let Some(responder) = new_responder {
                            tree.emit_event(responder.widget(), Event::BecameResponder);
                            current_responder = Some(responder);
                        }

                        if let Some(responder) = &current_responder {
                            touches.update(responder.transform_touch(&touch));
                            issue_touch = true;
                            tree.emit_event(
                                responder.widget(),
                                Event::TouchBegin(responder.transform_touch(&touch)),
                            );
                        }
                    }
                    AppEvent::TouchUpdate(touch) => {
                        if let Some(responder) = &current_responder {
                            touches.update(responder.transform_touch(&touch));
                            issue_touch = true;
                            tree.emit_event(
                                responder.widget(),
                                Event::TouchUpdate(responder.transform_touch(&touch)),
                            );
                        }
                    }
                    AppEvent::TouchEnd(touch) => {
                        if let Some(responder) = &current_responder {
                            touches.remove(touch.id());
                            issue_touch = true;
                            tree.emit_event(
                                responder.widget(),
                                Event::TouchEnd(responder.transform_touch(&touch)),
                            );
                        }
                    }
                    AppEvent::Hover(location) => {
                        // scan through hover stack from first, if one is not hover, the rest aren't either
                        for i in 0..hover_responders.len() {
                            let responder = &hover_responders[i];
                            let hit_responder = tree.hit_test(
                                responder.widget(),
                                responder.transform_point(location),
                            );
                            if hit_responder.is_none() {
                                for j in hover_responders.len() - 1..=i {
                                    let responder = &hover_responders[j];
                                    if responder.has_cursor() {
                                        cursors.pop();
                                    }
                                    tree.emit_event(responder.widget(), Event::Leave);
                                }
                                hover_responders = hover_responders.drain(0..i).collect();
                                break;
                            }
                        }

                        // check last hover, whether it hits a child, if so add it to the stack, otherwise hit whole tree
                        if let Some(responder) = hover_responders.last() {
                            let hit_responder = tree.hit_test(
                                responder.widget(),
                                responder.transform_point(location),
                            );
                            if let Some(hit_responder) = hit_responder {
                                if hit_responder.widget() != responder.widget() {
                                    if let Some(cursor) = hit_responder.cursor() {
                                        cursors.push(cursor);
                                    }
                                    tree.emit_event(hit_responder.widget(), Event::Enter);
                                    hover_responders.push(hit_responder);
                                }
                            }
                        } else if let Some(responder) = tree.hit_test(None, location) {
                            if let Some(cursor) = responder.cursor() {
                                cursors.push(cursor);
                            }
                            tree.emit_event(responder.widget(), Event::Enter);
                            hover_responders.push(responder);
                        }

                        if let Some(responder) = hover_responders.last() {
                            tree.emit_event(
                                responder.widget(),
                                Event::Hover(responder.transform_point(location)),
                            );
                        }
                    }
                    AppEvent::Frame(timestamp) => tree.emit_event(None, Event::Frame(timestamp)),
                    AppEvent::Key(key) => {
                        if let Some(responder) = &current_responder {
                            tree.emit_event(responder.widget(), Event::Key(key));
                        }
                    }
                }
            }
            if issue_touch {
                if let Some(responder) = &current_responder {
                    tree.emit_event(
                        responder.widget(),
                        Event::Touch(touches.clone()),
                    );
                }
            }
        }

        tree.render(current_size);

        if let Some(timer_time) = tree.request_next_timer_time() {
            app_run_loop.set_next_wakeup_in(timer_time);
        }

        if tree.needs_frame() {
            let mut event_emitter = events.emitter();
            let mut run_loop = app_run_loop.clone();
            vsync.request_frame(move |timestamp| {
                event_emitter.emit_event(AppEvent::Frame(timestamp));
                run_loop.wakeup();
            });
        }
    });
    run_loop.run();
}
