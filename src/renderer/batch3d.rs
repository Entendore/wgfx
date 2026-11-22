use crate::core::{Mesh3D, Texture};
use crate::math::Mat4;
use wgpu::util::DeviceExt;

/// Maximum number of 3D instances per batch.
pub const MAX_3D_INSTANCES: usize = 50_000;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct Instance {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    light_dir: [f32; 3],
    time: f32,
}

#[derive(Debug, Clone)]
pub struct Shape3D {
    pub model: Mat4,
    pub color: crate::math::Vec4,
}

impl Shape3D {
    #[must_use]
    pub fn new(model: Mat4, color: crate::math::Vec4) -> Self {
        Self { model, color }
    }
}

pub struct Batch3D {
    count: u32,
    index_count: u32,
    instances: Vec<Instance>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl Batch3D {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        texture: &Texture,
        mesh: &Mesh3D,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
            label: Some("Batch3D Vertex Buffer"),
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
            label: Some("Batch3D Index Buffer"),
        });
        let index_count = mesh.indices.len() as u32;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: std::mem::size_of::<Instance>() as u64 * MAX_3D_INSTANCES as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Batch3D Instance Buffer"),
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Batch3D Uniform Buffer"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Batch3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/batch3d.wgsl").into()),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Batch3D Bind Group Layout"),
        });
        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
            label: Some("Batch3D Pipeline Layout"),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Batch3D Pipeline"),
            layout: Some(&pll),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 32,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None, // Disabled backface culling to ensure visibility regardless of winding order
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("Batch3D Bind Group"),
        });

        Self {
            count: 0,
            index_count,
            instances: Vec::with_capacity(MAX_3D_INSTANCES),
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

    pub fn push(&mut self, shape: Shape3D) {
        if (self.count as usize) >= MAX_3D_INSTANCES {
            log::warn!(
                "Batch3D: instance limit reached ({}), shape dropped",
                MAX_3D_INSTANCES
            );
            return;
        }
        self.instances.push(Instance {
            model: shape.model.to_cols_array_2d(),
            color: [shape.color.x, shape.color.y, shape.color.z, shape.color.w],
        });
        self.count += 1;
    }

    pub fn flush(
        &self,
        ctx: &mut crate::RenderContext,
        view_proj: Mat4,
        light_dir: crate::math::Vec3,
        time: f32,
    ) {
        if self.count == 0 {
            return;
        }
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            light_dir: [light_dir.x, light_dir.y, light_dir.z],
            time,
        };
        ctx.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
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
        ctx.pass.draw_indexed(0..self.index_count, 0, 0..self.count);
    }

    /// Returns the number of instances currently in the batch.
    pub fn count(&self) -> u32 {
        self.count
    }
}
