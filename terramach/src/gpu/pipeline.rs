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

use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

use terramach_graphics::{Display, Size};

use crate::gpu::{Frame, TextureRegistry, RenderTexture, TextureId, Texture};
use crate::platform::VSync;
use crate::Id;

enum Command {
    Push(Frame),
    Terminate,
    Resize(Size),
    RegisterTexture(Id, Box<dyn RenderTexture>),
    UnregisterTexture(Id),
    InvalidateTexture(Id),
    UpdateTexture(Id),
}

pub struct Pipeline {
    sender: Sender<Command>,
}

impl Pipeline {
    pub fn new<D>(vsync: VSync, display: D) -> Self where D: 'static + Display {
        let (sender, receiver) = channel();
        let mut render_pipeline = RenderPipeline::new(
            vsync,
            Box::new(display),
            sender.clone(),
            receiver,
        );
        std::thread::spawn(move || render_pipeline.render());
        Pipeline { sender }
    }

    pub fn resize(&mut self, display_size: impl Into<Size>) {
        let _ = self.sender.send(Command::Resize(display_size.into()));
    }

    pub fn share(&self) -> SharedPipeline {
        SharedPipeline::new(&self.sender)
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Terminate);
    }
}

#[derive(Clone)]
pub struct SharedPipeline {
    sender: Sender<Command>,
}

impl SharedPipeline {
    fn new(sender: &Sender<Command>) -> Self {
        SharedPipeline {
            sender: sender.clone(),
        }
    }

    pub fn submit_frame(&mut self, frame: Frame) {
        let _ = self.sender.send(Command::Push(frame));
    }

    pub fn register_texture<T: 'static + RenderTexture + Send>(&mut self, id: TextureId, texture: T) {
        let _ = self.sender.send(Command::RegisterTexture(id, Box::new(texture)));
    }

    pub fn update_texture(&mut self, id: Id) {
        let _ = self.sender.send(Command::UpdateTexture(id));
    }

    pub fn invalidate_texture(&mut self, id: Id) {
        let _ = self.sender.send(Command::InvalidateTexture(id));
    }

    pub fn unregister_texture(&mut self, id: Id) {
        let _ = self.sender.send(Command::UnregisterTexture(id));
    }
}

struct RenderPipeline {
    vsync: VSync,
    display: Box<dyn Display>,
    sender: Sender<Command>,
    receiver: Receiver<Command>,
}

impl RenderPipeline {
    pub fn new(
        vsync: VSync,
        display: Box<dyn Display>,
        sender: Sender<Command>,
        receiver: Receiver<Command>,
    ) -> Self {
        RenderPipeline {
            vsync,
            display,
            sender,
            receiver,
        }
    }

    pub fn render(&mut self) -> Option<()> {
        self.display.make_current();
        let mut surface = self.display.new_surface()?;
        let mut frame = None;
        let mut textures = TextureRegistry::new();

        'main: while let Ok(command) = self.receiver.recv() {
            let mut will_draw_frame = false;
            let mut will_invalidate_surface = false;

            self.vsync.wait()?;

            let mut commands = vec![command];
            loop {
                match self.receiver.try_recv() {
                    Ok(command) => commands.push(command),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => break 'main,
                }
            }

            for command in commands {
                match command {
                    Command::Terminate => break,
                    Command::Push(new_frame) => {
                        will_draw_frame = true;
                        frame = Some(new_frame);
                    }
                    Command::Resize(size) => {
                        will_invalidate_surface = true;
                        will_draw_frame = true;
                        self.display.resize(size);
                    }
                    Command::RegisterTexture(id, render_texture) => {
                        if let Some(gl) = self.display.gl() {
                            if let Some(context) = self.display.gpu_context() {
                                will_draw_frame = true;
                                let texture = Texture::new(
                                    gl,
                                    context,
                                    id,
                                    SharedPipeline::new(&self.sender),
                                    render_texture,
                                );
                                textures.register(id, texture);
                            }
                        }
                    }
                    Command::InvalidateTexture(id) => {
                        if let Some(texture) = textures.texture(id) {
                            will_draw_frame = true;
                            texture.mark_need_render();
                        }
                    }
                    Command::UpdateTexture(id) => {
                        if let Some(texture) = textures.texture(id) {
                            texture.update();
                        }
                    }
                    Command::UnregisterTexture(id) => {
                        if textures.unregister(id) {
                            will_draw_frame = true;
                        }
                    }
                }
            }

            if will_invalidate_surface {
                surface = self.display.new_surface()?;
            }

            if will_draw_frame {
                if let Some(frame) = &frame {
                    let mut canvas = surface.canvas();
                    canvas.clear(0);
                    frame.draw(canvas, self.display.size(), &mut textures);
                    canvas.flush();
                    self.display.present_current();
                }
            }
        }

        Some(())
    }
}

unsafe impl Send for RenderPipeline {}
