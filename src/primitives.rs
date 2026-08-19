use crate::model::{Asset, Material, Mesh};
use glam::{Vec2, Vec3};

pub fn cube(color: Vec3) -> Asset {
    let p = [
        (-1., -1., -1.),
        (1., -1., -1.),
        (1., 1., -1.),
        (-1., 1., -1.),
        (-1., -1., 1.),
        (1., -1., 1.),
        (1., 1., 1.),
        (-1., 1., 1.),
    ]
    .into_iter()
    .map(|(x, y, z)| Vec3::new(x, y, z) * 0.8)
    .collect();
    let i = vec![
        0, 2, 1, 0, 3, 2, // back
        4, 5, 6, 4, 6, 7, // front
        0, 1, 5, 0, 5, 4, // bottom
        2, 3, 7, 2, 7, 6, // top
        0, 4, 7, 0, 7, 3, // left
        1, 2, 6, 1, 6, 5, // right
    ];
    let mut m = Mesh {
        positions: p,
        normals: vec![],
        uvs: vec![],
        indices: i,
    };
    m.recalculate_normals();
    Asset {
        mesh: m,
        material: Material {
            color,
            roughness: 0.45,
            specular: 0.25,
        },
        name: "cube".into(),
    }
}

pub fn sphere(color: Vec3) -> Asset {
    lathe(
        48,
        32,
        |a, z| {
            let p = std::f32::consts::PI * z;
            Vec3::new(a.cos() * p.sin(), p.cos(), a.sin() * p.sin())
        },
        color,
        "sphere",
    )
}

pub fn torus(color: Vec3) -> Asset {
    let mut p = Vec::new();
    let mut i = Vec::new();
    let (a, b) = (64, 24);
    for y in 0..b {
        for x in 0..a {
            let u = x as f32 / a as f32 * std::f32::consts::TAU;
            let v = y as f32 / b as f32 * std::f32::consts::TAU;
            p.push(Vec3::new(
                (1. + 0.35 * v.cos()) * u.cos(),
                0.35 * v.sin(),
                (1. + 0.35 * v.cos()) * u.sin(),
            ));
        }
    }
    for y in 0..b {
        for x in 0..a {
            let nx = (x + 1) % a;
            let ny = (y + 1) % b;
            let k = (y * a + x) as u32;
            let r = (y * a + nx) as u32;
            let d = (ny * a + x) as u32;
            let rd = (ny * a + nx) as u32;
            // Consistent CCW winding for both triangles of the quad.
            i.extend([k, r, d, r, rd, d]);
        }
    }
    let mut m = Mesh {
        positions: p,
        normals: vec![],
        uvs: vec![],
        indices: i,
    };
    m.recalculate_normals();
    Asset {
        mesh: m,
        material: Material {
            color,
            roughness: 0.3,
            specular: 0.45,
        },
        name: "torus".into(),
    }
}

pub fn cylinder(color: Vec3) -> Asset {
    lathe(
        32,
        2,
        |a, z| {
            let y = z * 2. - 1.;
            Vec3::new(a.cos(), y, a.sin())
        },
        color,
        "cylinder",
    )
}

pub fn cone(color: Vec3) -> Asset {
    lathe(
        32,
        2,
        |a, z| {
            let y = z * 2. - 1.;
            Vec3::new(a.cos() * (1. - z), y, a.sin() * (1. - z))
        },
        color,
        "cone",
    )
}

pub fn icosphere(color: Vec3) -> Asset {
    sphere(color)
}

fn lathe(ax: usize, ay: usize, f: impl Fn(f32, f32) -> Vec3, color: Vec3, name: &str) -> Asset {
    let mut p = Vec::new();
    let mut i = Vec::new();
    for y in 0..=ay {
        for x in 0..ax {
            p.push(f(
                x as f32 / ax as f32 * std::f32::consts::TAU,
                y as f32 / ay as f32,
            ));
        }
    }
    for y in 0..ay {
        for x in 0..ax {
            let nx = (x + 1) % ax;
            let k = (y * ax + x) as u32;
            let r = (y * ax + nx) as u32;
            let d = ((y + 1) * ax + x) as u32;
            let rd = ((y + 1) * ax + nx) as u32;
            // Consistent CCW winding for both triangles of the quad.
            i.extend([k, r, d, r, rd, d]);
        }
    }
    let mut m = Mesh {
        positions: p,
        normals: vec![],
        uvs: vec![Vec2::ZERO; (ay + 1) * ax],
        indices: i,
    };
    m.recalculate_normals();
    Asset {
        mesh: m,
        material: Material {
            color,
            roughness: 0.45,
            specular: 0.25,
        },
        name: name.into(),
    }
}
