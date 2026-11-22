#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn new(pos: [f32; 3], uv: [f32; 2], normal: [f32; 3]) -> Self {
        Self { pos, uv, normal }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh3D {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh3D {
    #[must_use]
    pub fn cube() -> Self {
        let positions: [[f32; 3]; 24] = [
            // Back face (z = -0.5)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
            // Front face (z = +0.5)
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
            // Top face (y = +0.5)
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
            // Bottom face (y = -0.5)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // Right face (x = +0.5)
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // Left face (x = -0.5)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, -0.5, 0.5],
        ];

        let normals: [[f32; 3]; 6] = [
            [0.0, 0.0, -1.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
        ];

        let vertices: Vec<Vertex> = positions
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let face = i / 4;
                let corner = i % 4;
                let uv = match corner {
                    0 => [0.0, 0.0],
                    1 => [1.0, 0.0],
                    2 => [1.0, 1.0],
                    _ => [0.0, 1.0],
                };
                Vertex::new(*p, uv, normals[face])
            })
            .collect();

        let indices = vec![
            0, 1, 2, 2, 3, 0, // back
            4, 5, 6, 6, 7, 4, // front
            8, 9, 10, 10, 11, 8, // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];

        Self { vertices, indices }
    }

    #[must_use]
    pub fn plane(size: f32, resolution: usize) -> Self {
        let half = size / 2.0;
        let step = size / resolution as f32;

        let vertices: Vec<Vertex> = (0..=resolution)
            .flat_map(|z| {
                (0..=resolution).map(move |x| {
                    let px = -half + x as f32 * step;
                    let pz = -half + z as f32 * step;
                    Vertex::new(
                        [px, 0.0, pz],
                        [x as f32 / resolution as f32, z as f32 / resolution as f32],
                        [0.0, 1.0, 0.0],
                    )
                })
            })
            .collect();

        let indices: Vec<u16> = (0..resolution)
            .flat_map(|z| {
                (0..resolution).flat_map(move |x| {
                    let i0 = (z * (resolution + 1) + x) as u16;
                    let i1 = i0 + 1;
                    let i2 = i0 + (resolution + 1) as u16;
                    let i3 = i2 + 1;
                    [i0, i2, i1, i1, i2, i3]
                })
            })
            .collect();

        Self { vertices, indices }
    }

    #[must_use]
    pub fn sphere(radius: f32, sectors: usize, stacks: usize) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let sector_step = std::f32::consts::TAU / sectors as f32;
        let stack_step = std::f32::consts::PI / stacks as f32;

        for i in 0..=stacks {
            let stack_angle = std::f32::consts::PI / 2.0 - i as f32 * stack_step;
            let xy = radius * stack_angle.cos();
            let z = radius * stack_angle.sin();

            for j in 0..=sectors {
                let sector_angle = j as f32 * sector_step;
                let x = xy * sector_angle.cos();
                let y = xy * sector_angle.sin();

                let pos = [x, y, z];
                let uv = [j as f32 / sectors as f32, 1.0 - (i as f32 / stacks as f32)];
                let normal = [x / radius, y / radius, z / radius];
                vertices.push(Vertex::new(pos, uv, normal));
            }
        }

        for i in 0..stacks {
            let k1 = i * (sectors + 1);
            let k2 = k1 + sectors + 1;
            for j in 0..sectors {
                if i != 0 {
                    indices.push((k1 + j) as u16);
                    indices.push((k2 + j) as u16);
                    indices.push((k1 + j + 1) as u16);
                }
                if i != stacks - 1 {
                    indices.push((k1 + j + 1) as u16);
                    indices.push((k2 + j) as u16);
                    indices.push((k2 + j + 1) as u16);
                }
            }
        }

        Self { vertices, indices }
    }

    #[must_use]
    pub fn quad(width: f32, height: f32) -> Self {
        let hw = width / 2.0;
        let hh = height / 2.0;
        Self {
            vertices: vec![
                Vertex::new([-hw, -hh, 0.0], [0.0, 0.0], [0.0, 0.0, 1.0]),
                Vertex::new([hw, -hh, 0.0], [1.0, 0.0], [0.0, 0.0, 1.0]),
                Vertex::new([hw, hh, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0]),
                Vertex::new([-hw, hh, 0.0], [0.0, 1.0], [0.0, 0.0, 1.0]),
            ],
            indices: vec![0, 1, 2, 2, 3, 0],
        }
    }

    /// Returns the bounding box of the mesh vertices.
    pub fn bounding_box(&self) -> crate::math::AABB {
        if self.vertices.is_empty() {
            return crate::math::AABB::new(Vec3::ZERO, Vec3::ZERO);
        }
        let mut min = Vec3::from(self.vertices[0].pos);
        let mut max = min;
        for v in &self.vertices[1..] {
            let p = Vec3::from(v.pos);
            min = min.min(p);
            max = max.max(p);
        }
        crate::math::AABB::new(min, max)
    }
}

use crate::math::Vec3;
