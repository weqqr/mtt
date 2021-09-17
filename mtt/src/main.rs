#![allow(dead_code)]

mod net;
mod renderer;
mod serialize;

use anyhow::Result;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct App {
    window: Window,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_min_inner_size(PhysicalSize::new(320, 180))
            .build(&event_loop)?;

        Ok(Self { window })
    }

    fn handle_event(&mut self, event: WindowEvent) -> Option<ControlFlow> {
        match event {
            WindowEvent::CloseRequested => Some(ControlFlow::Exit),
            _ => None,
        }
    }

    fn repaint(&mut self) {

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
            },
            RedrawRequested(_) | MainEventsCleared => app.repaint(),
            _ => (),
        }
    });
}
