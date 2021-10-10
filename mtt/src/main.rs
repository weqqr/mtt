#![allow(dead_code)]

mod math;
mod net;
mod renderer;
mod serialize;

use crate::net::clientbound::ClientBound;
use crate::net::{connect, Request, Response};
use crate::renderer::Renderer;
use anyhow::Result;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub struct App {
    renderer: Renderer,
    runtime: Runtime,
    player_name: String,
    request_tx: mpsc::Sender<Request>,
    response_rx: mpsc::Receiver<Response>,
}

impl App {
    pub fn new(runtime: Runtime, event_loop: &EventLoop<()>) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_min_inner_size(PhysicalSize::new(320, 180))
            .build(event_loop)?;

        let renderer = runtime.block_on(Renderer::new(window))?;

        let server_address = std::env::args().nth(1).unwrap();
        let player_name = std::env::args().nth(2).unwrap();

        let (request_tx, response_rx) = connect(server_address, player_name.clone());

        Ok(Self {
            renderer,
            runtime,
            player_name,
            request_tx,
            response_rx,
        })
    }

    fn handle_event(&mut self, event: WindowEvent) -> Option<ControlFlow> {
        match event {
            WindowEvent::CloseRequested => Some(ControlFlow::Exit),
            WindowEvent::Resized(size) => {
                self.renderer.resize(size);
                None
            }
            _ => None,
        }
    }

    fn handle_packet(&mut self, packet: ClientBound) -> Result<()> {
        match packet {
            ClientBound::Hello { .. } => self.request_tx.blocking_send(Request::Authenticate {
                player_name: self.player_name.clone(),
                password: "".to_string(),
            })?,
            _ => println!("Ignoring {:?}", packet),
        }

        Ok(())
    }

    fn update(&mut self) {
        while let Ok(response) = self.response_rx.try_recv() {
            match response {
                Response::Disconnect => (),
                Response::Error(err) => panic!("{}", err),
                Response::Receive(packet) => self.handle_packet(packet).unwrap(),
            }
        }
    }

    fn repaint(&mut self) {
        self.renderer.render().unwrap();
    }

    fn shutdown(&mut self) {
        let _ = self.request_tx.blocking_send(Request::Disconnect);
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    let _enter = runtime.enter();

    let event_loop = EventLoop::new();
    let mut app = App::new(runtime, &event_loop)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => {
                if let Some(cf) = app.handle_event(event) {
                    *control_flow = cf;
                }
            }
            Event::MainEventsCleared => {
                app.update();
                app.repaint();
            }
            Event::LoopDestroyed => {
                app.shutdown();
            }
            _ => (),
        }
    });
}
