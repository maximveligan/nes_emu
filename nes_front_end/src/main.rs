use std::path::Path;

use config::ButtonLayout;
use config::Config;
use glfw::fail_on_errors;
use nes_emu::controller::Button;
use nes_emu::rom::load_rom;
use nes_emu::NesEmulator;
// use sha3::Digest;
// use sha3::Sha3_256;
// use std::convert::Infallible;
use std::mem;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::ffi::c_void;

use glfw::{Action, Context, Key, WindowEvent};
use gl;
use std::collections::HashMap;
use std::str;

use log::*;

pub mod config;

const NES_SCREEN_WIDTH: u32 = 256;
const NES_SCREEN_HEIGHT: u32 = 240;
// const WINDOW_SCALE: f32 = 3.;

const VS: &str = include_str!("shaders/vs.glsl");
const FS: &str = include_str!("shaders/fs.glsl");

// #[derive(UniformInterface)]
// struct ShaderInterface {
//     tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
// }

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

    let mut glfw = glfw::init(fail_on_errors!())?;
    let (mut window, events) = glfw.create_window(NES_SCREEN_WIDTH, NES_SCREEN_HEIGHT, "Res", glfw::WindowMode::Windowed).ok_or("Create window failed")?;
    let data: [u8; (NES_SCREEN_WIDTH * NES_SCREEN_HEIGHT * 3) as usize] = [0xFF; (NES_SCREEN_WIDTH * NES_SCREEN_HEIGHT * 3) as usize];
    window.make_current();
    window.set_key_polling(true);

    let mut texture = 0;

    unsafe {
        gl::load_with(|s| window.get_proc_address(s) as *const _);
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Viewport(0, 0, NES_SCREEN_WIDTH as i32, NES_SCREEN_HEIGHT as i32);
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);

        gl::ShaderSource(vertex_shader, 1, &(VS.as_bytes().as_ptr().cast()), &(VS.len().try_into().unwrap()));
        gl::CompileShader(vertex_shader);

        let mut success = 0;
        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(
                vertex_shader,
                1024,
                &mut log_len,
                v.as_mut_ptr().cast(),
            );
            v.set_len(log_len.try_into().unwrap());
            panic!("Vertex Compile Error: {}", String::from_utf8_lossy(&v));
        }

        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        assert_ne!(fragment_shader, 0);

        gl::ShaderSource(fragment_shader, 1, &(FS.as_bytes().as_ptr().cast()), &(FS.len().try_into().unwrap()));
        gl::CompileShader(fragment_shader);

        let mut success = 0;
        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetShaderInfoLog(
                fragment_shader,
                1024,
                &mut log_len,
                v.as_mut_ptr().cast(),
            );
            v.set_len(log_len.try_into().unwrap());
            panic!("Fragment Compile Error: {}", String::from_utf8_lossy(&v));
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        gl::UseProgram(shader_program);

        gl::GenTextures(1, &mut texture);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32,
                            NES_SCREEN_WIDTH as i32, NES_SCREEN_HEIGHT as i32, 0, gl::RGB,
                            gl::UNSIGNED_BYTE, data.as_ptr().cast());
        
        gl::Uniform1i(gl::GetUniformLocation(shader_program, "tex".as_ptr().cast()), 0);
    }

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    // let back_buffer = context.back_buffer().expect("back buffer");

    // let mut program = context
    //     .new_shader_program::<(), (), ShaderInterface>()
    //     .from_strings(VS, None, None, FS)
    //     .expect("program creation")
    //     .ignore_warnings();

    // let tess = context
    //     .new_tess()
    //     .set_render_vertex_nb(4)
    //     .set_mode(Mode::TriangleFan)
    //     .build()
    //     .unwrap();

    // let nearest_sampler = Sampler {
    //     min_filter: MinFilter::Nearest,
    //     mag_filter: MagFilter::Nearest,
    //     ..Sampler::default()
    // };

    // let mut nes_fb_texture: Texture<_, Dim2, NormRGB8UI> = Texture::new(
    //     &mut context,
    //     [NES_SCREEN_WIDTH, NES_SCREEN_HEIGHT],
    //     nearest_sampler,
    //     TexelUpload::base_level_without_mipmaps(
    //         &[[0, 0, 0]; (NES_SCREEN_WIDTH * NES_SCREEN_HEIGHT) as usize],
    //     ),
    // )?;

    let mut nes = None;
    nes = Some(init_emulator(Path::new("C:\\Users\\maxim\\Desktop\\Castlevania (E).nes"))?);

    while !window.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            println!("{:?}", event);
            match event {
                WindowEvent::Close => {
                    println!("break");
                }
                WindowEvent::FileDrop(f) => {
                    // if f.len() > 1 {
                    //     warn!(
                    //         "Expected one file, got {}. Defaulting to first file, {:?}",
                    //         f.len(),
                    //         f[0].to_str()
                    //     );
                    // }
                    // nes = Some(init_emulator(&f[0])?);
                }
                WindowEvent::Key(k, _, a, _) => {
                    if let Some(n) = &mut nes {
                        set_ctrl_state(k, a, n, &ctrl0, &ctrl1);
                    }
                }
                _ => println!("{:?}", event),
            }
        }

        if let Some(n) = &mut nes {
            let nes_buffer = n.next_frame();
            unsafe {
                gl::TexSubImage2D(
                    gl::TEXTURE_2D, 0, 0, 0, NES_SCREEN_WIDTH as i32,
                    NES_SCREEN_HEIGHT as i32, gl::RGB, gl::UNSIGNED_BYTE, nes_buffer.as_ptr().cast());
                // nes_fb_texture.upload_raw(TexelUpload::base_level_without_mipmaps(nes_buffer))?;
                gl::BindTexture(gl::TEXTURE_2D, texture)
            }
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        }

        window.swap_buffers();

        // let render = context
        //     .new_pipeline_gate()
        //     .pipeline(
        //         &back_buffer,
        //         &PipelineState::default(),
        //         |pipeline, mut shd_gate| {
        //             let bound_tex = pipeline.bind_texture(&mut nes_fb_texture)?;

        //             shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
        //                 iface.set(&uni.tex, bound_tex.binding());

        //                 rdr_gate.render(&RenderState::default(), |mut tess_gate| {
        //                     tess_gate.render(&tess)
        //                 })
        //             })
        //         },
        //     )
        //     .assume();

        // if render.is_ok() {
        //     context.window.swap_buffers();
        // } else {
        //     break;
        // }
    }

    Ok(())
}
