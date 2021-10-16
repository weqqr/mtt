use anyhow::{Context, Result};
use shaderc::{CompileOptions, Compiler, ShaderKind};
use std::borrow::Cow;
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct Renderer {
    window: Window,
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface_format: TextureFormat,
    pipeline_layout: PipelineLayout,
    render_pipeline: RenderPipeline,
}

fn compile_glsl(source: &str, kind: ShaderKind) -> Vec<u32> {
    let mut compiler = Compiler::new().unwrap();
    let options = CompileOptions::new().unwrap();

    compiler
        .compile_into_spirv(source, kind, "<???>", "main", Some(&options))
        .unwrap()
        .as_binary()
        .to_owned()
}

impl Renderer {
    pub async fn new(window: Window) -> Result<Self> {
        let instance = Instance::new(Backends::VULKAN);

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
            .await
            .context("no compatible adapter found")?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    limits: Limits::default(),
                    features: Features::empty(),
                },
                None,
            )
            .await?;

        let surface_format = surface
            .get_preferred_format(&adapter)
            .context("surface is incompatible with adapter")?;
        let size = window.inner_size();

        surface.configure(
            &device,
            &SurfaceConfiguration {
                width: size.width,
                height: size.height,
                format: surface_format,
                usage: TextureUsages::RENDER_ATTACHMENT,
                present_mode: PresentMode::Fifo,
            },
        );

        let vertex_shader = compile_glsl(include_str!("shaders/triangle.vert"), ShaderKind::Vertex);
        let fragment_shader = compile_glsl(include_str!("shaders/triangle.frag"), ShaderKind::Fragment);

        let vertex_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::SpirV(Cow::Owned(vertex_shader)),
        });

        let fragment_shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::SpirV(Cow::Owned(fragment_shader)),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[surface_format.into()],
            }),
        });

        Ok(Renderer {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_format,
            pipeline_layout,
            render_pipeline,
        })
    }

    pub fn render(&self) -> Result<()> {
        const SKY_COLOR: Color = Color {
            r: 0.3,
            g: 0.7,
            b: 0.9,
            a: 1.0,
        };

        let frame = self.surface.get_current_texture()?;

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let view = frame.texture.create_view(&TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: &view,
                    ops: Operations {
                        load: LoadOp::Clear(SKY_COLOR),
                        store: true,
                    },
                    resolve_target: None,
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                width: size.width,
                height: size.height,
                format: self.surface_format,
                usage: TextureUsages::RENDER_ATTACHMENT,
                present_mode: PresentMode::Fifo,
            },
        );
    }
}
