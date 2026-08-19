use crate::{
    camera::OrbitCamera,
    cli::{Background, Cli, Demo, RenderMode},
    formats, kitty, primitives,
    renderer::{self, Framebuffer},
    theme,
};
use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use glam::{Mat4, Vec3};
use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

pub fn run(cli: Cli) -> anyhow::Result<()> {
    let pal = theme::palette(cli.theme);
    let mut asset = cli
        .model
        .as_ref()
        .map(|p| formats::load(p, pal.object))
        .transpose()?;
    if asset.is_none() {
        asset = Some(match cli.demo.unwrap_or(Demo::Torus) {
            Demo::Cube => primitives::cube(pal.object),
            Demo::Sphere => primitives::sphere(pal.object),
            Demo::Torus => primitives::torus(pal.object),
            Demo::Cylinder => primitives::cylinder(pal.object),
            Demo::Cone => primitives::cone(pal.object),
            Demo::Icosphere => primitives::icosphere(pal.object),
        });
    }
    let asset = asset.unwrap_or_else(|| primitives::cube(pal.object));

    if cli.screenshot.is_some() || cli.benchmark {
        return run_headless(&asset, &cli, pal.background);
    }
    if !kitty::supported() {
        anyhow::bail!(
            "k3d requires Kitty Graphics Protocol support. Run inside kitty or another \
             compatible terminal (detected: {}).",
            std::env::var("TERM").unwrap_or_else(|_| "unknown".into())
        );
    }

    let _raw = TerminalGuard::new()?;
    let mut fb = Framebuffer::new(1, 1);
    let mut camera = OrbitCamera::default();
    let mut presenter = kitty::Presenter::new();
    let default_demo = cli.model.is_none() && cli.demo.is_none();
    let mut spin = (cli.spin || default_demo) && !cli.no_animation;
    let mut paused = false;
    let mut stats = false;
    let mut help = false;
    let mut mode = if cli.wireframe {
        RenderMode::Wireframe
    } else {
        cli.mode
    };
    let mut background = cli.background;
    let mut angle = 0.0f32;
    let mut drag: Option<(MouseButton, u16, u16)> = None;
    let mut dirty = true;
    let frame = Duration::from_secs_f64(1.0 / cli.fps as f64);
    let mut last = Instant::now();

    loop {
        while event::poll(Duration::ZERO)? {
            dirty = true;
            match event::read()? {
                Event::Key(k) => {
                    if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) {
                        return Ok(());
                    }
                    match k.code {
                        KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('r') => camera.reset(),
                        KeyCode::Char('a') => spin = !spin,
                        KeyCode::Char(' ') => paused = !paused,
                        KeyCode::Char('f') => stats = !stats,
                        KeyCode::Char('?') => help = !help,
                        KeyCode::Char('1') => mode = RenderMode::Smooth,
                        KeyCode::Char('2') => mode = RenderMode::Flat,
                        KeyCode::Char('3') | KeyCode::Char('w') => mode = RenderMode::Wireframe,
                        KeyCode::Char('4') => mode = RenderMode::Normals,
                        KeyCode::Char('5') => mode = RenderMode::Depth,
                        KeyCode::Char('6') => mode = RenderMode::Unlit,
                        KeyCode::Char('b') => {
                            background = match background {
                                Background::Solid => Background::Gradient,
                                Background::Gradient => Background::Terminal,
                                _ => Background::Solid,
                            }
                        }
                        KeyCode::Left | KeyCode::Char('h') => camera.orbit(-20.0, 0.0),
                        KeyCode::Right | KeyCode::Char('l') => camera.orbit(20.0, 0.0),
                        KeyCode::Up | KeyCode::Char('k') => camera.orbit(0.0, -20.0),
                        KeyCode::Down | KeyCode::Char('j') => camera.orbit(0.0, 20.0),
                        KeyCode::Char('+') | KeyCode::Char('=') => camera.zoom(1.0),
                        KeyCode::Char('-') => camera.zoom(-1.0),
                        _ => {}
                    }
                }
                Event::Mouse(m) => match m.kind {
                    MouseEventKind::ScrollUp => camera.zoom(1.0),
                    MouseEventKind::ScrollDown => camera.zoom(-1.0),
                    MouseEventKind::Down(button @ (MouseButton::Left | MouseButton::Right)) => {
                        drag = Some((button, m.column, m.row));
                    }
                    MouseEventKind::Drag(button) => {
                        if let Some((active, x, y)) = drag {
                            if active == button {
                                let dx = m.column as f32 - x as f32;
                                let dy = m.row as f32 - y as f32;
                                if button == MouseButton::Left {
                                    camera.orbit(dx * 8.0, dy * 8.0);
                                } else {
                                    camera.pan(-dx * 8.0, dy * 8.0);
                                }
                                drag = Some((button, m.column, m.row));
                            }
                        }
                    }
                    MouseEventKind::Up(_) => drag = None,
                    _ => {}
                },
                Event::Resize(_, _) => dirty = true,
                _ => {}
            }
        }

        let now = Instant::now();
        if !dirty && (!spin || paused) {
            std::thread::sleep(Duration::from_millis(8));
            continue;
        }
        if now.duration_since(last) < frame {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }
        // Clamp dt to avoid large jumps after lag or rate-limit skips.
        let dt = now.duration_since(last).as_secs_f32().min(0.1);
        last = now;

        if spin && !paused {
            angle += dt * 0.75;
        }

        let (w, h) = terminal::size()?;
        let scale = cli.scale.clamp(0.1, 2.0);
        let window = terminal::window_size()?;
        let pixel_width = if window.width == 0 {
            w as usize * 8
        } else {
            window.width as usize
        };
        let pixel_height = if window.height == 0 {
            h as usize * 16
        } else {
            window.height as usize
        };
        fb.resize(
            (pixel_width as f32 * scale) as usize,
            (pixel_height as f32 * scale) as usize,
        );

        renderer::render(
            &asset,
            &mut fb,
            Mat4::from_rotation_y(angle),
            camera.view(),
            mode,
            match background {
                Background::Gradient => pal.background * (0.65 + 0.35 * angle.sin().abs()),
                Background::Terminal | Background::Transparent => Vec3::ZERO,
                Background::Solid => pal.background,
            },
        );

        presenter.present(&fb, w, h)?;
        draw_overlay(&asset, &fb, stats, help, mode);
        io::stdout().flush()?;
        dirty = false;
    }
}

