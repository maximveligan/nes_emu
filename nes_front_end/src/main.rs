use std::path::Path;

use config::ButtonLayout;
use config::Config;
use nes_emu::controller::Button;
use nes_emu::rom::load_rom;
use nes_emu::NesEmulator;
// use sha3::Digest;
// use sha3::Sha3_256;
use std::convert::Infallible;
use std::error::Error;
use std::fs::File;
use std::io::Read;

use glfw::{Action, Context, Key, SwapInterval, WindowEvent, WindowMode};
use luminance::context::GraphicsContext;
use luminance::pipeline::{PipelineState, TextureBinding};
use luminance::pixel::{NormRGB8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::Uniform;
use luminance::tess::Mode;
use luminance::texture::{Dim2, MagFilter, MinFilter, Sampler, TexelUpload, Texture};
use luminance::UniformInterface;
use luminance_glfw::{GlfwSurface, GlfwSurfaceError};
use std::collections::HashMap;

use log::*;

pub mod config;

const NES_SCREEN_WIDTH: u32 = 256;
const NES_SCREEN_HEIGHT: u32 = 240;
const WINDOW_SCALE: f32 = 3.;

const VS: &str = include_str!("shaders/vs.glsl");
const FS: &str = include_str!("shaders/fs.glsl");

#[derive(UniformInterface)]
struct ShaderInterface {
    tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

fn _get_save_state_name(rom_path: &Path) -> Result<&str, Box<dyn Error>> {
    if let Some(os_stem) = rom_path.file_stem() {
        Ok(os_stem.to_str().ok_or("Failed to convert from utf-8")?)
    } else {
        warn!("Rom file name did not have an extension");
        Ok(rom_path.to_str().ok_or("Failed to convert path to UTF-8")?)
    }
}

fn init_emulator(path: &std::path::Path) -> Result<NesEmulator, Box<dyn Error>> {
    let mut raw_rom = File::open(path)?;
    let mut raw_bytes = Vec::new();
    raw_rom.read_to_end(&mut raw_bytes)?;

    let rom = load_rom(&raw_bytes)?;

    Ok(NesEmulator::new(rom))
}

fn set_ctrl_state(
    key: Key,
    action: Action,
    nes: &mut NesEmulator,
    ctrl1: &HashMap<Key, Button>,
    ctrl2: &HashMap<Key, Button>,
) {
    let state = action != Action::Release;
    if let Some(button) = ctrl1.get(&key) {
        nes.cpu.mmu.ctrl0.set_button_state(*button, state);
    }

    if let Some(button) = ctrl2.get(&key) {
        nes.cpu.mmu.ctrl1.set_button_state(*button, state);
    }
}

fn _save_state(save_name: &str, nes: &NesEmulator) -> Result<String, Box<dyn Error>> {
    let mut file = File::create(save_name)?;
    nes.get_state().save(&mut file)?;
    Ok(format!("Successfully saved state: {}", save_name))
}

fn _load_state(save_name: &str, nes: &mut NesEmulator) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(save_name)?;
    let state = nes_emu::state::State::load(&mut file)?;
    nes.load_state(state);
    Ok("Loaded state successfully".to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let config = Config::load_config("./config.toml".to_string())?;

    let ctrl0 = ButtonLayout::make_ctrl_map(&config.ctrl1_layout)?;
    let ctrl1 = ButtonLayout::make_ctrl_map(&config.ctrl2_layout)?;

    let surface = GlfwSurface::new(|glfw| {
        let (mut window, events) = glfw
            .create_window(
                (NES_SCREEN_WIDTH as f32 * WINDOW_SCALE).ceil() as u32,
                (NES_SCREEN_HEIGHT as f32 * WINDOW_SCALE).ceil() as u32,
                "res",
                WindowMode::Windowed,
            )
            .unwrap();
        window.make_current();
        window.set_all_polling(true);
        glfw.set_swap_interval(SwapInterval::Sync(1));

        Ok::<_, GlfwSurfaceError<Infallible>>((window, events))
    })
    .expect("GLFW surface creation");

    let mut context = surface.context;
    let events = surface.events_rx;
    let back_buffer = context.back_buffer().expect("back buffer");

    let mut program = context
        .new_shader_program::<(), (), ShaderInterface>()
        .from_strings(VS, None, None, FS)
        .expect("program creation")
        .ignore_warnings();

    let tess = context
        .new_tess()
        .set_render_vertex_nb(4)
        .set_mode(Mode::TriangleFan)
        .build()
        .unwrap();

    let nearest_sampler = Sampler {
        min_filter: MinFilter::Nearest,
        mag_filter: MagFilter::Nearest,
        ..Sampler::default()
    };

    let mut nes_fb_texture: Texture<_, Dim2, NormRGB8UI> = Texture::new(
        &mut context,
        [NES_SCREEN_WIDTH, NES_SCREEN_HEIGHT],
        nearest_sampler,
        TexelUpload::base_level_without_mipmaps(
            &[[0, 0, 0]; (NES_SCREEN_WIDTH * NES_SCREEN_HEIGHT) as usize],
        ),
    )?;

    let mut nes = None;

    'app: loop {
        context.window.glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Close => break 'app,
                WindowEvent::FileDrop(f) => {
                    if f.len() > 1 {
                        warn!(
                            "Expected one file, got {}. Defaulting to first file, {:?}",
                            f.len(),
                            f[0].to_str()
                        );
                    }
                    nes = Some(init_emulator(&f[0])?);
                }
                WindowEvent::Key(k, _, a, _) => {
                    if let Some(n) = &mut nes {
                        set_ctrl_state(k, a, n, &ctrl0, &ctrl1);
                    }
                }
                _ => (),
            }
        }

        if let Some(n) = &mut nes {
            let nes_buffer = n.next_frame();
            nes_fb_texture.upload_raw(TexelUpload::base_level_without_mipmaps(nes_buffer))?;
        }

        let render = context
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default(),
                |pipeline, mut shd_gate| {
                    let bound_tex = pipeline.bind_texture(&mut nes_fb_texture)?;

                    shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
                        iface.set(&uni.tex, bound_tex.binding());

                        rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                            tess_gate.render(&tess)
                        })
                    })
                },
            )
            .assume();

        if render.is_ok() {
            context.window.swap_buffers();
        } else {
            break 'app;
        }
    }

    Ok(())
}
