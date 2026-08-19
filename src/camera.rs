use glam::{Mat4, Vec3};

pub struct OrbitCamera {
    pub azimuth: f32,
    pub elevation: f32,
    pub distance: f32,
    pub target: Vec3,
}
impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            azimuth: 0.7,
            elevation: 0.42,
            distance: 2.8,
            target: Vec3::ZERO,
        }
    }
}
impl OrbitCamera {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn eye(&self) -> Vec3 {
        let ce = self.elevation.cos();
        self.target
            + Vec3::new(
                self.azimuth.cos() * ce,
                self.elevation.sin(),
                self.azimuth.sin() * ce,
            ) * self.distance
    }
    pub fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.azimuth -= dx * 0.008;
        self.elevation = (self.elevation + dy * 0.008).clamp(-1.5, 1.5);
    }
    pub fn zoom(&mut self, amount: f32) {
        self.distance = (self.distance * (1.0 - amount * 0.12)).clamp(0.25, 100.0);
    }
    pub fn pan(&mut self, dx: f32, dy: f32) {
        // Derive the camera-local right and up vectors from the current
        // azimuth / elevation so that panning feels correct at any angle.
        let ce = self.elevation.cos();
        let forward = Vec3::new(
            self.azimuth.cos() * ce,
            self.elevation.sin(),
            self.azimuth.sin() * ce,
        )
        .normalize_or_zero();
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();
        self.target += right * dx * 0.005 + up * dy * 0.005;
    }
}
