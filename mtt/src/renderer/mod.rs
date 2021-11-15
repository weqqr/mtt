use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use log::error;
use mtt_core::game::node::DrawType;
use mtt_core::game::Game;
use mtt_core::math::{Vector3i16, Vector4};
use mtt_core::world::Block;
use shaderc::{CompileOptions, Compiler, ShaderKind};
use std::borrow::Cow;
use std::time::Instant;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct RendererBase {
    window: Window,
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface_format: TextureFormat,
}

impl RendererBase {
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

        Ok(Self {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_format,
        })
    }

    pub fn create_shader(&self, source: &[u32]) -> ShaderModule {
        // SAFETY: is just an illusion
        unsafe {
            self.device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                label: None,
                source: Cow::Borrowed(source),
            })
        }
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

    pub fn update_entire_buffer<T: Pod>(&self, buffer: &Buffer, value: &T) {
        let _ = buffer.slice(..).map_async(MapMode::Write);

        self.device.poll(Maintain::Wait);

        let bytes = bytemuck::bytes_of(value);
        buffer.slice(..).get_mapped_range_mut().copy_from_slice(bytes);
        buffer.unmap();
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct LoadingUniforms {
    time: f32,
    aspect_ratio: f32,
}

pub struct LoadingScreen {
    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
    uniforms: Buffer,
    t0: Instant,
}

impl LoadingScreen {
    pub fn new(base: &RendererBase) -> Self {
        let vertex_shader = compile_glsl(include_str!("../shaders/fullscreen.vert"), ShaderKind::Vertex);
        let fragment_shader = compile_glsl(
            &std::fs::read_to_string("mtt/src/shaders/loading.frag").unwrap(),
            ShaderKind::Fragment,
        );

        let vertex_shader = base.create_shader(&vertex_shader);
        let fragment_shader = base.create_shader(&fragment_shader);

        let bind_group_layout = base.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
            }],
        });

        let pipeline_layout = base.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&bind_group_layout],
        });

        let pipeline = base.device.create_render_pipeline(&RenderPipelineDescriptor {
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
                targets: &[base.surface_format.into()],
            }),
        });

        let uniforms = LoadingUniforms {
            time: 0.0,
            aspect_ratio: 16.0 / 9.0,
        };

        let uniforms = base.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::MAP_WRITE,
            contents: bytemuck::bytes_of(&uniforms),
        });

        LoadingScreen {
            bind_group_layout,
            pipeline_layout,
            pipeline,
            uniforms,
            t0: Instant::now(),
        }
    }

    pub fn render(&self, base: &RendererBase, view: &TextureView, encoder: &mut CommandEncoder) {
        base.update_entire_buffer(
            &self.uniforms,
            &LoadingUniforms {
                time: (Instant::now() - self.t0).as_secs_f32(),
                aspect_ratio: 16.0 / 9.0,
            },
        );

        let bind_group = base.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: self.uniforms.as_entire_binding(),
            }],
        });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[RenderPassColorAttachment {
                view,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLUE),
                    store: true,
                },
                resolve_target: None,
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

pub struct Renderer {
    base: RendererBase,

    loading: LoadingScreen,

    view_buffer: Buffer,
    blocks: Vec<Buffer>,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct View {
    pub position: Vector4,
    pub look_dir: Vector4,
    pub aspect_ratio: f32,
    pub fov: f32,
}

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
        let base = RendererBase::new(window).await?;

        let loading = LoadingScreen::new(&base);

        let view = View {
            position: Vector4::new(0.1, 0.2, 0.3, 1.0),
            look_dir: Vector4::new(1.0, 0.0, 0.0, 0.0),
            aspect_ratio: 1.0,
            fov: 90.0,
        };

        let view_buffer = base.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&view),
            usage: BufferUsages::UNIFORM | BufferUsages::MAP_WRITE,
        });

        Ok(Renderer {
            base,
            loading,
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

        self.blocks
            .push(self.base.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(data.as_slice()),
                usage: BufferUsages::STORAGE,
            }));
    }

    pub fn set_view(&mut self, view: View) {
        self.base.update_entire_buffer(&self.view_buffer, &view);
    }

    fn record_command_buffer(&self, view: &TextureView) -> CommandBuffer {
        let mut encoder = self
            .base
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            self.loading.render(&self.base, view, &mut encoder);
        }

        encoder.finish()
    }

    pub fn render(&self) -> Result<()> {
        let frame = self.base.surface.get_current_texture()?;
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let command_list = self.record_command_buffer(&view);

        self.base.queue.submit(Some(command_list));
        frame.present();

        Ok(())
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.base.resize(size);
    }
}
