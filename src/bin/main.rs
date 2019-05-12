#[macro_use]
extern crate log;
extern crate nes_emu;
extern crate sdl2;

#[macro_use]
extern crate failure;
use std::path::Path;
use failure::Error;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureAccess;
use sdl2::pixels::PixelFormatEnum;

use std::collections::HashMap;
use nes_emu::config::ButtonLayout;
use nes_emu::config::Config;
use nes_emu::controller::Button;
use nes_emu::rom::load_rom;
use nes_emu::NesEmulator;
use std::fs::File;
use std::io::Read;

use std::env;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

fn get_save_state_name<'a>(rom_path: &'a Path) -> Result<&'a str, Error> {
    if let Some(os_stem) = rom_path.file_stem() {
        if let Some(rom_stem) = os_stem.to_str() {
            Ok(rom_stem)
        } else {
            bail!("Failed to convert path to UTF-8");
        }
    } else {
        warn!("Rom file name did not have an extension");
        if let Some(rom_string) = rom_path.to_str() {
            Ok(rom_string)
        } else {
            bail!("Failed to convert path to UTF-8");
        }
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    if let Some(str_path) = env::args().nth(1) {
        let rom_path = Path::new(&str_path);
        if !rom_path.is_file() {
            bail!("Given path is not a file");
        } else {
            let save_state_name = get_save_state_name(rom_path)?;
            start_emulator(
                rom_path
                    .to_str()
                    .expect("Checked for this in get_save_state_name"),
                save_state_name,
            )
        }
    } else {
        bail!("No given path");
    }
}

struct NesFrontEnd {
    nes: NesEmulator,
    pause: bool,
    ctrl0: HashMap<Keycode, Button>,
    ctrl1: HashMap<Keycode, Button>,
    save_name: String,
}

enum EventRes {
    StateRes(String),
    Quit,
}

impl NesFrontEnd {
    fn handle_event(&mut self, event: sdl2::event::Event) -> Option<EventRes> {
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
                keycode: Some(Keycode::R),
                ..
            } => {
                self.nes.reset();
                None
            }
            Event::KeyDown {
                keycode: Some(Keycode::Q),
                ..
            } => {
                let state_res = match self.save_state() {
                    Ok(res) => res,
                    Err(e) => e.to_string(),
                };
                Some(EventRes::StateRes(state_res))
            }
            Event::KeyDown {
                keycode: Some(Keycode::E),
                ..
            } => {
                let state_res = match self.load_state() {
                    Ok(res) => res,
                    Err(e) => e.to_string(),
                };
                Some(EventRes::StateRes(state_res))
            }
            Event::KeyDown {
                keycode: Some(key), ..
            } => {
                self.set_ctrl0_state(key, true);
                self.set_ctrl1_state(key, true);
                None
            }
            Event::KeyUp {
                keycode: Some(key), ..
            } => {
                self.set_ctrl0_state(key, false);
                self.set_ctrl1_state(key, false);
                None
            }
            _ => None,
        }
    }

    fn set_ctrl0_state(&mut self, key: Keycode, state: bool) {
        if let Some(button) = self.ctrl0.get(&key) {
            self.nes.cpu.mmu.ctrl0.set_button_state(*button, state);
        }
    }

    fn set_ctrl1_state(&mut self, key: Keycode, state: bool) {
        if let Some(button) = self.ctrl1.get(&key) {
            self.nes.cpu.mmu.ctrl1.set_button_state(*button, state);
        }
    }

    fn switch_pause(&mut self) {
        self.pause = !self.pause;
    }

    fn save_state(&mut self) -> Result<String, Error> {
        let mut file = File::create(&self.save_name)?;
        self.nes.get_state().save(&mut file)?;
        Ok(format!("Successfully saved state: {}", &self.save_name))
    }

    fn load_state(&mut self) -> Result<String, Error> {
        let mut file = File::open(&self.save_name)?;
        let state = nes_emu::state::State::load(&mut file)?;
        self.nes.load_state(state);
        Ok("Loaded state successfully".to_string())
    }
}

fn start_emulator(path_in: &str, rom_stem: &str) -> Result<(), Error> {
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

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
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

    let mut nes_frontend = NesFrontEnd {
        nes: NesEmulator::new(rom),
        pause: false,
        ctrl0: ButtonLayout::make_ctrl_map(&config.ctrl1_layout)?,
        ctrl1: ButtonLayout::make_ctrl_map(&config.ctrl2_layout)?,
        save_name: rom_stem.to_string() + ".sav",
    };

    loop {
        if !nes_frontend.pause {
            let framebuffer = nes_frontend.nes.next_frame();
            texture
                .update(
                    None,
                    &framebuffer[config.overscan.top as usize * 3 * SCREEN_WIDTH
                        ..(SCREEN_WIDTH * SCREEN_HEIGHT
                            - config.overscan.bottom as usize)
                            * 3],
                    SCREEN_WIDTH * 3,
                )
                .unwrap();
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        for event in event_pump.poll_iter() {
            if let Some(result) = nes_frontend.handle_event(event) {
                match result {
                    EventRes::StateRes(r) => {
                        println!("{}", r)
                    }
                    EventRes::Quit => return Ok(()),
                }
            }
        }
    }
}
