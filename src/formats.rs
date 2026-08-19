use crate::{
    error::K3dError,
    model::{Asset, Material, Mesh},
};
use glam::Vec3;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

const MAX_OBJ_TRIANGLES: usize = 120_000;

pub fn load(path: &Path, color: Vec3) -> Result<Asset, K3dError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "obj" => obj(path, color),
        "stl" => {
            let data = fs::read(path).map_err(|e| K3dError::Model {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;
            stl(&data, color, path)
        }
        "gltf" | "glb" => gltf(path, color),
        _ => Err(K3dError::Model {
            path: path.display().to_string(),
            reason: "supported formats are OBJ and STL in this build".into(),
        }),
    }
}

fn gltf(path: &Path, color: Vec3) -> Result<Asset, K3dError> {
    let (document, buffers, _) = gltf::import(path).map_err(|e| K3dError::Model {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    let primitive = document
        .meshes()
        .flat_map(|m| m.primitives())
        .next()
        .ok_or_else(|| K3dError::InvalidModel("GLTF contains no mesh primitives".into()))?;
    let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|b| b.0.as_slice()));
    let positions: Vec<Vec3> = reader
        .read_positions()
        .ok_or_else(|| K3dError::InvalidModel("GLTF primitive has no positions".into()))?
        .map(Vec3::from_array)
        .collect();
    let indices: Vec<u32> = reader
        .read_indices()
        .map(|i| i.into_u32().collect())
        .unwrap_or_else(|| (0..positions.len() as u32).collect());
    let normals = reader
        .read_normals()
        .map(|n| n.map(Vec3::from_array).collect())
        .unwrap_or_default();
    let mut mesh = Mesh {
        positions,
        normals,
        uvs: vec![],
        indices,
    };
    if mesh.normals.len() != mesh.positions.len() {
        mesh.recalculate_normals();
    }
    Ok(Asset {
        mesh,
        material: Material {
            color,
            roughness: 0.45,
            specular: 0.25,
        },
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
    })
}
fn obj(path: &Path, color: Vec3) -> Result<Asset, K3dError> {
    let file = File::open(path).map_err(|e| K3dError::Model {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    let mut p = Vec::new();
    let mut idx = Vec::new();
    let mut face_number = 0usize;
    let mut sampling_step = 1usize;
    while reader.read_line(&mut line).map_err(|e| K3dError::Model {
        path: path.display().to_string(),
        reason: e.to_string(),
    })? > 0
    {
        let l = line.trim_start();
        if let Some(count) = l
            .strip_prefix("#Face Count ")
            .and_then(|s| s.trim().parse::<usize>().ok())
        {
            sampling_step = count.div_ceil(MAX_OBJ_TRIANGLES).max(1);
        }
        let mut w = l.split_whitespace();
        match w.next() {
            Some("v") => {
                let a: [f32; 3] = match (w.next(), w.next(), w.next()) {
                    (Some(x), Some(y), Some(z)) => match (x.parse(), y.parse(), z.parse()) {
                        (Ok(x), Ok(y), Ok(z)) => [x, y, z],
                        _ => {
                            line.clear();
                            continue;
                        }
                    },
                    _ => {
                        line.clear();
                        continue;
                    }
                };
                p.push(Vec3::from_array(a));
            }
            Some("f") => {
                face_number += 1;
                if !face_number.is_multiple_of(sampling_step) {
                    line.clear();
                    continue;
                }
                let a: Vec<i64> = w
                    .take(64)
                    .filter_map(|x| x.split('/').next()?.parse().ok())
                    .collect();
                if a.len() >= 3 {
                    let mut resolved = [0u32; 64];
                    let mut valid = true;
                    for (slot, raw) in a.iter().enumerate() {
                        let index = if *raw < 0 {
                            p.len() as i64 + raw
                        } else {
                            raw - 1
                        };
                        if !(0..p.len() as i64).contains(&index) {
                            valid = false;
                            break;
                        }
                        resolved[slot] = index as u32;
                    }
                    if valid {
                        for t in 1..a.len() - 1 {
                            idx.extend([resolved[0], resolved[t], resolved[t + 1]]);
                        }
                    }
                }
            }
            _ => {}
        }
        if idx.len() / 3 >= MAX_OBJ_TRIANGLES {
            break;
        }
        line.clear();
    }
    if p.is_empty() || idx.is_empty() {
        return Err(K3dError::Model {
            path: path.display().to_string(),
            reason: "OBJ has no renderable triangles".into(),
        });
    }
    let mut m = Mesh {
        positions: p,
        normals: vec![],
        uvs: vec![],
        indices: idx,
    };
    compact(&mut m);
    m.normalize();
    m.recalculate_normals();
    Ok(Asset {
        mesh: m,
        material: Material {
            color,
            roughness: 0.45,
            specular: 0.25,
        },
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
    })
}

fn compact(mesh: &mut Mesh) {
    let mut remap = vec![u32::MAX; mesh.positions.len()];
    let mut positions = Vec::new();
    let mut indices = Vec::with_capacity(mesh.indices.len());
    for &index in &mesh.indices {
        let old = index as usize;
        if remap[old] == u32::MAX {
            remap[old] = positions.len() as u32;
            positions.push(mesh.positions[old]);
        }
        indices.push(remap[old]);
    }
    mesh.positions = positions;
    mesh.indices = indices;
}
fn stl(data: &[u8], color: Vec3, path: &Path) -> Result<Asset, K3dError> {
    let text = String::from_utf8_lossy(data);
    let mut ascii = Vec::new();
    for l in text.lines() {
        let w: Vec<_> = l.split_whitespace().collect();
        if w.first() == Some(&"vertex") {
            if let (Some(a), Some(b), Some(c)) = (
                w.get(1).and_then(|x| x.parse().ok()),
                w.get(2).and_then(|x| x.parse().ok()),
                w.get(3).and_then(|x| x.parse().ok()),
            ) {
                ascii.push(Vec3::new(a, b, c));
            }
        }
    }
    let positions = if ascii.len() >= 3 {
        ascii
    } else {
        if data.len() < 84 {
            return Err(K3dError::Model {
                path: path.display().to_string(),
                reason: "invalid STL".into(),
            });
        }
        let n = u32::from_le_bytes(data[80..84].try_into().unwrap_or([0; 4])) as usize;
        let mut out = Vec::with_capacity(n * 3);
        for chunk in data[84..].chunks_exact(50).take(n) {
            for j in 0..3 {
                let o = 12 + j * 12;
                out.push(Vec3::new(
                    f32::from_le_bytes(chunk[o..o + 4].try_into().unwrap()),
                    f32::from_le_bytes(chunk[o + 4..o + 8].try_into().unwrap()),
                    f32::from_le_bytes(chunk[o + 8..o + 12].try_into().unwrap()),
                ));
            }
        }
        out
    };
    if positions.len() < 3 {
        return Err(K3dError::Model {
            path: path.display().to_string(),
            reason: "STL has no triangles".into(),
        });
    }
    let indices = (0..positions.len() as u32).collect();
    let mut m = Mesh {
        positions,
        normals: vec![],
        uvs: vec![],
        indices,
    };
    m.recalculate_normals();
    Ok(Asset {
        mesh: m,
        material: Material {
            color,
            roughness: 0.5,
            specular: 0.2,
        },
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
    })
}
