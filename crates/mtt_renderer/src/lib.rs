pub mod mesh;

use std::borrow::Cow;

use crate::mesh::{GpuMesh, Mesh, Vertex};
use anyhow::Result;
use glam::vec3;
use pollster::FutureExt;
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::window::Window;


#[allow(dead_code)]
pub struct Renderer {
    window: Window,
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface_format: TextureFormat,
    gpu_mesh: GpuMesh,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
}

impl Renderer {
    pub fn new(window: Window) -> Result<Self> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        });
        let surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    limits: Limits::default(),
                    features: Features::default() | Features::SPIRV_SHADER_PASSTHROUGH,
                },
                None,
            )
            .block_on()
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities.formats[0];
        let mut mesh = Mesh::new();
        mesh.add_vertex(Vertex {
            position: vec3(0.0, 0.0, 0.0),
        });
        mesh.add_vertex(Vertex {
            position: vec3(1.0, 0.0, 0.0),
        });
        mesh.add_vertex(Vertex {
            position: vec3(0.0, 1.0, 0.0),
        });

        let gpu_mesh = GpuMesh::upload(&device, &mesh);

        let shader_source = include_str!("../../../data/shaders/world.wgsl");

        let vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
        });

        let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vertex_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout()],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &fragment_shader,
                entry_point: "fs_main",
                targets: &[Some(surface_format.into())],
            }),
            multiview: None,
        });

        let renderer = Self {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_format,
            gpu_mesh,
            pipeline_layout,
            pipeline,
        };

        renderer.resize(renderer.window.inner_size());

        Ok(renderer)
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                width: size.width,
                height: size.height,
                present_mode: PresentMode::Fifo,
                alpha_mode: CompositeAlphaMode::Opaque,
                view_formats: vec![],
            },
        );
    }

    pub fn render(&self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = &frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut rp = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                depth_stencil_attachment: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                    resolve_target: None,
                })],
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(&self.pipeline);
            self.gpu_mesh.draw(&mut rp);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
