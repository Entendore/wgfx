use crate::math::{Mat4, Vec2};

/// GPU particle with lifetime support.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct GpuParticle {
    pos: [f32; 2],
    vel: [f32; 2],
    color: [f32; 4],
    age: f32,
    lifetime: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct Uniforms {
    mvp: [[f32; 4]; 4],
    dt: f32,
    count: u32,
    gravity: [f32; 2],
    _pad1: [f32; 4],
}

pub struct ParticleSystem {
    count: usize,
    capacity: usize,
    write_cursor: usize,
    particle_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
}

impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        capacity: usize,
    ) -> Self {
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: (capacity * std::mem::size_of::<GpuParticle>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Particle Buffer"),
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("Particle Uniform Buffer"),
        });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../shaders/particles_compute.wgsl").into(),
            ),
        });
        let cbgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Particle Compute Bind Group Layout"),
        });
        let cpll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&cbgl],
            push_constant_ranges: &[],
            label: Some("Particle Compute Pipeline Layout"),
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Compute Pipeline"),
            layout: Some(&cpll),
            module: &compute_shader,
            entry_point: "cs_main",
            compilation_options: Default::default(),
            cache: None,
        });
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &cbgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("Particle Compute Bind Group"),
        });

        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Render Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../shaders/particles_render.wgsl").into(),
            ),
        });
        let rbgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Particle Render Bind Group Layout"),
        });
        let rpll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&rbgl],
            push_constant_ranges: &[],
            label: Some("Particle Render Pipeline Layout"),
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(&rpll),
            cache: None,
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                ..Default::default()
            },
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
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &rbgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("Particle Render Bind Group"),
        });

        Self {
            count: 0,
            capacity,
            write_cursor: 0,
            particle_buffer,
            uniform_buffer,
            compute_pipeline,
            compute_bind_group,
            render_pipeline,
            render_bind_group,
        }
    }

    /// Emit particles at a position with random velocities. Uses a ring buffer
    /// to overwrite the oldest particles when capacity is reached.
    pub fn emit(&mut self, queue: &wgpu::Queue, pos: Vec2, count: usize) {
        let count = count.min(self.capacity);
        let mut new_particles = Vec::with_capacity(count);
        for _ in 0..count {
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            let speed = 50.0 + rand::random::<f32>() * 200.0;
            let lifetime = 1.0 + rand::random::<f32>() * 2.0;
            new_particles.push(GpuParticle {
                pos: [pos.x, pos.y],
                vel: [angle.cos() * speed, angle.sin() * speed],
                color: [1.0, 0.5 + rand::random::<f32>() * 0.5, 0.2, 1.0],
                age: 0.0,
                lifetime,
            });
        }

        // Ring buffer: wrap around when we exceed capacity
        let write_end = self.write_cursor + count;
        if write_end <= self.capacity {
            let offset = (self.write_cursor * std::mem::size_of::<GpuParticle>()) as u64;
            queue.write_buffer(
                &self.particle_buffer,
                offset,
                bytemuck::cast_slice(&new_particles),
            );
        } else {
            let first_chunk = self.capacity - self.write_cursor;
            let _second_chunk = count - first_chunk;

            let offset1 = (self.write_cursor * std::mem::size_of::<GpuParticle>()) as u64;
            queue.write_buffer(
                &self.particle_buffer,
                offset1,
                bytemuck::cast_slice(&new_particles[..first_chunk]),
            );

            let offset2 = 0;
            queue.write_buffer(
                &self.particle_buffer,
                offset2,
                bytemuck::cast_slice(&new_particles[first_chunk..]),
            );
        }

        self.write_cursor = write_end % self.capacity;
        self.count = (self.count + count).min(self.capacity);
    }

    /// Emit particles with custom parameters.
    pub fn emit_burst(
        &mut self,
        queue: &wgpu::Queue,
        pos: Vec2,
        count: usize,
        speed_range: (f32, f32),
        lifetime_range: (f32, f32),
        color: crate::math::Vec4,
    ) {
        let count = count.min(self.capacity);
        let mut new_particles = Vec::with_capacity(count);
        for _ in 0..count {
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            let speed = speed_range.0 + rand::random::<f32>() * (speed_range.1 - speed_range.0);
            let lifetime =
                lifetime_range.0 + rand::random::<f32>() * (lifetime_range.1 - lifetime_range.0);
            new_particles.push(GpuParticle {
                pos: [pos.x, pos.y],
                vel: [angle.cos() * speed, angle.sin() * speed],
                color: [color.x, color.y, color.z, color.w],
                age: 0.0,
                lifetime,
            });
        }

        let write_end = self.write_cursor + count;
        if write_end <= self.capacity {
            let offset = (self.write_cursor * std::mem::size_of::<GpuParticle>()) as u64;
            queue.write_buffer(
                &self.particle_buffer,
                offset,
                bytemuck::cast_slice(&new_particles),
            );
        } else {
            let first_chunk = self.capacity - self.write_cursor;
            let offset1 = (self.write_cursor * std::mem::size_of::<GpuParticle>()) as u64;
            queue.write_buffer(
                &self.particle_buffer,
                offset1,
                bytemuck::cast_slice(&new_particles[..first_chunk]),
            );

            let offset2 = 0;
            queue.write_buffer(
                &self.particle_buffer,
                offset2,
                bytemuck::cast_slice(&new_particles[first_chunk..]),
            );
        }
        self.write_cursor = write_end % self.capacity;
        self.count = (self.count + count).min(self.capacity);
    }

    /// Update particles using a GPU compute pass. Records into a new command encoder
    /// and submits it immediately to avoid borrow conflicts with the active render pass.
    pub fn update(&self, device: &wgpu::Device, queue: &wgpu::Queue, dt: f32, mvp: Mat4) {
        if self.count == 0 {
            return;
        }

        let uniforms = Uniforms {
            mvp: mvp.to_cols_array_2d(),
            dt,
            count: self.count as u32,
            gravity: [0.0, 400.0],
            _pad1: [0.0; 4],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Particle Compute Encoder"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Compute Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups((self.count as u32 + 63) / 64, 1, 1);
        }
        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn flush(&self, ctx: &mut crate::RenderContext) {
        if self.count == 0 {
            return;
        }
        ctx.pass.set_pipeline(&self.render_pipeline);
        ctx.pass.set_bind_group(0, &self.render_bind_group, &[]);
        ctx.pass.draw(0..1, 0..self.count as u32);
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
