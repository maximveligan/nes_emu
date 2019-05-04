extern crate nes_emu;
extern crate sdl2;

use std::path::Path;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureAccess;
use sdl2::pixels::PixelFormatEnum;
use sdl2::joystick::HatState;
use nes_emu::controller::Controller;

use std::collections::HashMap;
use nes_emu::config::ButtonLayout;
use nes_emu::config::Config;
use nes_emu::controller::Button;
use nes_emu::rom::load_rom;
use nes_emu::NesEmulator;

use std::env;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

fn main() {
    if let Some(str_path) = env::args().nth(1) {
        let path = Path::new(&str_path);
        if path.is_file() {
            if let Some(os_stem) = path.file_stem() {
                if let Some(rom_stem) = os_stem.to_str() {
                    start_emulator(&str_path, rom_stem);
                } else {
                    println!("Failed to convert path to UTF-8");
                }
            } else {
                println!("Rom file name did not have .nes extension");
            }
        } else {
            println!("Did not recieve a rom file name!");
        }
    } else {
        println!("Did not recieve a rom path");
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
    SaveState(String),
    LoadState(String),
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
            } => Some(EventRes::SaveState(self.save_state())),
            Event::KeyDown {
                keycode: Some(Keycode::E),
                ..
            } => Some(EventRes::LoadState(self.load_state())),
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

    fn save_state(&mut self) -> String {
        match self.nes.get_state().save(&self.save_name) {
            Ok(size) => format!("Wrote {} bytes", size),
            Err(e) => format!("Error saving state {}", e),
        }
    }

    fn load_state(&mut self) -> String {
        match nes_emu::State::load(&self.save_name) {
            Ok((state, size_read)) => match self.nes.load_state(state) {
                Ok(()) => format!(
                    "Loaded state successfully, {} bytes read",
                    size_read
                ),
                Err(e) => format!("Emulator could not load state: {}", e),
            },
            Err(e) => format!(
                "Loading state from file failed: {}. Filename: {}",
                e, self.save_name
            ),
        }
    }

    fn hatstate_handler(ctrl: &mut Controller, state: HatState) {
        match state {
            HatState::Centered => {
                ctrl.set_button_state(Button::Up, false);
                ctrl.set_button_state(Button::Down, false);
                ctrl.set_button_state(Button::Left, false);
                ctrl.set_button_state(Button::Right, false);
            }
            HatState::Up => {
                ctrl.set_button_state(Button::Left, false);
                ctrl.set_button_state(Button::Right, false);
                ctrl.set_button_state(Button::Up, true);
            }
            HatState::Right => {
                ctrl.set_button_state(Button::Down, false);
                ctrl.set_button_state(Button::Up, false);
                ctrl.set_button_state(Button::Right, true);
            }
            HatState::Down => {
                ctrl.set_button_state(Button::Left, false);
                ctrl.set_button_state(Button::Right, false);
                ctrl.set_button_state(Button::Down, true);
            }
            HatState::Left => {
                ctrl.set_button_state(Button::Down, false);
                ctrl.set_button_state(Button::Up, false);
                ctrl.set_button_state(Button::Left, true);
            }
            HatState::LeftUp => {
                ctrl.set_button_state(Button::Left, true);
                ctrl.set_button_state(Button::Up, true);
            }
            HatState::LeftDown => {
                ctrl.set_button_state(Button::Left, true);
                ctrl.set_button_state(Button::Down, true);
            }
            HatState::RightUp => {
                ctrl.set_button_state(Button::Right, true);
                ctrl.set_button_state(Button::Up, true);
            }
            HatState::RightDown => {
                ctrl.set_button_state(Button::Right, true);
                ctrl.set_button_state(Button::Down, true);
            }
        }
    }
}

