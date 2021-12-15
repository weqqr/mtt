pub mod camera;
pub mod mesh;

use crate::mesh::{GpuMesh, Mesh, Vertex};
use anyhow::Result;
use nalgebra_glm::vec3;
use pollster::FutureExt;
use shaderc::{Compiler, ShaderKind};
use std::borrow::Cow;
use std::path::Path;
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct ShaderCompiler {
    compiler: Compiler,
}

impl ShaderCompiler {
    fn new() -> Self {
        let compiler = Compiler::new().unwrap();
        Self { compiler }
    }

    fn load_shader(
        &mut self,
        device: &Device,
        source_path: impl AsRef<Path>,
        kind: ShaderKind,
    ) -> Result<ShaderModule> {
        let source_path = source_path.as_ref();
        let path_str = source_path.as_os_str().to_string_lossy().into_owned();
        let source = std::fs::read_to_string(source_path)?;
        let artifact = self
            .compiler
            .compile_into_spirv(&source, kind, &path_str, "main", None)?;

        Ok(unsafe {
            device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                label: None,
                source: Cow::Borrowed(bytemuck::cast_slice(artifact.as_binary())),
            })
        })
    }
}

#[allow(dead_code)]
pub struct Renderer {
    compiler: ShaderCompiler,
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
        let mut compiler = ShaderCompiler::new();
        let instance = Instance::new(Backends::VULKAN);
        let surface = unsafe { instance.create_surface(&window) };
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

        let surface_format = surface.get_preferred_format(&adapter).unwrap();
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

        let vertex_shader = compiler.load_shader(&device, "data/shaders/world.vert", ShaderKind::Vertex)?;
        let fragment_shader = compiler.load_shader(&device, "data/shaders/world.frag", ShaderKind::Fragment)?;

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
                entry_point: "main",
                buffers: &[Vertex::layout()],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[surface_format.into()],
            }),
        });

        let renderer = Self {
            compiler,
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
                color_attachments: &[RenderPassColorAttachment {
                    view,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                    resolve_target: None,
                }],
            });
            rp.set_pipeline(&self.pipeline);
            self.gpu_mesh.draw(&mut rp);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
