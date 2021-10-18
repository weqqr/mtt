use bytemuck::Pod;
use crate::math::Vector4;
use anyhow::{Context, Result};
use log::error;
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
    fullscreen_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    color_buffer: Buffer,
    uniform_buffer: Buffer,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
pub struct ViewUniforms {
    position: Vector4,
    look_dir: Vector4,
}

unsafe impl bytemuck::Zeroable for ViewUniforms {}

fn compile_glsl(source: &str, kind: ShaderKind) -> Vec<u32> {
    let mut compiler = Compiler::new().unwrap();
    let options = CompileOptions::new().unwrap();

    let artifact = compiler.compile_into_spirv(source, kind, "<???>", "main", Some(&options));
    match artifact {
        Ok(artifact) => artifact.as_binary().to_owned(),
        Err(err) => {
            error!("{}", err);
            panic!();
        }
    }
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

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                },
                BindGroupLayoutEntry {
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&bind_group_layout],
        });

        let fullscreen_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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

        let color_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 4,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: true,
        });

        {
            let mut mapped_range = color_buffer.slice(..).get_mapped_range_mut();
            let grey = bytemuck::bytes_of(&0.0f32);
            mapped_range[0..4].copy_from_slice(grey);
        }

        color_buffer.unmap();

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<ViewUniforms>() as BufferAddress,
            usage: BufferUsages::UNIFORM,
            mapped_at_creation: true,
        });

        {
            let mut mapped_range = uniform_buffer.slice(..).get_mapped_range_mut();
            let uniform = ViewUniforms {
                position: Vector4::new(0.1, 0.2, 0.3, 1.0),
                look_dir: Vector4::new(1.0, 0.0, 0.0, 0.0),
            };
            let grey = bytemuck::bytes_of(&uniform);
            mapped_range[..std::mem::size_of::<ViewUniforms>()].copy_from_slice(grey);
        }

        uniform_buffer.unmap();

        Ok(Renderer {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_format,
            pipeline_layout,
            fullscreen_pipeline,
            bind_group_layout,
            color_buffer,
            uniform_buffer,
        })
    }

    pub fn render(&self) -> Result<()> {
        let frame = self.surface.get_current_texture()?;

        const SKY_COLOR: Color = Color {
            r: 0.3,
            g: 0.7,
            b: 0.9,
            a: 1.0,
        };

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.color_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let command_list = {
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
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_pipeline(&self.fullscreen_pipeline);
                render_pass.draw(0..3, 0..1);
            }

            encoder.finish()
        };

        self.queue.submit(Some(command_list));
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
