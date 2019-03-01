#[macro_use]
extern crate nom;
extern crate image;
extern crate sdl2;

pub mod apu;
pub mod cpu;
pub mod cpu_const;
pub mod mapper;
pub mod mmu;
pub mod ppu;
pub mod rom;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::TextureAccess;
use sdl2::render::Texture;
use sdl2::render::Canvas;
use sdl2::pixels::PixelFormatEnum;
use std::time::Duration;

use cpu::Cpu;
use apu::Apu;
use ppu::Ppu;
use rom::RomType;
use rom::Region;
use rom::parse_rom;
use std::fs::File;
use std::io::Read;
use mapper::Mapper;
use mmu::Mmu;
use mmu::Ram;
use std::cell::RefCell;
use std::rc::Rc;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

pub fn start_emulator(path_in: Option<String>) {
    let mut raw_bytes = Vec::new();
    let raw_rom =  match path_in {
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

    let mut mapper = Rc::new(RefCell::new(Mapper::from_rom(rom)));
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
            (SCREEN_WIDTH * 5) as u32,
            (SCREEN_HEIGHT * 5) as u32,
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
    loop {
        let cc = match cpu.step(false) {
            Ok(cc) => cc,
            Err(e) => {
                println!("{:?}", e);
                return;
            }
        };

        match cpu.mmu.ppu.emulate_cycles(cc) {
            Some(buff) => {
                cpu.proc_nmi();
                texture.update(None, &buff, SCREEN_WIDTH * 3).unwrap();
                canvas.clear();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
            }
            None => ()
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return,
                _ => {}
            }
        }
    }
}
