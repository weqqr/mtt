use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use mtt_core::math::Vector4;
use shaderc::{CompileOptions, Compiler, ShaderKind};
use std::borrow::Cow;
use std::time::Instant;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct RendererBase {
    window: Window,
    compiler: Compiler,
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface_format: TextureFormat,
}

// This is required to move shaderc across threads.
// TODO: Switch to Naga, use WGSL or load precompiled SPIR-V binaries
unsafe impl Send for RendererBase {}

impl RendererBase {
    pub async fn new(window: Window) -> Result<Self> {
        let compiler = Compiler::new().context("failed to initialize shader compiler")?;

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
            compiler,
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_format,
        })
    }

    pub fn create_shader(&mut self, source: &str, kind: ShaderKind) -> Result<ShaderModule> {
        let options = CompileOptions::new().unwrap();

        let artifact = self.compiler.compile_into_spirv(source, kind, "<???>", "main", Some(&options))?;

        Ok(unsafe {
            self.device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                label: None,
                source: Cow::Borrowed(artifact.as_binary()),
            })
        })
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
    pub fn new(base: &mut RendererBase) -> Self {
        let vertex_shader = base.create_shader(include_str!("../shaders/fullscreen.vert"), ShaderKind::Vertex).unwrap();
        let fragment_shader = base.create_shader(
            &std::fs::read_to_string("mtt/src/shaders/loading.frag").unwrap(),
            ShaderKind::Fragment,
        ).unwrap();

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
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct View {
    pub position: Vector4,
    pub look_dir: Vector4,
    pub aspect_ratio: f32,
    pub fov: f32,
}

impl Renderer {
    pub async fn new(window: Window) -> Result<Self> {
        let mut base = RendererBase::new(window).await?;

        let loading = LoadingScreen::new(&mut base);

        Ok(Renderer {
            base,
            loading,
        })
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
