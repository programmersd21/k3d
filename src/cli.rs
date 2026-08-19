use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Demo {
    Cube,
    Sphere,
    Torus,
    Cylinder,
    Cone,
    Icosphere,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum RenderMode {
    Smooth,
    Flat,
    Wireframe,
    Normals,
    Depth,
    Unlit,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Background {
    Solid,
    Transparent,
    Terminal,
    Gradient,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Theme {
    Default,
    Monochrome,
    Catppuccin,
    Gruvbox,
    TokyoNight,
    Nord,
}

#[derive(Debug, Parser)]
#[command(
    name = "k3d",
    about = "Interactive 3D graphics, directly in your terminal.",
    version
)]
pub struct Cli {
    pub model: Option<PathBuf>,
    #[arg(long, value_enum)]
    pub demo: Option<Demo>,
    #[arg(long, value_enum, default_value_t=RenderMode::Smooth)]
    pub mode: RenderMode,
    #[arg(long, value_enum, default_value_t=Background::Solid)]
    pub background: Background,
    #[arg(long, value_enum, default_value_t=Theme::Default)]
    pub theme: Theme,
    #[arg(long, default_value_t=60, value_parser=clap::value_parser!(u32).range(1..=240))]
    pub fps: u32,
    #[arg(long)]
    pub no_animation: bool,
    #[arg(long)]
    pub spin: bool,
    #[arg(long, default_value_t = 0.6)]
    pub scale: f32,
    #[arg(long)]
    pub wireframe: bool,
    #[arg(long, value_name = "PATH", conflicts_with = "benchmark")]
    pub screenshot: Option<PathBuf>,
    #[arg(long)]
    pub benchmark: bool,
    #[arg(long)]
    pub verbose: bool,
}

pub fn parse() -> Cli {
    Cli::parse()
}
