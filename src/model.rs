use glam::{Vec2, Vec3};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
}
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Material {
    pub color: Vec3,
    pub roughness: f32,
    pub specular: f32,
}
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Asset {
    pub mesh: Mesh,
    pub material: Material,
    pub name: String,
}

#[allow(dead_code)]
impl Mesh {
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
    pub fn recalculate_normals(&mut self) {
        self.normals = vec![Vec3::ZERO; self.positions.len()];
        for tri in self.indices.chunks_exact(3) {
            let a = self.positions[tri[0] as usize];
            let b = self.positions[tri[1] as usize];
            let c = self.positions[tri[2] as usize];
            let n = (b - a).cross(c - a).normalize_or_zero();
            for &i in tri {
                self.normals[i as usize] += n;
            }
        }
        for n in &mut self.normals {
            *n = n.normalize_or_zero();
        }
    }

    pub fn normalize(&mut self) {
        if self.positions.is_empty() {
            return;
        }
        let mut min = self.positions[0];
        let mut max = min;
        for &p in &self.positions[1..] {
            min = min.min(p);
            max = max.max(p);
        }
        let center = (min + max) * 0.5;
        let extent = (max - min).max_element();
        if extent.is_finite() && extent > f32::EPSILON {
            let scale = 2.0 / extent;
            for p in &mut self.positions {
                *p = (*p - center) * scale;
            }
        }
    }
}
