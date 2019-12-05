#[macro_use]
extern crate log;
extern crate nes_emu;
extern crate sdl2;
extern crate sha3;
extern crate hex;
extern crate serde;

use std::path::Path;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureAccess;
use sdl2::pixels::PixelFormatEnum;

use std::collections::HashMap;
use config::ButtonLayout;
use config::Config;
use nes_emu::controller::Button;
use nes_emu::rom::load_rom;
use nes_emu::NesEmulator;
use std::fs::File;
use std::io::Read;
use std::env;
use sha3::Sha3_256;
use sha3::Digest;
use std::error::Error;

pub mod config;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

fn get_save_state_name<'a>(rom_path: &'a Path) -> Result<&'a str, Box<dyn Error>> {
    if let Some(os_stem) = rom_path.file_stem() {
        Ok(os_stem.to_str().ok_or("Failed to convert from utf-8")?)
    } else {
        warn!("Rom file name did not have an extension");
        Ok(rom_path.to_str().ok_or("Failed to convert path to UTF-8")?)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let str_path = env::args().nth(1).ok_or("No given path")?;
    let rom_path = Path::new(&str_path);
    if !rom_path.is_file() {
        Err("Given path is not a file")?
    } else {
        start_emulator(
            rom_path
                .to_str()
                .expect("Checked for this in get_save_state_name"),
            get_save_state_name(rom_path)?
        )
    }
}

struct NesFrontEnd<'a> {
    pause: bool,
    ctrl0: HashMap<Keycode, Button>,
    ctrl1: HashMap<Keycode, Button>,
    save_name: String,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    texture: sdl2::render::Texture<'a>,
}

enum EventRes {
    StateRes(String),
    Next,
    Quit,
    Hash,
}

impl NesFrontEnd<'_> {
    fn handle_event(
        &mut self,
        event: sdl2::event::Event,
        nes: &mut NesEmulator,
    ) -> Option<EventRes> {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => Some(EventRes::Quit),
            Event::KeyDown {
                keycode: Some(Keycode::P),
                ..
            } => {
                self.switch_pause();
                None
            }
            Event::KeyDown {
                keycode: Some(Keycode::N),
                ..
            } => {
                if self.pause {
                    Some(EventRes::Next)
                } else {
                    None
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::H),
                ..
            } => {
                if self.pause {
                    Some(EventRes::Hash)
                } else {
                    None
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::R),
                ..
            } => {
                nes.reset();
                None
            }
            Event::KeyDown {
                keycode: Some(Keycode::Q),
                ..
            } => {
                let state_res = match self.save_state(nes) {
                    Ok(res) => res,
                    Err(e) => e.to_string(),
                };
                Some(EventRes::StateRes(state_res))
            }
            Event::KeyDown {
                keycode: Some(Keycode::E),
                ..
            } => {
                let state_res = match self.load_state(nes) {
                    Ok(res) => res,
                    Err(e) => e.to_string(),
                };
                Some(EventRes::StateRes(state_res))
            }
            Event::KeyDown {
                keycode: Some(key), ..
            } => {
                self.set_ctrl0_state(key, true, nes);
                self.set_ctrl1_state(key, true, nes);
                None
            }
            Event::KeyUp {
                keycode: Some(key), ..
            } => {
                self.set_ctrl0_state(key, false, nes);
                self.set_ctrl1_state(key, false, nes);
                None
            }
            _ => None,
        }
    }

    fn set_ctrl0_state(
        &mut self,
        key: Keycode,
        state: bool,
        nes: &mut NesEmulator,
    ) {
        if let Some(button) = self.ctrl0.get(&key) {
            nes.cpu.mmu.ctrl0.set_button_state(*button, state);
        }
    }

    fn set_ctrl1_state(
        &mut self,
        key: Keycode,
        state: bool,
        nes: &mut NesEmulator,
    ) {
        if let Some(button) = self.ctrl1.get(&key) {
            nes.cpu.mmu.ctrl1.set_button_state(*button, state);
        }
    }

    fn switch_pause(&mut self) {
        self.pause = !self.pause;
    }

    fn save_state(&mut self, nes: &mut NesEmulator) -> Result<String, Box<dyn Error>> {
        let mut file = File::create(&self.save_name)?;
        nes.get_state().save(&mut file)?;
        Ok(format!("Successfully saved state: {}", &self.save_name))
    }

    fn load_state(&mut self, nes: &mut NesEmulator) -> Result<String, Box<dyn Error>> {
        let mut file = File::open(&self.save_name)?;
        let state = nes_emu::state::State::load(&mut file)?;
        nes.load_state(state);
        Ok("Loaded state successfully".to_string())
    }

    fn next_frame(&mut self, top: usize, bottom: usize, framebuffer: &[u8]) {
        self.texture
            .update(
                None,
                &framebuffer[top * 3 * SCREEN_WIDTH
                    ..(SCREEN_WIDTH * SCREEN_HEIGHT - bottom as usize) * 3],
                SCREEN_WIDTH * 3,
            )
            .unwrap();
        self.canvas.clear();
        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }
}

fn start_emulator(path_in: &str, rom_stem: &str) -> Result<(), Box<dyn Error>> {
    let mut frame_counter: usize = 0;
    let config = Config::load_config("./config.toml".to_string())?;

    let screen_height = SCREEN_HEIGHT as u32
        - config.overscan.bottom as u32
        - config.overscan.top as u32;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "Res",
            (SCREEN_WIDTH * config.pixel_scale) as u32,
            screen_height * config.pixel_scale as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    let canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture(
            PixelFormatEnum::RGB24,
            TextureAccess::Streaming,
            SCREEN_WIDTH as u32,
            screen_height,
        )
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut raw_bytes = Vec::new();
    let mut raw_rom = File::open(path_in)?;
    raw_rom.read_to_end(&mut raw_bytes)?;

    let rom = load_rom(&raw_bytes)?;

    let mut nes = NesEmulator::new(rom);

    let mut nes_frontend = NesFrontEnd {
        pause: false,
        ctrl0: ButtonLayout::make_ctrl_map(&config.ctrl1_layout)?,
        ctrl1: ButtonLayout::make_ctrl_map(&config.ctrl2_layout)?,
        save_name: rom_stem.to_string() + ".sav",
        canvas: canvas,
        texture: texture,
    };

    loop {
        if !nes_frontend.pause {
            nes_frontend.next_frame(
                config.overscan.top as usize,
                config.overscan.bottom as usize,
                nes.next_frame(),
            );
            frame_counter += 1;
        }

        for event in event_pump.poll_iter() {
            if let Some(result) = nes_frontend.handle_event(event, &mut nes) {
                match result {
                    EventRes::StateRes(r) => println!("{}", r),
                    EventRes::Quit => return Ok(()),
                    EventRes::Next => {
                        nes_frontend.next_frame(
                            config.overscan.top as usize,
                            config.overscan.bottom as usize,
                            nes.next_frame(),
                        );
                        frame_counter += 1;
                    }
                    EventRes::Hash => {
                        println!(
                            "Hash: {} Frame number: {}",
                            hex::encode(Sha3_256::digest(
                                nes.get_pixel_buffer()
                            )),
                            frame_counter
                        );
                    }
                }
            }
        }
    }
}
