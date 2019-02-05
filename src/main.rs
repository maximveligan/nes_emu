#[macro_use]
extern crate nom;

mod apu;
mod cpu;
mod cpu_const;
mod mapper;
mod mmu;
mod ppu;
mod rom;

use cpu::Cpu;
use rom::RomType;
use rom::parse_rom;
use std::fs::File;
use std::env;
use std::io::Read;
use mapper::Mapper;

fn main() {
    let mut raw_bytes = Vec::new();
    let raw_rom = match env::args().nth(1) {
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
            (_, rest) => {
                println!("{:?}", rest);
                rest
            }
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

    let mapper = Mapper::from_rom(rom);
    let mut cpu = Cpu::new(mapper);
    loop {
        match cpu.step() {
            Ok(()) => println!(""),
            Err(e) => println!("{:?}", e),
        }
        cpu.cycle_count = 0;
    }
}
