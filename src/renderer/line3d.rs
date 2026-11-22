use crate::core::Mesh3D;
use crate::math::{Mat4, Vec3, Vec4};

/// Maximum number of line vertices before the buffer needs to be recreated.
const INITIAL_LINE_BUFFER_SIZE: u64 = 1024 * 1024; // 1 MB

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct LineVertex {
    pos: [f32; 3],
    color: [f32; 4],
}

pub struct LineRenderer3D {
    vertices: Vec<LineVertex>,
    vertex_buffer: wgpu::Buffer,
    buffer_capacity: u64,
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl LineRenderer3D {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: INITIAL_LINE_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Line3D Vertex Buffer"),
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Line3D Uniform Buffer"),
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Line3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/line3d.wgsl").into()),
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
            label: Some("Line3D Bind Group Layout"),
        });
        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
            label: Some("Line3D Pipeline Layout"),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line3D Pipeline"),
            layout: Some(&pll),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<LineVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4],
                }],
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
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
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
            label: Some("Line3D Bind Group"),
        });
        Self {
            vertices: Vec::new(),
            vertex_buffer,
            buffer_capacity: INITIAL_LINE_BUFFER_SIZE,
            uniform_buffer,
            pipeline,
            bind_group,
        }
    }

    pub fn push_line(&mut self, p1: Vec3, p2: Vec3, color: Vec4) {
        self.vertices.push(LineVertex {
            pos: [p1.x, p1.y, p1.z],
            color: [color.x, color.y, color.z, color.w],
        });
        self.vertices.push(LineVertex {
            pos: [p2.x, p2.y, p2.z],
            color: [color.x, color.y, color.z, color.w],
        });
    }

    pub fn push_grid(&mut self, half_size: i32, spacing: f32, color: Vec4) {
        for i in -half_size..=half_size {
            let p = i as f32 * spacing;
            self.push_line(
                Vec3::new(-half_size as f32 * spacing, 0.0, p),
                Vec3::new(half_size as f32 * spacing, 0.0, p),
                color,
            );
            self.push_line(
                Vec3::new(p, 0.0, -half_size as f32 * spacing),
                Vec3::new(p, 0.0, half_size as f32 * spacing),
                color,
            );
        }
    }

    /// Draws a wireframe box (AABB).
    pub fn push_aabb(&mut self, min: Vec3, max: Vec3, color: Vec4) {
        let c = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
        // Bottom face
        self.push_line(c(min.x, min.y, min.z), c(max.x, min.y, min.z), color);
        self.push_line(c(max.x, min.y, min.z), c(max.x, min.y, max.z), color);
        self.push_line(c(max.x, min.y, max.z), c(min.x, min.y, max.z), color);
        self.push_line(c(min.x, min.y, max.z), c(min.x, min.y, min.z), color);
        // Top face
        self.push_line(c(min.x, max.y, min.z), c(max.x, max.y, min.z), color);
        self.push_line(c(max.x, max.y, min.z), c(max.x, max.y, max.z), color);
        self.push_line(c(max.x, max.y, max.z), c(min.x, max.y, max.z), color);
        self.push_line(c(min.x, max.y, max.z), c(min.x, max.y, min.z), color);
        // Vertical edges
        self.push_line(c(min.x, min.y, min.z), c(min.x, max.y, min.z), color);
        self.push_line(c(max.x, min.y, min.z), c(max.x, max.y, min.z), color);
        self.push_line(c(max.x, min.y, max.z), c(max.x, max.y, max.z), color);
        self.push_line(c(min.x, min.y, max.z), c(min.x, max.y, max.z), color);
    }

    pub fn push_wireframe(&mut self, mesh: &Mesh3D, model: Mat4, color: Vec4) {
        let mut drawn = std::collections::HashSet::new();
        for i in (0..mesh.indices.len()).step_by(3) {
            let i0 = mesh.indices[i] as usize;
            let i1 = mesh.indices[i + 1] as usize;
            let i2 = mesh.indices[i + 2] as usize;
            let p0 = model.transform_point3(Vec3::from(mesh.vertices[i0].pos));
            let p1 = model.transform_point3(Vec3::from(mesh.vertices[i1].pos));
            let p2 = model.transform_point3(Vec3::from(mesh.vertices[i2].pos));
            let key = |a: usize, b: usize| if a < b { (a, b) } else { (b, a) };
            if drawn.insert(key(i0, i1)) {
                self.push_line(p0, p1, color);
            }
            if drawn.insert(key(i1, i2)) {
                self.push_line(p1, p2, color);
            }
            if drawn.insert(key(i2, i0)) {
                self.push_line(p2, p0, color);
            }
        }
    }

    /// Draws a coordinate axis gizmo at the given position.
    pub fn push_axes(&mut self, origin: Vec3, size: f32) {
        self.push_line(
            origin,
            origin + Vec3::new(size, 0.0, 0.0),
            Vec4::new(1.0, 0.0, 0.0, 1.0),
        );
        self.push_line(
            origin,
            origin + Vec3::new(0.0, size, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 1.0),
        );
        self.push_line(
            origin,
            origin + Vec3::new(0.0, 0.0, size),
            Vec4::new(0.0, 0.0, 1.0, 1.0),
        );
    }

    pub fn flush(&mut self, ctx: &mut crate::RenderContext, mvp: Mat4) {
        if self.vertices.is_empty() {
            return;
        }

        // Check if we need to recreate the buffer (dynamic resizing)
        let required_size = (self.vertices.len() * std::mem::size_of::<LineVertex>()) as u64;
        if required_size > self.buffer_capacity {
            let new_capacity = required_size
                .next_power_of_two()
                .max(INITIAL_LINE_BUFFER_SIZE);
            log::info!(
                "Line3D: Recreating vertex buffer ({} -> {} bytes)",
                self.buffer_capacity,
                new_capacity
            );
            self.vertex_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
                size: new_capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
                label: Some("Line3D Vertex Buffer (Resized)"),
            });
            self.buffer_capacity = new_capacity;
        }

        ctx.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[mvp.to_cols_array_2d()]),
        );
        ctx.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        ctx.pass.set_pipeline(&self.pipeline);
        ctx.pass.set_bind_group(0, &self.bind_group, &[]);
        ctx.pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        ctx.pass.draw(0..self.vertices.len() as u32, 0..1);
        self.vertices.clear();
    }

    /// Returns the number of vertices (line endpoints) currently queued.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}
