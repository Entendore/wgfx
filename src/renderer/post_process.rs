use crate::core::Texture;

/// Post-processing effect configuration.
#[derive(Debug, Clone, Copy)]
pub struct PostProcessConfig {
    pub grayscale: bool,
    pub vignette_intensity: f32,
    pub vignette_radius: f32,
    pub chromatic_aberration: f32,
    pub brightness: f32,
    pub contrast: f32,
}

impl Default for PostProcessConfig {
    fn default() -> Self {
        Self {
            grayscale: false,
            vignette_intensity: 0.5,
            vignette_radius: 0.8,
            chromatic_aberration: 0.0,
            brightness: 1.0,
            contrast: 1.0,
        }
    }
}

pub struct PostProcessRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    config: PostProcessConfig,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct PostProcessUniforms {
    grayscale: u32,
    vignette_intensity: f32,
    vignette_radius: f32,
    chromatic_aberration: f32,
    brightness: f32,
    contrast: f32,
    _pad: [f32; 2],
}

impl PostProcessRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, src_texture: &Texture) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: std::mem::size_of::<PostProcessUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label: Some("PostProcess Uniform Buffer"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PostProcess Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../shaders/post_process.wgsl").into(),
            ),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("PostProcess Bind Group Layout"),
        });
        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
            label: Some("PostProcess Pipeline Layout"),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PostProcess Pipeline"),
            layout: Some(&pll),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&src_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("PostProcess Bind Group"),
        });
        Self {
            pipeline,
            bind_group,
            uniform_buffer,
            config: PostProcessConfig::default(),
        }
    }

    pub fn config(&self) -> &PostProcessConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: PostProcessConfig) {
        self.config = config;
    }

    pub fn flush(&self, ctx: &mut crate::RenderContext) {
        let uniforms = PostProcessUniforms {
            grayscale: if self.config.grayscale { 1 } else { 0 },
            vignette_intensity: self.config.vignette_intensity,
            vignette_radius: self.config.vignette_radius,
            chromatic_aberration: self.config.chromatic_aberration,
            brightness: self.config.brightness,
            contrast: self.config.contrast,
            _pad: [0.0, 0.0],
        };
        ctx.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        ctx.pass.set_pipeline(&self.pipeline);
        ctx.pass.set_bind_group(0, &self.bind_group, &[]);
        ctx.pass.draw(0..3, 0..1);
    }
}
