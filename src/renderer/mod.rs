pub mod batch2d;
pub mod batch3d;
pub mod line3d;
pub mod particles;
pub mod post_process;
pub mod sprite_batcher;
pub use batch2d::*;
pub use batch3d::*;
pub use line3d::*;
pub use particles::*;
pub use post_process::*;
pub use sprite_batcher::*;

use crate::core::{Mesh3D, Texture};

pub struct Renderer {
    pub batch2d: Batch2D,
    pub batch3d: Batch3D,
    pub sprite_batcher: SpriteBatcher,
    pub line3d: LineRenderer3D,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        texture: &Texture,
        mesh: &Mesh3D,
    ) -> Self {
        Self {
            batch2d: Batch2D::new(device, format, depth_format),
            batch3d: Batch3D::new(device, format, depth_format, texture, mesh),
            sprite_batcher: SpriteBatcher::new(device, format, depth_format, texture),
            line3d: LineRenderer3D::new(device, format, depth_format),
        }
    }
}
