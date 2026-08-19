use glam::Vec3;
use std::fs;

#[test]
fn obj_loader_triangulates_quads() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("quad.obj");
    fs::write(&path, "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n").unwrap();
    let asset = k3d::formats::load(&path, Vec3::ONE).unwrap();
    assert_eq!(asset.mesh.positions.len(), 4);
    assert_eq!(asset.mesh.triangle_count(), 2);
}

#[test]
fn ascii_stl_loader_reads_facets() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("triangle.stl");
    fs::write(&path, "solid test\n facet normal 0 0 1\n  outer loop\n   vertex 0 0 0\n   vertex 1 0 0\n   vertex 0 1 0\n  endloop\n endfacet\nendsolid test\n").unwrap();
    let asset = k3d::formats::load(&path, Vec3::ONE).unwrap();
    assert_eq!(asset.mesh.triangle_count(), 1);
    assert_eq!(asset.mesh.normals.len(), 3);
}
