#![allow(dead_code)]

mod net;
mod renderer;
mod serialize;

use crate::net::clientbound::ClientBound;
use crate::net::serverbound::ServerBound;
use crate::net::Connection;
use crate::renderer::Renderer;
use crate::serialize::RawBytes16;
use anyhow::Result;
use sha2::Sha256;
use srp::client::{srp_private_key, SrpClient};
use tokio::net::ToSocketAddrs;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::watch;
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
    clientbound_rx: Receiver<ClientBound>,
    disconnect_tx: watch::Sender<bool>,
    player_name: String,
}

impl App {
    pub fn new(runtime: Runtime, event_loop: &EventLoop<()>) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_min_inner_size(PhysicalSize::new(320, 180))
            .build(&event_loop)?;

        let renderer = runtime.block_on(Renderer::new(window))?;

        let (serverbound_tx, serverbound_rx) = mpsc::channel(100);
        let (clientbound_tx, clientbound_rx) = mpsc::channel(100);
        let (disconnect_tx, disconnect_rx) = watch::channel(false);

        let server_address = std::env::args().nth(1).unwrap();
        let player_name = std::env::args().nth(2).unwrap();

        let connection_task = runtime.spawn(connection_task(
            server_address,
            serverbound_rx,
            clientbound_tx,
            disconnect_rx,
            player_name.clone(),
        ));

        Ok(Self {
            renderer,
            serverbound_tx,
            runtime,
            connection_task,
            clientbound_rx,
            disconnect_tx,
            player_name,
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

    fn perform_auth(&mut self) -> Result<()> {
        // FIXME: use random values
        let a = [5; 64];
        let srp_client = SrpClient::<Sha256>::new(&a, &srp::groups::G_2048);
        let a = srp_client.get_a_pub();

        let packet = ServerBound::SrpBytesA {
            data: RawBytes16(a),
            based_on: 1,
        };

        self.serverbound_tx.blocking_send(packet)?;

        let (salt, b_pub) = loop {
            let response = self.clientbound_rx.blocking_recv().unwrap();
            match response {
                ClientBound::Hello { .. } => {
                    println!("Ignoring extraneous ClientBound::Hello during auth");
                }
                ClientBound::SrpBytesSB { s, b } => break (s.0, b.0),
                packet => self.handle_packet(packet)?,
            };
        };

        let private_key =
            srp_private_key::<Sha256>(&self.player_name.as_bytes().to_ascii_lowercase(), b"", salt.as_slice());
        let verifier = srp_client
            .process_reply_with_username_and_salt(self.player_name.as_bytes(), &salt, &private_key, &b_pub)
            .unwrap();
        let packet = ServerBound::SrpBytesM {
            data: RawBytes16(verifier.get_proof().to_vec()),
        };
        self.serverbound_tx.blocking_send(packet)?;

        Ok(())
    }

    fn handle_packet(&mut self, packet: ClientBound) -> Result<()> {
        match packet {
            ClientBound::Hello { .. } => self.perform_auth()?,
            _ => println!("Ignoring {:?}", packet),
        }

        Ok(())
    }

    fn update(&mut self) {
        while let Ok(clientbound) = self.clientbound_rx.try_recv() {
            self.handle_packet(clientbound).unwrap();
        }
    }

    fn repaint(&mut self) {
        self.renderer.render().unwrap();
    }

    fn shutdown(&mut self) {
        self.disconnect_tx.send(true).unwrap();
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
    mut disconnect_rx: watch::Receiver<bool>,
    player_name: String,
) {
    let conn = Connection::connect(address, player_name);
    let conn = tokio::time::timeout(Duration::from_secs(5), conn);

    let conn = match conn.await {
        Ok(conn) => conn,
        Err(_) => panic!("connection timed out"),
    };

    let (mut conn, hello) = conn.unwrap();

    clientbound_tx.send(hello).await.unwrap();

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
            _ = disconnect_rx.changed() => {
                conn.send_disconnect().await.unwrap();
            }
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

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