fn start_emulator(path_in: &str, rom_stem: &str) {
    let mut state_name = rom_stem.to_string();
    state_name.push_str(".sav");
    let config = match Config::load_config("./config.toml".to_string()) {
        Ok(config) => config,
        Err(e) => {
            println!("Error when loading config: {}", e);
            return;
        }
    };
    let sdl_ctrl0_map = match ButtonLayout::make_ctrl_map(&config.ctrl1_layout)
    {
        Ok(map) => map,
        Err(e) => {
            println!("Error occured while generating controller map 1 {}", e);
            return;
        }
    };

    let sdl_ctrl1_map = match ButtonLayout::make_ctrl_map(&config.ctrl2_layout)
    {
        Ok(map) => map,
        Err(e) => {
            println!("Error occured while generating controller map 2 {}", e);
            return;
        }
    };

    let screen_height = SCREEN_HEIGHT as u32
        - config.overscan.bottom as u32
        - config.overscan.top as u32;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let joystick_subsystem = sdl_context.joystick().expect("Should work");

    let joystick0 = match joystick_subsystem.open(0) {
        Ok(c) => {
            println!("Success: opened \"{}\"", c.name());
            Some(c)
        }
        Err(e) => {
            println!("failed: {:?}", e);
            None
        }
    }
    .expect("Couldn't open any joystick");

    let joystick1 = match joystick_subsystem.open(1) {
        Ok(c) => {
            println!("Success: opened \"{}\"", c.name());
            Some(c)
        }
        Err(e) => {
            println!("failed: {:?}", e);
            None
        }
    }
    .expect("Couldn't open any joystick");

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

    let rom = match load_rom(path_in) {
        Ok(rom) => rom,
        Err(e) => {
            println!("Error during rom parsing {}", e);
            return;
        }
    };

    let mut nes_frontend = NesFrontEnd {
        nes: NesEmulator::new(rom),
        pause: false,
        ctrl0: sdl_ctrl0_map,
        ctrl1: sdl_ctrl1_map,
        save_name: state_name,
    };

    loop {
        if !nes_frontend.pause {
            match nes_frontend.nes.next_frame() {
                Ok(framebuffer) => {
                    texture
                        .update(
                            None,
                            &framebuffer[config.overscan.top as usize
                                * 3
                                * SCREEN_WIDTH
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
                Err(e) => println!("The following error has occured: {}", e),
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::JoyHatMotion { state, which, .. } => match which {
                    0 => NesFrontEnd::hatstate_handler(
                        &mut nes_frontend.nes.cpu.mmu.ctrl1,
                        state,
                    ),
                    1 => NesFrontEnd::hatstate_handler(
                        &mut nes_frontend.nes.cpu.mmu.ctrl0,
                        state,
                    ),
                    _ => (),
                },

                Event::JoyButtonDown {
                    button_idx, which, ..
                } => match which {
                    0 => match button_idx {
                        1 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl1
                            .set_button_state(Button::B, true),
                        0 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl1
                            .set_button_state(Button::A, true),
                        6 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl1
                            .set_button_state(Button::Select, true),
                        7 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl1
                            .set_button_state(Button::Start, true),
                        _ => (),
                    },
                    1 => match button_idx {
                        0 => println!("{}", which),
                        1 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl0
                            .set_button_state(Button::B, true),
                        2 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl0
                            .set_button_state(Button::A, true),
                        3 => println!("X not bound"),
                        4 => println!("{}", nes_frontend.save_state(),),
                        5 => println!("{}", nes_frontend.load_state()),
                        6 => nes_frontend.nes.reset(),
                        7 => nes_frontend.switch_pause(),
                        8 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl0
                            .set_button_state(Button::Select, true),
                        9 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl0
                            .set_button_state(Button::Start, true),
                        b => panic!("Can't get here {}", b),
                    },
                    _ => (),
                },
                Event::JoyButtonUp {
                    button_idx, which, ..
                } => match which {
                    0 => match button_idx {
                        1 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl1
                            .set_button_state(Button::B, false),
                        0 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl1
                            .set_button_state(Button::A, false),
                        6 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl1
                            .set_button_state(Button::Select, false),
                        7 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl1
                            .set_button_state(Button::Start, false),
                        _ => (),
                    },
                    1 => match button_idx {
                        0 => println!("Y not bound"),
                        1 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl0
                            .set_button_state(Button::B, false),
                        2 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl0
                            .set_button_state(Button::A, false),
                        3 => (),
                        4 => (),
                        5 => (),
                        6 => (),
                        7 => (),
                        8 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl0
                            .set_button_state(Button::Select, false),
                        9 => nes_frontend
                            .nes
                            .cpu
                            .mmu
                            .ctrl0
                            .set_button_state(Button::Start, false),
                        _ => panic!("Can't get here"),
                    },
                    _ => (),
                },

                _ => (),
            }

            //Keyboard stuff
            if let Some(result) = nes_frontend.handle_event(event) {
                match result {
                    EventRes::SaveState(r) | EventRes::LoadState(r) => {
                        println!("{}", r)
                    }
                    EventRes::Quit => return,
                }
            }
        }
    }
}
