use nalgebra_glm::Vec3;
use wgpu::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub struct Vertex {
    pub position: Vec3,
}

impl Vertex {
    pub fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: 3 * 4,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
            step_mode: VertexStepMode::Vertex,
        }
    }
}

pub struct Mesh {
    data: Vec<f32>,
    vertex_count: u32,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            vertex_count: 0,
        }
    }

    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.data.push(vertex.position.x);
        self.data.push(vertex.position.y);
        self.data.push(vertex.position.z);

        self.vertex_count += 1;
    }

    pub fn data(&self) -> &[f32] {
        self.data.as_slice()
    }

    pub fn upload_to_gpu(&self, device: &Device, queue: &Queue) {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 0,
            usage: BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&self.data));
    }
}

pub struct GpuMesh {
    vertex_buffer: Buffer,
    vertex_count: u32,
}

impl GpuMesh {
    pub(crate) fn upload(device: &Device, mesh: &Mesh) -> Self {
        let vertex_count = mesh.vertex_count;
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&mesh.data),
        });

        Self {
            vertex_count,
            vertex_buffer,
        }
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn draw<'a>(&'a self, rp: &mut RenderPass<'a>) {
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.draw(0..self.vertex_count, 0..1);
    }
}