/// Draws an informational overlay on top of the rendered image.
fn draw_overlay(
    asset: &crate::model::Asset,
    fb: &Framebuffer,
    stats: bool,
    help: bool,
    mode: RenderMode,
) {
    let mut out = io::stdout();
    // Fully opaque overlay background strip so text is always legible.
    let bg = "\x1b[48;5;234m";
    let fg = "\x1b[38;5;252m";
    let reset = "\x1b[0m";
    if help {
        // Multi-line help overlay anchored at top-left.
        let lines = [
            " k3d  controls",
            "",
            " LMB drag  rotate    RMB drag  pan",
            " Wheel     zoom      R         reset",
            " 1-6       shading   A         auto-spin",
            " F         statistics  B         background",
            " Q / Esc   quit       ?         close help",
        ];
        for (i, line) in lines.iter().enumerate() {
            let _ = write!(out, "\x1b[{};1H{bg}{fg} {line:<42}{reset}", i + 1);
        }
    } else if stats {
        let mode_name = match mode {
            RenderMode::Smooth => "smooth",
            RenderMode::Flat => "flat",
            RenderMode::Wireframe => "wireframe",
            RenderMode::Normals => "normals",
            RenderMode::Depth => "depth",
            RenderMode::Unlit => "unlit",
        };
        let info = format!(
            " k3d  {} \u{2022} {} triangles \u{2022} {}\u{00d7}{} ",
            mode_name,
            asset.mesh.triangle_count(),
            fb.width,
            fb.height
        );
        let _ = write!(out, "\x1b[1;1H{bg}{fg}{info}{reset}");
    } else {
        // Clear the first few lines where overlay text may have been drawn.
        for i in 1..=8 {
            let _ = write!(out, "\x1b[{i};1H\x1b[2K");
        }
    }
}

fn run_headless(asset: &crate::model::Asset, cli: &Cli, background: Vec3) -> anyhow::Result<()> {
    let mut fb = Framebuffer::new(960, 540);
    let start = Instant::now();
    let frames = if cli.benchmark { 120 } else { 1 };
    for frame in 0..frames {
        renderer::render(
            asset,
            &mut fb,
            Mat4::from_rotation_y(frame as f32 * 0.02),
            OrbitCamera::default().view(),
            if cli.wireframe {
                RenderMode::Wireframe
            } else {
                cli.mode
            },
            background,
        );
    }
    if let Some(path) = &cli.screenshot {
        let file = std::fs::File::create(path)?;
        let mut encoder = png::Encoder::new(file, fb.width as u32, fb.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&fb.pixels)?;
        return Ok(());
    }
    let seconds = start.elapsed().as_secs_f64();
    println!(
        "frames={} fps={:.1} frame_ms={:.2} triangles={} resolution={}x{}",
        frames,
        frames as f64 / seconds,
        seconds * 1000.0 / frames as f64,
        asset.mesh.triangle_count(),
        fb.width,
        fb.height
    );
    Ok(())
}

struct TerminalGuard;
impl TerminalGuard {
    fn new() -> anyhow::Result<Self> {
        let mut o = io::stdout();
        execute!(o, EnterAlternateScreen, cursor::Hide, EnableMouseCapture)?;
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut o = io::stdout();
        let _ = terminal::disable_raw_mode();
        let _ = execute!(o, DisableMouseCapture, cursor::Show, LeaveAlternateScreen);
    }
}
