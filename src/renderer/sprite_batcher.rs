use crate::core::Texture;
use crate::math::{Mat4, Vec2, Vec4};
use wgpu::util::DeviceExt;

/// Maximum number of sprites per batch.
pub const MAX_SPRITES: usize = 100_000;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct Instance {
    pos: [f32; 2],
    size: [f32; 2],
    uv_off: [f32; 2],
    uv_scale: [f32; 2],
    color: [f32; 4],
}

/// A sprite region within a texture atlas.
#[derive(Debug, Clone, Copy)]
pub struct SpriteRegion {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl SpriteRegion {
    /// Full texture region.
    pub fn full() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        }
    }

    /// Creates a region from pixel coordinates.
    pub fn from_pixels(px: f32, py: f32, pw: f32, ph: f32, tex_w: f32, tex_h: f32) -> Self {
        Self {
            x: px / tex_w,
            y: py / tex_h,
            w: pw / tex_w,
            h: ph / tex_h,
        }
    }

    pub fn to_uv_off(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    pub fn to_uv_scale(&self) -> [f32; 2] {
        [self.w, self.h]
    }
}

pub struct SpriteBatcher {
    count: u32,
    instances: Vec<Instance>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl SpriteBatcher {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        texture: &Texture,
    ) -> Self {
        let vertices: [Vertex; 4] = [
            Vertex {
                pos: [0.0, 0.0],
                uv: [0.0, 0.0],
            },
            Vertex {
                pos: [1.0, 0.0],
                uv: [1.0, 0.0],
            },
            Vertex {
                pos: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
            Vertex {
                pos: [0.0, 1.0],
                uv: [0.0, 1.0],
            },
        ];
        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
            label: Some("Sprite Vertex Buffer"),
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
            label: Some("Sprite Index Buffer"),
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: std::mem::size_of::<Instance>() as u64 * MAX_SPRITES as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Sprite Instance Buffer"),
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Sprite Uniform Buffer"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/sprite.wgsl").into()),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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
            label: Some("Sprite Bind Group Layout"),
        });
        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
            label: Some("Sprite Pipeline Layout"),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sprite Pipeline"),
            layout: Some(&pll),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 16,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![2 => Float32x2, 3 => Float32x2, 4 => Float32x2, 5 => Float32x2, 6 => Float32x4],
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
            label: Some("Sprite Bind Group"),
        });
        Self {
            count: 0,
            instances: Vec::with_capacity(MAX_SPRITES),
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

    /// Push a sprite with the full texture.
    pub fn push(&mut self, pos: Vec2, size: Vec2, color: Vec4) {
        self.push_with_region(pos, size, color, SpriteRegion::full(), 0.0);
    }

    /// Push a sprite with a specific UV region (for texture atlases).
    pub fn push_with_region(
        &mut self,
        pos: Vec2,
        size: Vec2,
        color: Vec4,
        region: SpriteRegion,
        _rot: f32,
    ) {
        if (self.count as usize) >= MAX_SPRITES {
            log::warn!(
                "SpriteBatcher: sprite limit reached ({}), sprite dropped",
                MAX_SPRITES
            );
            return;
        }
        self.instances.push(Instance {
            pos: [pos.x, pos.y],
            size: [size.x, size.y],
            uv_off: region.to_uv_off(),
            uv_scale: region.to_uv_scale(),
            color: [color.x, color.y, color.z, color.w],
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

    pub fn count(&self) -> u32 {
        self.count
    }
}
