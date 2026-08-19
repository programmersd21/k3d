use glam::Vec3;
use k3d::{camera::OrbitCamera, primitives, renderer::Framebuffer};

#[test]
fn camera_orbit_is_bounded_and_resettable() {
    let mut camera = OrbitCamera::default();
    camera.orbit(1000.0, 1000.0);
    assert!(camera.elevation <= 1.5);
    camera.reset();
    assert_eq!(camera.distance, 2.8);
}

#[test]
fn framebuffer_reuses_storage_on_same_size() {
    let mut buffer = Framebuffer::new(8, 4);
    let pixels = buffer.pixels.as_ptr();
    buffer.resize(8, 4);
    assert_eq!(pixels, buffer.pixels.as_ptr());
}

#[test]
fn cube_renders_pixels_and_depth() {
    let asset = primitives::cube(Vec3::new(0.3, 0.6, 0.9));
    assert_eq!(asset.mesh.triangle_count(), 12);
    let mut buffer = Framebuffer::new(96, 64);
    buffer.clear(Vec3::new(0.1, 0.2, 0.3));
    assert_eq!(&buffer.pixels[..4], &[25, 51, 76, 255]);
    assert!(buffer.depth.iter().all(|d| *d == 1.0));
    k3d::renderer::render(
        &asset,
        &mut buffer,
        glam::Mat4::IDENTITY,
        OrbitCamera::default().view(),
        k3d::cli::RenderMode::Smooth,
        Vec3::ZERO,
    );
    assert!(buffer.depth.iter().any(|d| *d < 1.0));
}

#[test]
fn every_builtin_demo_has_valid_indices() {
    let color = Vec3::ONE;
    let assets = [
        primitives::cube(color),
        primitives::sphere(color),
        primitives::torus(color),
        primitives::cylinder(color),
        primitives::cone(color),
        primitives::icosphere(color),
    ];
    for asset in assets {
        assert!(asset
            .mesh
            .indices
            .iter()
            .all(|&i| (i as usize) < asset.mesh.positions.len()));
        assert_eq!(asset.mesh.normals.len(), asset.mesh.positions.len());
    }
}
