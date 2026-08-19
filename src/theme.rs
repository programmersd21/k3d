use crate::cli::Theme;
use glam::Vec3;
#[allow(dead_code)]
pub struct Palette {
    pub background: Vec3,
    pub object: Vec3,
    pub accent: Vec3,
}
pub fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Monochrome => Palette {
            background: Vec3::splat(0.03),
            object: Vec3::splat(0.78),
            accent: Vec3::splat(0.95),
        },
        Theme::Catppuccin => Palette {
            background: Vec3::new(0.06, 0.06, 0.10),
            object: Vec3::new(0.55, 0.72, 0.95),
            accent: Vec3::new(0.96, 0.72, 0.85),
        },
        Theme::Gruvbox => Palette {
            background: Vec3::new(0.12, 0.10, 0.08),
            object: Vec3::new(0.83, 0.57, 0.25),
            accent: Vec3::new(0.98, 0.75, 0.22),
        },
        Theme::TokyoNight => Palette {
            background: Vec3::new(0.04, 0.05, 0.10),
            object: Vec3::new(0.39, 0.66, 0.95),
            accent: Vec3::new(0.73, 0.55, 0.95),
        },
        Theme::Nord => Palette {
            background: Vec3::new(0.09, 0.12, 0.16),
            object: Vec3::new(0.52, 0.72, 0.78),
            accent: Vec3::new(0.56, 0.74, 0.96),
        },
        Theme::Default => Palette {
            background: Vec3::new(0.025, 0.03, 0.045),
            object: Vec3::new(0.42, 0.65, 0.95),
            accent: Vec3::new(0.35, 0.9, 0.8),
        },
    }
}
