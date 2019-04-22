extern crate nes_emu;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureAccess;
use sdl2::pixels::PixelFormatEnum;

use std::collections::HashMap;
use nes_emu::config::ButtonLayout;
use nes_emu::config::Config;
use nes_emu::controller::Button;
use nes_emu::controller::Controller;
use nes_emu::rom::load_rom;
use nes_emu::NesEmulator;
use nes_emu::State;

use std::env;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

fn main() {
    match env::args().nth(1) {
        Some(path) => start_emulator(path),
        None => println!("Did not recieve a rom path"),
    }
}

fn keycode_to_str(keycode: Keycode) -> Option<String> {
    match keycode {
        Keycode::A => Some("A".to_string()),
        Keycode::B => Some("B".to_string()),
        Keycode::C => Some("C".to_string()),
        Keycode::D => Some("D".to_string()),
        Keycode::E => Some("E".to_string()),
        Keycode::F => Some("F".to_string()),
        Keycode::G => Some("G".to_string()),
        Keycode::H => Some("H".to_string()),
        Keycode::I => Some("I".to_string()),
        Keycode::J => Some("J".to_string()),
        Keycode::K => Some("K".to_string()),
        Keycode::L => Some("L".to_string()),
        Keycode::M => Some("M".to_string()),
        Keycode::N => Some("N".to_string()),
        Keycode::O => Some("O".to_string()),
        Keycode::P => Some("P".to_string()),
        Keycode::Q => Some("Q".to_string()),
        Keycode::R => Some("R".to_string()),
        Keycode::S => Some("S".to_string()),
        Keycode::T => Some("T".to_string()),
        Keycode::U => Some("U".to_string()),
        Keycode::V => Some("V".to_string()),
        Keycode::W => Some("W".to_string()),
        Keycode::X => Some("X".to_string()),
        Keycode::Y => Some("Y".to_string()),
        Keycode::Z => Some("Z".to_string()),
        Keycode::Left => Some("Left".to_string()),
        Keycode::Up => Some("Up".to_string()),
        Keycode::Right => Some("Right".to_string()),
        Keycode::Down => Some("Down".to_string()),
        Keycode::LShift => Some("LShift".to_string()),
        Keycode::RShift => Some("RShift".to_string()),
        Keycode::Return => Some("Enter".to_string()),
        _ => None,
    }
}

fn set_ctrl_state(
    key: Keycode,
    ctrl: &mut Controller,
    button_map: &HashMap<String, Button>,
    state: bool,
) {
    match keycode_to_str(key) {
        Some(key) => match button_map.get(&key) {
            Some(button) => ctrl.set_button_state(*button, state),
            None => (),
        },
        None => (),
    }
}

fn start_emulator(path_in: String) {
    let config = match Config::load_config("./config.toml".to_string()) {
        Ok(config) => config,
        Err(_e) => {
            // println!("Error when loading config: {}", e);
            return;
        }
    };
    let mut pause = false;
    let sdl_ctrl1_map = ButtonLayout::make_ctrl_map(&config.ctrl1_layout);
    let sdl_ctrl2_map = ButtonLayout::make_ctrl_map(&config.ctrl2_layout);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "Nust",
            (SCREEN_WIDTH * config.pixel_scale) as u32,
            (SCREEN_HEIGHT * config.pixel_scale) as u32,
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
            SCREEN_HEIGHT as u32,
        )
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let rom = match load_rom(path_in) {
        Ok(rom) => rom,
        Err(_e) => {
            // TODO: This guy needs a proper formatter to print it
            // println!("Error during rom parsing {}", e),
            return;
        }
    };

    let mut nes = NesEmulator::new(rom, false);

    loop {
        if !pause {
            match nes.next_frame() {
                Ok(framebuffer) => {
                    texture
                        .update(None, framebuffer, SCREEN_WIDTH * 3)
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
                Event::Quit { .. } => return,

                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return,
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    pause = !pause;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    nes.reset();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => {
                    let state = nes.get_state();
                    match state.save_state() {
                        Ok(()) => (),
                        Err(e) => panic!(),
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => match nes_emu::State::load_state("save.bin".to_string()) {
                    Ok(state) => match nes.load_state(state) {
                        Ok(()) => (),
                        Err(e) => panic!("emulator didn't like the load state"),
                    },
                    Err(e) => panic!("loading state failed"),
                },
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    set_ctrl_state(
                        key,
                        &mut nes.cpu.mmu.ctrl0,
                        &sdl_ctrl1_map,
                        true,
                    );
                    set_ctrl_state(
                        key,
                        &mut nes.cpu.mmu.ctrl1,
                        &sdl_ctrl2_map,
                        true,
                    );
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    set_ctrl_state(
                        key,
                        &mut nes.cpu.mmu.ctrl0,
                        &sdl_ctrl1_map,
                        false,
                    );
                    set_ctrl_state(
                        key,
                        &mut nes.cpu.mmu.ctrl1,
                        &sdl_ctrl2_map,
                        false,
                    );
                }
                _ => {}
            }
        }
    }
}
