#![allow(dead_code)]

mod net;
mod renderer;
mod serialize;

use crate::renderer::Renderer;
use anyhow::Result;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct App {
    window: Window,
    renderer: Renderer,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_min_inner_size(PhysicalSize::new(320, 180))
            .build(&event_loop)?;

        let renderer = Renderer::new(&window)?;

        Ok(Self { window, renderer })
    }

    fn handle_resize(&mut self, size: PhysicalSize<u32>) -> Option<ControlFlow> {
        self.renderer.resize(size);

        None
    }

    fn handle_event(&mut self, event: WindowEvent) -> Option<ControlFlow> {
        match event {
            WindowEvent::CloseRequested => Some(ControlFlow::Exit),
            WindowEvent::Resized(size) => self.handle_resize(size),
            _ => None,
        }
    }

    fn repaint(&mut self) {
        self.renderer.render().unwrap();
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new();

    let mut app = App::new(&event_loop)?;

    event_loop.run(move |event, _, control_flow| {
        use Event::*;

        *control_flow = ControlFlow::Poll;

        match event {
            WindowEvent { event, .. } => {
                if let Some(cf) = app.handle_event(event) {
                    *control_flow = cf;
                }
            }
            RedrawRequested(_) => app.repaint(),
            _ => (),
        }
    });
}
