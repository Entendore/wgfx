#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub format: wgpu::TextureFormat,
    pub size: wgpu::Extent3d,
}

impl Texture {
    #[must_use]
    pub fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Depth Sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            format,
            size,
        }
    }

    #[must_use]
    pub fn create_color_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Color Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        Self {
            texture,
            view,
            sampler,
            format,
            size,
        }
    }

    #[must_use]
    pub fn create_checkerboard(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let size = 64u32;
        let mut pixels = Vec::with_capacity((size * size * 4) as usize);
        for _y in 0..size {
            for x in 0..size {
                let is_light = (x / 8 + _y / 8) % 2 == 0;
                let c = if is_light { 255 } else { 100 };
                let r = if is_light { c } else { c };
                let g = if is_light { c } else { c };
                let b = if is_light { c } else { 140 };
                pixels.extend_from_slice(&[r, g, b, 255]);
            }
        }
        Self::from_pixels(
            device,
            queue,
            &pixels,
            size,
            size,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            "Checkerboard",
        )
    }

    #[must_use]
    pub fn create_solid_color(device: &wgpu::Device, queue: &wgpu::Queue, color: [u8; 4]) -> Self {
        let size = 1u32;
        let pixels = color;
        Self::from_pixels(
            device,
            queue,
            &pixels,
            size,
            size,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            "Solid Color",
        )
    }

    #[must_use]
    pub fn from_pixels(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pixels: &[u8],
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            format,
            size,
        }
    }

    /// Creates a diagonal gradient texture (useful for testing).
    #[must_use]
    pub fn create_gradient(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        c1: [u8; 4],
        c2: [u8; 4],
    ) -> Self {
        let size = 256u32;
        let mut pixels = Vec::with_capacity((size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                // Diagonal blend from c1 (top-left) to c2 (bottom-right)
                let t = (x as f32 + y as f32) / (size as f32 * 2.0);
                let r = (c1[0] as f32 * (1.0 - t) + c2[0] as f32 * t) as u8;
                let g = (c1[1] as f32 * (1.0 - t) + c2[1] as f32 * t) as u8;
                let b = (c1[2] as f32 * (1.0 - t) + c2[2] as f32 * t) as u8;
                let a = (c1[3] as f32 * (1.0 - t) + c2[3] as f32 * t) as u8;
                pixels.extend_from_slice(&[r, g, b, a]);
            }
        }
        Self::from_pixels(
            device,
            queue,
            &pixels,
            size,
            size,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            "Gradient",
        )
    }
}
