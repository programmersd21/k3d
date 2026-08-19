#[cfg(not(target_os = "linux"))]
compile_error!("k3d currently supports Linux only");

mod app;
mod camera;
mod cli;
mod error;
mod formats;
mod kitty;
mod model;
mod primitives;
mod renderer;
mod theme;

fn main() -> anyhow::Result<()> {
    app::run(cli::parse())
}
