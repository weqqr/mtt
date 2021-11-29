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
}

impl Renderer {
    pub fn new(window: Window) -> Self {
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
                    features: Features::default(),
                },
                None,
            )
            .block_on()
            .unwrap();

        let surface_format = surface.get_preferred_format(&adapter).unwrap();

        let renderer = Self {
            window,
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_format,
        };

        renderer.resize(renderer.window.inner_size());

        renderer
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
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
            encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                depth_stencil_attachment: None,
                color_attachments: &[
                    RenderPassColorAttachment {
                        view,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: true,
                        },
                        resolve_target: None,
                    }
                ]
            });
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
