use crate::math::Mat4;
use wgpu::util::DeviceExt;

/// Maximum number of 2D shapes per batch.
pub const MAX_2D_INSTANCES: usize = 100_000;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct Vertex {
    pos: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct Instance {
    pos: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    kind: u32,
    rot: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShapeKind {
    Rect,
    Circle,
    RoundedRect,
}

impl ShapeKind {
    pub fn as_u32(self) -> u32 {
        match self {
            ShapeKind::Rect => 0,
            ShapeKind::Circle => 1,
            ShapeKind::RoundedRect => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Shape2D {
    pub pos: crate::math::Vec2,
    pub size: crate::math::Vec2,
    pub color: crate::math::Vec4,
    pub kind: ShapeKind,
    pub rot: f32,
}

impl Shape2D {
    #[must_use]
    pub fn rect(pos: crate::math::Vec2, size: crate::math::Vec2, color: crate::math::Vec4) -> Self {
        Self {
            pos,
            size,
            color,
            kind: ShapeKind::Rect,
            rot: 0.0,
        }
    }

    #[must_use]
    pub fn circle(pos: crate::math::Vec2, radius: f32, color: crate::math::Vec4) -> Self {
        Self {
            pos,
            size: crate::math::Vec2::new(radius * 2.0, radius * 2.0),
            color,
            kind: ShapeKind::Circle,
            rot: 0.0,
        }
    }

    #[must_use]
    pub fn with_rot(mut self, rot: f32) -> Self {
        self.rot = rot;
        self
    }
}

pub struct Batch2D {
    count: u32,
    instances: Vec<Instance>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl Batch2D {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let vertices: [Vertex; 4] = [
            Vertex { pos: [0.0, 0.0] },
            Vertex { pos: [1.0, 0.0] },
            Vertex { pos: [1.0, 1.0] },
            Vertex { pos: [0.0, 1.0] },
        ];
        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
            label: Some("Batch2D Vertex Buffer"),
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
            label: Some("Batch2D Index Buffer"),
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: std::mem::size_of::<Instance>() as u64 * MAX_2D_INSTANCES as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Batch2D Instance Buffer"),
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Batch2D Uniform Buffer"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Batch2D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/batch2d.wgsl").into()),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Batch2D Bind Group Layout"),
        });
        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
            label: Some("Batch2D Pipeline Layout"),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Batch2D Pipeline"),
            layout: Some(&pll),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 8,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32x2, 3 => Float32x4, 4 => Uint32, 5 => Float32],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Batch2D Bind Group"),
        });
        Self {
            count: 0,
            instances: Vec::with_capacity(MAX_2D_INSTANCES),
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            pipeline,
            bind_group,
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.instances.clear();
    }

    pub fn push(&mut self, shape: Shape2D) {
        if (self.count as usize) >= MAX_2D_INSTANCES {
            log::warn!(
                "Batch2D: instance limit reached ({}), shape dropped",
                MAX_2D_INSTANCES
            );
            return;
        }
        self.instances.push(Instance {
            pos: [shape.pos.x, shape.pos.y],
            size: [shape.size.x, shape.size.y],
            color: [shape.color.x, shape.color.y, shape.color.z, shape.color.w],
            kind: shape.kind.as_u32(),
            rot: shape.rot,
        });
        self.count += 1;
    }

    pub fn flush(&self, ctx: &mut crate::RenderContext, mvp: Mat4) {
        if self.count == 0 {
            return;
        }
        ctx.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[mvp.to_cols_array_2d()]),
        );
        ctx.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );
        ctx.pass.set_pipeline(&self.pipeline);
        ctx.pass.set_bind_group(0, &self.bind_group, &[]);
        ctx.pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        ctx.pass
            .set_vertex_buffer(1, self.instance_buffer.slice(..));
        ctx.pass
            .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        ctx.pass.draw_indexed(0..6, 0, 0..self.count);
    }

    /// Returns the number of shapes currently in the batch.
    pub fn count(&self) -> u32 {
        self.count
    }
}
