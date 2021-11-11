#![allow(dead_code)]

mod client;
mod net;
mod renderer;

use crate::client::Client;
use crate::net::Credentials;
use crate::renderer::{Renderer, View};
use anyhow::Result;
use mtt_core::game::Game;
use mtt_core::world::World;
use tokio::runtime::Runtime;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub struct App {
    renderer: Renderer,
    runtime: Runtime,
    client: Client,
    game: Game,
    world: World,
}

impl App {
    pub fn new(runtime: Runtime, event_loop: &EventLoop<()>) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_min_inner_size(PhysicalSize::new(320, 180))
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(event_loop)?;

        let renderer = runtime.block_on(Renderer::new(window))?;

        let server_address = std::env::args().nth(1).unwrap();
        let player_name = std::env::args().nth(2).unwrap();

        let client = Client::connect(
            server_address,
            Credentials {
                name: player_name,
                password: "".to_string(),
            },
        );

        let game = Game::new();
        let world = World::new();

        Ok(Self {
            renderer,
            runtime,
            client,
            game,
            world,
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

    fn update(&mut self) {
        self.client.process_packets(&mut self.game, &mut self.world);
        self.renderer.set_view(View {
            position: self.world.player.position.extend(0.0),
            look_dir: self.world.player.look_dir.extend(0.0),
            aspect_ratio: 16.0 / 9.0,
            fov: 90.0,
        });

        for pos in &self.world.map.dirty_blocks() {
            let block = self.world.map.get(pos);
            if let Some(block) = block {
                self.renderer.add_block(&self.game, *pos, block);
            }
        }
    }

    fn repaint(&mut self) {
        self.renderer.render().unwrap();
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
            _ => (),
        }
    });
}
