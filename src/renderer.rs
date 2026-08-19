use crate::{cli::RenderMode, model::Asset};
use glam::{Mat4, Vec3, Vec4};

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
    pub depth: Vec<f32>,
}
impl Framebuffer {
    pub fn new(w: usize, h: usize) -> Self {
        let mut s = Self {
            width: 0,
            height: 0,
            pixels: Vec::new(),
            depth: Vec::new(),
        };
        s.resize(w, h);
        s
    }
    pub fn resize(&mut self, w: usize, h: usize) {
        self.width = w.max(1);
        self.height = h.max(1);
        self.pixels.resize(self.width * self.height * 4, 0);
        self.depth.resize(self.width * self.height, 1.0);
    }
    pub fn clear(&mut self, color: Vec3) {
        let c = [
            (color.x.clamp(0., 1.) * 255.) as u8,
            (color.y.clamp(0., 1.) * 255.) as u8,
            (color.z.clamp(0., 1.) * 255.) as u8,
            255,
        ];
        for p in self.pixels.chunks_exact_mut(4) {
            p.copy_from_slice(&c);
        }
        self.depth.fill(1.0);
    }
}
pub fn render(
    asset: &Asset,
    fb: &mut Framebuffer,
    transform: Mat4,
    view: Mat4,
    mode: RenderMode,
    bg: Vec3,
) {
    fb.clear(bg);
    let aspect = fb.width as f32 / fb.height.max(1) as f32;
    let mvp = Mat4::perspective_rh(0.9, aspect, 0.05, 100.) * view * transform;
    let normal_matrix = transform.inverse().transpose();
    let light = Vec3::new(-0.5, 0.8, 0.6).normalize();
    let view_dir = Vec3::new(0., 0., 1.); // approximate view direction for specular
    let mesh = &asset.mesh;
    for tri in mesh.indices.chunks_exact(3) {
        if tri
            .iter()
            .any(|&index| index as usize >= mesh.positions.len())
        {
            continue;
        }
        let mut v = [Vec4::ZERO; 3];
        let mut world = [Vec3::ZERO; 3];
        for j in 0..3 {
            let i = tri[j] as usize;
            world[j] = (transform * mesh.positions[i].extend(1.)).truncate();
            v[j] = mvp * mesh.positions[i].extend(1.);
        }
        // Near-plane clip: skip triangles with any vertex behind the camera.
        if v.iter().any(|p| p.w <= 0.) {
            continue;
        }
        let ndc = [
            v[0].truncate() / v[0].w,
            v[1].truncate() / v[1].w,
            v[2].truncate() / v[2].w,
        ];
        let sx = |p: Vec3| Vec2Like {
            x: (p.x * 0.5 + 0.5) * fb.width as f32,
            y: (1. - (p.y * 0.5 + 0.5)) * fb.height as f32,
            z: p.z * 0.5 + 0.5,
        };
        let p = [sx(ndc[0]), sx(ndc[1]), sx(ndc[2])];
        let area = (p[1].x - p[0].x) * (p[2].y - p[0].y) - (p[1].y - p[0].y) * (p[2].x - p[0].x);
        if area.abs() < f32::EPSILON {
            continue;
        }
        // `edge` uses the opposite cross-product orientation from `area`.
        // Negating the reciprocal accepts either screen-space winding.
        let inv_area = -1.0 / area;
        let minx = p
            .iter()
            .map(|x| x.x)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.) as usize;
        let maxx = p
            .iter()
            .map(|x| x.x)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(fb.width as f32 - 1.) as usize;
        let miny = p
            .iter()
            .map(|x| x.y)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.) as usize;
        let maxy = p
            .iter()
            .map(|x| x.y)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(fb.height as f32 - 1.) as usize;
        if minx > maxx || miny > maxy {
            continue;
        }
        let face = (world[1] - world[0])
            .cross(world[2] - world[0])
            .normalize_or_zero();
        let base = asset.material.color;
        let mut vertex_normals = [face; 3];
        if mesh.normals.len() == mesh.positions.len() {
            for j in 0..3 {
                vertex_normals[j] = (normal_matrix * mesh.normals[tri[j] as usize].extend(0.))
                    .truncate()
                    .normalize_or_zero();
            }
        }
        let flat_color = if mode == RenderMode::Flat {
            let n_dot_l = face.dot(light).max(0.0);
            let ambient = 0.24;
            let diffuse = 0.68 * n_dot_l;
            let specular = if n_dot_l > 0.0 {
                let half = (light + view_dir).normalize_or_zero();
                let dot = face.dot(half).max(0.0);
                let d2 = dot * dot;
                let d4 = d2 * d2;
                let d8 = d4 * d4;
                let d16 = d8 * d8;
                let d32 = d16 * d16;
                asset.material.specular * d32
            } else {
                0.0
            };
            base * (ambient + diffuse) + Vec3::splat(specular)
        } else {
            Vec3::ZERO
        };

        let dx0 = p[2].x - p[1].x;
        let dy0 = p[2].y - p[1].y;
        let dx1 = p[0].x - p[2].x;
        let dy1 = p[0].y - p[2].y;
        let dx2 = p[1].x - p[0].x;
        let dy2 = p[1].y - p[0].y;

        let step_w0 = dy0 * inv_area;
        let step_w1 = dy1 * inv_area;
        let step_w2 = dy2 * inv_area;

        let x_start = minx as f32 + 0.5;

        for y in miny..=maxy {
            let y_val = y as f32 + 0.5;
            let mut w0 = ((x_start - p[1].x) * dy0 - (y_val - p[1].y) * dx0) * inv_area;
            let mut w1 = ((x_start - p[2].x) * dy1 - (y_val - p[2].y) * dx1) * inv_area;
            let mut w2 = ((x_start - p[0].x) * dy2 - (y_val - p[0].y) * dx2) * inv_area;

            let row_offset = y * fb.width;

            for x in minx..=maxx {
                if w0 >= 0. && w1 >= 0. && w2 >= 0. {
                    if mode == RenderMode::Wireframe && w0.min(w1).min(w2) >= 0.025 {
                        w0 += step_w0;
                        w1 += step_w1;
                        w2 += step_w2;
                        continue;
                    }
                    let d = w0 * p[0].z + w1 * p[1].z + w2 * p[2].z;
                    if !(0.0..=1.0).contains(&d) {
                        w0 += step_w0;
                        w1 += step_w1;
                        w2 += step_w2;
                        continue;
                    }
                    let at = row_offset + x;
                    if d < fb.depth[at] {
                        let color = match mode {
                            RenderMode::Unlit | RenderMode::Wireframe => base,
                            RenderMode::Depth => Vec3::splat(1.0 - d),
                            RenderMode::Normals => {
                                let normal = (vertex_normals[0] * w0
                                    + vertex_normals[1] * w1
                                    + vertex_normals[2] * w2)
                                    .normalize_or_zero();
                                normal * 0.5 + Vec3::splat(0.5)
                            }
                            RenderMode::Flat => flat_color,
                            _ => {
                                let normal = (vertex_normals[0] * w0
                                    + vertex_normals[1] * w1
                                    + vertex_normals[2] * w2)
                                    .normalize_or_zero();
                                let n_dot_l = normal.dot(light).max(0.0);
                                let ambient = 0.24;
                                let diffuse = 0.68 * n_dot_l;
                                let specular = if n_dot_l > 0.0 {
                                    let half = (light + view_dir).normalize_or_zero();
                                    let dot = normal.dot(half).max(0.0);
                                    let d2 = dot * dot;
                                    let d4 = d2 * d2;
                                    let d8 = d4 * d4;
                                    let d16 = d8 * d8;
                                    let d32 = d16 * d16;
                                    asset.material.specular * d32
                                } else {
                                    0.0
                                };
                                base * (ambient + diffuse) + Vec3::splat(specular)
                            }
                        };
                        let col = [
                            (color.x.clamp(0., 1.) * 255.) as u8,
                            (color.y.clamp(0., 1.) * 255.) as u8,
                            (color.z.clamp(0., 1.) * 255.) as u8,
                            255,
                        ];
                        fb.depth[at] = d;
                        fb.pixels[at * 4..at * 4 + 4].copy_from_slice(&col);
                    }
                }
                w0 += step_w0;
                w1 += step_w1;
                w2 += step_w2;
            }
        }
    }
}
#[derive(Clone, Copy)]
struct Vec2Like {
    x: f32,
    y: f32,
    z: f32,
}
