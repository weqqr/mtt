#![allow(dead_code)]

mod client;
mod media;
mod net;
mod renderer;

use crate::client::Client;
use crate::media::MediaStorage;
use crate::net::Credentials;
use crate::renderer::Renderer;
use anyhow::Result;
use mtt_core::game::Game;
use mtt_core::world::WorldState;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub struct SharedResources {
    media: MediaStorage,
    renderer: Renderer,
}

pub struct Connecting {}

impl Connecting {
    pub fn new() -> Self {
        Self {}
    }

    fn run(self, resources: &mut SharedResources, events: &mut Receiver<Event<()>>) -> AppState {
        let address = std::env::args().nth(1).unwrap();
        let name = std::env::args().nth(2).unwrap();

        let mut client = Client::connect(
            address,
            Credentials {
                name,
                password: "".to_owned(),
            },
        );

        let mut game = Game::new();
        let mut world_state = WorldState::new();

        while let Some(event) = events.blocking_recv() {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => return AppState::Exit,
                    WindowEvent::Resized(size) => {
                        resources.renderer.resize(size.clone());
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    client.process_packets(&mut resources.media, &mut game, &mut world_state);
                    resources.renderer.render().unwrap();

                    if client.is_ready() {
                        return AppState::InGame(InGame::new(client, game, world_state));
                    }
                }
                _ => (),
            }
        }

        AppState::Exit
    }
}

pub struct InGame {
    client: Client,
    game: Game,
    world_state: WorldState,
}

impl InGame {
    pub fn new(client: Client, game: Game, world_state: WorldState) -> Self {
        Self {
            client,
            game,
            world_state,
        }
    }

    fn run(mut self, resources: &mut SharedResources, events: &mut Receiver<Event<()>>) -> AppState {
        while let Some(event) = events.blocking_recv() {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => return AppState::Exit,
                    WindowEvent::Resized(size) => {
                        resources.renderer.resize(size.clone());
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    self.client
                        .process_packets(&mut resources.media, &mut self.game, &mut self.world_state);
                    resources.renderer.render().unwrap();
                }
                _ => (),
            }
        }

        AppState::Exit
    }
}

pub enum AppState {
    Connecting(Connecting),
    InGame(InGame),
    Exit,
}

impl AppState {
    pub fn run(mut self, resources: &mut SharedResources, mut events: Receiver<Event<()>>, exit: oneshot::Sender<()>) {
        loop {
            self = match self {
                AppState::Connecting(connecting) => connecting.run(resources, &mut events),
                AppState::InGame(in_game) => in_game.run(resources, &mut events),
                AppState::Exit => {
                    exit.send(()).unwrap();
                    return;
                }
            };
        }
    }
}

pub struct App {
    resources: SharedResources,
    state: AppState,
    events: Receiver<Event<'static, ()>>,
    exit: oneshot::Sender<()>,
}

impl App {
    pub async fn new(
        event_loop: &EventLoop<()>,
        events: Receiver<Event<'static, ()>>,
        exit: oneshot::Sender<()>,
    ) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_min_inner_size(PhysicalSize::new(320, 180))
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(event_loop)?;

        let media = MediaStorage::new()?;
        let renderer = Renderer::new(window).await?;

        let resources = SharedResources { media, renderer };

        Ok(Self {
            resources,
            state: AppState::Connecting(Connecting::new()),
            events,
            exit,
        })
    }

    fn run(mut self, runtime: Runtime) {
        let _enter = runtime.enter();
        self.state.run(&mut self.resources, self.events, self.exit);
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;

    let event_loop = EventLoop::new();

    let (events_tx, events_rx) = tokio::sync::mpsc::channel(10);
    let (exit_tx, mut exit_rx) = oneshot::channel();
    let app = runtime.block_on(App::new(&event_loop, events_rx, exit_tx))?;

    std::thread::spawn(move || app.run(runtime));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Err(_) = events_tx.blocking_send(event.to_static().unwrap()) {
            *control_flow = ControlFlow::Exit;
        }

        if let Ok(_) = exit_rx.try_recv() {
            *control_flow = ControlFlow::Exit;
        }
    });
}
