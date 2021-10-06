#![allow(dead_code)]

mod net;
mod renderer;
mod serialize;

use crate::net::clientbound::ClientBound;
use crate::net::serverbound::ServerBound;
use crate::net::Connection;
use crate::renderer::Renderer;
use anyhow::Result;
use tokio::net::ToSocketAddrs;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::Duration;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub struct App {
    renderer: Renderer,
    serverbound_tx: Sender<ServerBound>,
    runtime: Runtime,
    connection_task: JoinHandle<()>,
}

impl App {
    pub fn new(runtime: Runtime, event_loop: &EventLoop<()>) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_min_inner_size(PhysicalSize::new(320, 180))
            .build(&event_loop)?;

        let renderer = runtime.block_on(Renderer::new(window))?;

        let (serverbound_tx, serverbound_rx) = mpsc::channel(1);
        let (clientbound_tx, clientbound_rx) = mpsc::channel(100);

        let connection_task = runtime.spawn(connection_task(
            std::env::args().nth(1).unwrap(),
            serverbound_rx,
            clientbound_tx,
            std::env::args().nth(2).unwrap(),
        ));

        Ok(Self {
            renderer,
            serverbound_tx,
            runtime,
            connection_task,
        })
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

pub enum ConnectionStage {
    Handshake,
    Auth,
}

async fn connection_task<A: ToSocketAddrs>(
    address: A,
    mut serverbound_rx: Receiver<ServerBound>,
    clientbound_tx: Sender<ClientBound>,
    player_name: String,
) {
    let conn = Connection::connect(address, player_name);
    let conn = tokio::time::timeout(Duration::from_secs(5), conn);

    let conn = match conn.await {
        Ok(conn) => conn,
        Err(_) => panic!("connection timed out"),
    };

    let mut conn = conn.unwrap();
    loop {
        tokio::select! {
            packet = serverbound_rx.recv() => {
                let packet = packet.unwrap();
                conn.send_packet(packet, false, 1).await.unwrap();
            }
            packet = conn.receive_packet() => {
                let packet = packet.unwrap();
                if let Some(clientbound) = packet.body {
                    clientbound_tx.send(clientbound).await.unwrap();
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;

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
                app.repaint();
            }
            _ => (),
        }
    });
}
