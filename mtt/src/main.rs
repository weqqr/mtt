#![allow(dead_code)]

mod client;
mod media;
mod net;
mod renderer;

use crate::media::MediaStorage;
use crate::renderer::Renderer;
use anyhow::Result;
use tokio::runtime::Runtime;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

pub struct SharedResources {
    media: MediaStorage,
    renderer: Renderer,
    runtime: Runtime,
}

pub struct MainMenu {}

impl MainMenu {
    pub fn new() -> Self {
        Self {}
    }

    fn handle_event(&mut self, resources: &mut SharedResources, event: Event<()>) -> Option<ControlFlow> {
        match event {
            Event::MainEventsCleared => {
                resources.renderer.render().unwrap();
            }
            _ => (),
        }

        None
    }
}

pub enum AppState {
    MainMenu(MainMenu),
}

impl AppState {
    pub fn handle_event(&mut self, resources: &mut SharedResources, event: Event<()>) -> Option<ControlFlow> {
        if let Event::WindowEvent { event, .. } = &event {
            match event {
                WindowEvent::CloseRequested => return Some(ControlFlow::Exit),
                WindowEvent::Resized(size) => {
                    resources.renderer.resize(size.clone());
                }
                _ => (),
            }
        }

        match self {
            AppState::MainMenu(main_menu) => main_menu.handle_event(resources, event),
        }
    }
}

pub struct App {
    resources: SharedResources,
    state: AppState,
}

impl App {
    pub fn new(runtime: Runtime, event_loop: &EventLoop<()>) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_min_inner_size(PhysicalSize::new(320, 180))
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(event_loop)?;

        let media = MediaStorage::new()?;
        let renderer = runtime.block_on(Renderer::new(window))?;

        let resources = SharedResources {
            media,
            renderer,
            runtime,
        };

        Ok(Self {
            resources,
            state: AppState::MainMenu(MainMenu::new()),
        })
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

        if let Some(cf) = app.state.handle_event(&mut app.resources, event) {
            *control_flow = cf;
        }
    });
}
