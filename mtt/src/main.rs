#![allow(dead_code)]

mod net;
mod renderer;
mod serialize;

use crate::renderer::Renderer;
use anyhow::Result;
use tokio::sync::mpsc;
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

    fn handle_event(&mut self, event: Event<()>) -> Option<ControlFlow> {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => Some(ControlFlow::Exit),
                WindowEvent::Resized(size) => self.handle_resize(size),
                _ => None,
            },
            Event::MainEventsCleared => {
                println!("repaint");
                self.repaint();
                None
            }
            Event::RedrawRequested(_) => None,
            _ => None,
        }
    }

    fn repaint(&mut self) {
        self.renderer.render().unwrap();
    }
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;

    let event_loop = EventLoop::new();
    let mut app = App::new(&event_loop)?;

    let (event_tx, mut event_rx) = mpsc::channel(1);
    let (control_tx, mut control_rx) = mpsc::channel(1);

    rt.spawn(async move {
        loop {
            let event = event_rx.recv().await.unwrap();
            let control_flow = app.handle_event(event);

            if let Some(cf) = control_flow {
                control_tx.send(cf).await.unwrap();
            }

            if let Some(ControlFlow::Exit) = control_flow {
                break;
            }
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if !event_tx.is_closed() {
            event_tx.blocking_send(event.to_static().unwrap()).unwrap();
        }

        if let Ok(cf) = control_rx.try_recv() {
            *control_flow = cf;
        }
    });
}
