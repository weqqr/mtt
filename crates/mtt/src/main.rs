use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::time::Duration;

use mtt_protocol::clientbound::ClientBound;
use mtt_protocol::serverbound::{self, ServerBound};
use mtt_renderer::Renderer;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub struct ClientThread {
    client: mtt_protocol::Client,
    socket: UdpSocket,

    serverbound_tx: mpsc::Sender<ServerBound>,
    serverbound_rx: mpsc::Receiver<ServerBound>,

    clientbound_tx: mpsc::Sender<ClientBound>,
    clientbound_rx: mpsc::Receiver<ClientBound>,
}

impl ClientThread {
    pub fn new(address: String) -> Self {
        let client = mtt_protocol::Client::new();

        let (clientbound_tx, clientbound_rx) = mpsc::channel();
        let (serverbound_tx, serverbound_rx) = mpsc::channel();

        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.connect(address).unwrap();
        socket.set_nonblocking(true).unwrap();

        Self {
            client,
            socket,

            serverbound_tx,
            serverbound_rx,

            clientbound_tx,
            clientbound_rx,
        }
    }

    #[must_use]
    fn run(mut self) -> Client {
        let _ = std::thread::spawn(move || loop {
            let mut buf = [0; 1536];

            // Receive all packets from remote server
            loop {
                let len = match self.socket.recv(&mut buf) {
                    Ok(len) => len,
                    Err(err) => match err {
                        e if e.kind() == ErrorKind::WouldBlock => break,
                        _ => panic!("{}", err),
                    },
                };

                println!("Receive: {:?}", &buf[..len]);

                self.client
                    .handle_input(mtt_protocol::Input::Receive(&buf[..len]))
                    .unwrap();
            }

            // Send all queued packets from client
            for packet in self.serverbound_rx.try_iter() {
                println!("Send: {:?}", packet);
                self.client.handle_input(mtt_protocol::Input::Packet(packet)).unwrap();
            }

            // Handle I/O
            for output in self.client.poll_output() {
                match output {
                    mtt_protocol::Output::Packet(packet) => {
                        self.clientbound_tx.send(packet).unwrap();
                    }
                    mtt_protocol::Output::Send(data) => {
                        self.socket.send(&data).unwrap();
                    }
                    mtt_protocol::Output::None => {}
                }
            }

            std::thread::sleep(Duration::from_millis(32));
        });

        Client {
            serverbound_tx: self.serverbound_tx,
            clientbound_rx: self.clientbound_rx,
        }
    }
}

pub struct Client {
    serverbound_tx: mpsc::Sender<ServerBound>,
    clientbound_rx: mpsc::Receiver<ClientBound>,
}

impl Client {
    pub fn receive(&self) -> impl Iterator<Item = ClientBound> + '_ {
        self.clientbound_rx.try_iter()
    }

    pub fn send<S: Into<ServerBound>>(&self, packet: S) {
        self.serverbound_tx.send(packet.into()).unwrap()
    }
}

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().with_title("mtt").build(&event_loop).unwrap();
    let renderer = Renderer::new(window).unwrap();

    let address = std::env::args().nth(1).expect("address required");
    let client_thread = ClientThread::new(address);
    let client = client_thread.run();
    client.send(serverbound::Handshake {});

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => renderer.resize(size),
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                    Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                _ => (),
            },
            Event::MainEventsCleared => {
                renderer.render();
            }
            _ => (),
        }
    });
}
