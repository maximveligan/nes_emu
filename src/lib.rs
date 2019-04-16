#![feature(nll)]
#[macro_use]
extern crate nom;
extern crate sdl2;
extern crate serde;
extern crate toml;
#[macro_use]
extern crate bitfield;

pub mod apu;
pub mod controller;
pub mod cpu;
pub mod cpu_const;
pub mod mapper;
pub mod mmu;
pub mod ppu;
pub mod pregisters;
pub mod rom;
pub mod sprite;
pub mod vram;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureAccess;
use sdl2::pixels::PixelFormatEnum;

use serde::Serialize;
use serde::Deserialize;
use controller::Button;
use controller::Controller;
use cpu::Cpu;
use apu::Apu;
use ppu::Ppu;
use ppu::PpuRes;
use rom::RomType;
use rom::Region;
use rom::parse_rom;
use mapper::Mapper;
use mmu::Mmu;
use mmu::Ram;
use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    pixel_scale: usize,
    ctrl1_layout: ButtonLayout,
    ctrl2_layout: ButtonLayout,
}

#[derive(Serialize, Deserialize, Debug)]
struct ButtonLayout {
    left: String,
    up: String,
    down: String,
    right: String,
    a: String,
    b: String,
    start: String,
    select: String,
}

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

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

fn make_ctrl_map(layout: ButtonLayout) -> HashMap<String, Button> {
    let mut sdl_map = HashMap::new();
    sdl_map.insert(layout.left, Button::Left);
    sdl_map.insert(layout.right, Button::Right);
    sdl_map.insert(layout.down, Button::Down);
    sdl_map.insert(layout.up, Button::Up);
    sdl_map.insert(layout.a, Button::A);
    sdl_map.insert(layout.b, Button::B);
    sdl_map.insert(layout.start, Button::Start);
    sdl_map.insert(layout.select, Button::Select);
    sdl_map
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

pub fn start_emulator(path_in: Option<String>) {
    let config: Config = if Path::new("./config.toml").exists() {
        match File::open("./config.toml") {
            Ok(mut file) => {
                let mut config_string = String::new();
                match file.read_to_string(&mut config_string) {
                    Ok(_) => match toml::from_str(&config_string) {
                        Ok(config) => {
                            //println!("Loading config: {:#?}", config);
                            config
                        }
                        Err(err) => {
                            println!(
                                "Unable to parse config file correctly! {}",
                                err
                            );
                            return;
                        }
                    },
                    Err(err) => {
                        println!("Unable to read config file! {}", err);
                        return;
                    }
                }
            }
            Err(err) => {
                println!("Unable to open config file! {}", err);
                return;
            }
        }
    } else {
        let layout1 = ButtonLayout {
            left: "A".to_string(),
            up: "W".to_string(),
            down: "S".to_string(),
            right: "D".to_string(),
            a: "F".to_string(),
            b: "G".to_string(),
            start: "T".to_string(),
            select: "Y".to_string(),
        };

        let layout2 = ButtonLayout {
            left: "Left".to_string(),
            up: "Up".to_string(),
            down: "Down".to_string(),
            right: "Right".to_string(),
            a: "RShift".to_string(),
            b: "Enter".to_string(),
            start: "B".to_string(),
            select: "B".to_string(),
        };

        let config = Config {
            pixel_scale: 3,
            ctrl1_layout: layout1,
            ctrl2_layout: layout2,
        };

        println!("Did not find config file! Loading defaults {:?}", config);
        config
    };
    let sdl_map1 = make_ctrl_map(config.ctrl1_layout);
    let sdl_map2 = make_ctrl_map(config.ctrl2_layout);

    let mut raw_bytes = Vec::new();
    let raw_rom = match path_in {
        Some(path) => match File::open(path) {
            Ok(mut rom) => {
                rom.read_to_end(&mut raw_bytes)
                    .expect("Something went wrong while reading the rom");
                parse_rom(&raw_bytes)
            }
            Err(err) => {
                println!("Unable to open file {}", err);
                return;
            }
        },

        _ => {
            println!("Didn't recieve a rom");
            return;
        }
    };

    let rom = match raw_rom {
        Ok(out) => match out {
            (_, rest) => rest,
        },
        Err(err) => {
            println!("Parsing failed due to {}", err);
            return;
        }
    };

    match rom.header.rom_type {
        RomType::Nes2 => {
            println!("Unsupported rom type NES2.0!");
            return;
        }
        _ => (),
    }

    match rom.header.region {
        Region::PAL => {
            println!("Unsupported region PAL!");
            return;
        }
        _ => (),
    }

    let mapper = Rc::new(RefCell::new(Mapper::from_rom(rom)));
    let mut cpu = Cpu::new(Mmu::new(
        Apu::new(),
        Ram::new(),
        Ppu::new(mapper.clone()),
        mapper,
    ));

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

    //let mut cycle_counter: usize = 0;
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        let cc = match cpu.step(false) {
            Ok(cc) => cc,
            Err(e) => {
                println!("Got unsupported op {:X}", e);
                return;
            }
        };

        match cpu.mmu.ppu.emulate_cycles(cc) {
            Some(r) => match r {
                PpuRes::Nmi => cpu.proc_nmi(),
                PpuRes::Draw => {
                    texture
                        .update(
                            None,
                            cpu.mmu.ppu.get_buffer(),
                            SCREEN_WIDTH * 3,
                        )
                        .unwrap();
                    canvas.clear();
                    canvas.copy(&texture, None, None).unwrap();
                    canvas.present();
                }
            },
            None => (),
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    set_ctrl_state(key, &mut cpu.mmu.ctrl0, &sdl_map1, true);
                    set_ctrl_state(key, &mut cpu.mmu.ctrl1, &sdl_map2, true);
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    set_ctrl_state(key, &mut cpu.mmu.ctrl0, &sdl_map1, false);
                    set_ctrl_state(key, &mut cpu.mmu.ctrl1, &sdl_map2, false);
                }
                _ => {}
            }
        }
    }
}
