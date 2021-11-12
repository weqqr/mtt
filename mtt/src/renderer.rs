use anyhow::{Context, Result};
use bytemuck::Pod;
use log::error;
use mtt_core::game::node::DrawType;
use mtt_core::game::Game;
use mtt_core::math::{Vector3i16, Vector4};
use mtt_core::world::Block;
use shaderc::{CompileOptions, Compiler, ShaderKind};
use std::borrow::Cow;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
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
    depth_buffer: TextureView,
    pipeline_layout: PipelineLayout,
    fullscreen_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    view_buffer: Buffer,
    blocks: Vec<Buffer>,
}

#[derive(Clone, Copy, Pod)]
#[repr(C)]
pub struct View {
    pub position: Vector4,
    pub look_dir: Vector4,
    pub aspect_ratio: f32,
    pub fov: f32,
}

unsafe impl bytemuck::Zeroable for View {}

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
                    features: Features::SPIRV_SHADER_PASSTHROUGH,
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

        let vertex_shader = compile_glsl(include_str!("shaders/fullscreen.vert"), ShaderKind::Vertex);
        let fragment_shader = compile_glsl(include_str!("shaders/block.frag"), ShaderKind::Fragment);

        let vertex_shader = unsafe {
            device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                label: None,
                source: Cow::Owned(vertex_shader),
            })
        };

        let fragment_shader = unsafe {
            device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                label: None,
                source: Cow::Owned(fragment_shader),
            })
        };

        let depth_buffer = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: 1280,
                height: 720,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth24Plus,
            usage: TextureUsages::RENDER_ATTACHMENT,
        });

        let depth_buffer = depth_buffer.create_view(&TextureViewDescriptor::default());

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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[surface_format.into()],
            }),
        });

        let view = View {
            position: Vector4::new(0.1, 0.2, 0.3, 1.0),
            look_dir: Vector4::new(1.0, 0.0, 0.0, 0.0),
            aspect_ratio: 1.0,
            fov: 90.0,
        };

        let view_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<View>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::MAP_WRITE,
            mapped_at_creation: true,
        });

        {
            let mut mapped_range = view_buffer.slice(..).get_mapped_range_mut();

            let grey = bytemuck::bytes_of(&view);
            mapped_range.copy_from_slice(grey);
        }

        view_buffer.unmap();

        Ok(Renderer {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_format,
            depth_buffer,
            pipeline_layout,
            fullscreen_pipeline,
            bind_group_layout,
            view_buffer,
            blocks: Vec::new(),
        })
    }

    pub fn add_block(&mut self, game: &Game, position: Vector3i16, block: &Block) {
        let mut data = Vec::new();

        data.push(position.x as i32 as u32);
        data.push(position.y as i32 as u32);
        data.push(position.z as i32 as u32);

        for z in 0..Block::SIZE {
            for y in 0..Block::SIZE {
                for x in 0..Block::SIZE {
                    let node = block.get(x, y, z);
                    let is_normal = game
                        .nodes
                        .get(node.id as usize)
                        .map(|node| matches!(node.draw_type, DrawType::Normal))
                        .unwrap_or(false);

                    if is_normal {
                        data.push(0xEEEEEEEE);
                    } else {
                        data.push(0x00000000u32);
                    }
                }
            }
        }

        self.blocks.push(self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(data.as_slice()),
            usage: BufferUsages::STORAGE,
        }));
    }

    pub fn set_view(&mut self, view: View) {
        let _ = self.view_buffer.slice(..).map_async(MapMode::Write);

        self.device.poll(Maintain::Wait);

        let bytes = bytemuck::bytes_of(&view);
        self.view_buffer.slice(..).get_mapped_range_mut().copy_from_slice(bytes);
        self.view_buffer.unmap();
    }

    pub fn render(&self) -> Result<()> {
        let frame = self.surface.get_current_texture()?;

        const SKY_COLOR: Color = Color {
            r: 0.3,
            g: 0.7,
            b: 0.9,
            a: 1.0,
        };

        let command_list = {
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor { label: None });

            let bind_groups = self
                .blocks
                .iter()
                .map(|block| {
                    self.device.create_bind_group(&BindGroupDescriptor {
                        label: None,
                        layout: &self.bind_group_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: block.as_entire_binding(),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: self.view_buffer.as_entire_binding(),
                            },
                        ],
                    })
                })
                .collect::<Vec<_>>();

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
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: &self.depth_buffer,
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });
                render_pass.set_pipeline(&self.fullscreen_pipeline);

                for bind_group in &bind_groups {
                    render_pass.set_bind_group(0, bind_group, &[]);
                    render_pass.draw(0..3, 0..1);
                }
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
