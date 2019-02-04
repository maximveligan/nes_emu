#[macro_use]
extern crate nom;

mod apu;
mod cpu;
mod cpu_const;
mod mmu;
mod ppu;
mod rom;

use cpu::Cpu;
use rom::parse_rom;
use std::fs::File;
use std::env;
use std::io::Read;
use nom::IResult;

fn main() {
    let mut raw_bytes = Vec::new();
    let rom = match env::args().nth(1) {
        Some(path) => match File::open(path) {
            Ok(mut rom) => {
                rom.read_to_end(&mut raw_bytes).expect("Something went wrong while reading the rom");
                parse_rom(&raw_bytes)
            }
            Err(err) => {
                println!("Unable to open file {}", err);
                return;
            }
        }

        _ => {
            println!("Didn't recieve a rom");
            return;
        }
    };

    match rom {
        Ok(out) => match out {
            (_, res) => {
                println!("{:?}", res.header);
                println!("{}", res.prg_rom.len());
                println!("{}", res.chr_rom.len());
            }
        }
        Err(err) => {println!("Parsing failed due to {}", err); return;}
    };

    let mut cpu = Cpu::new();
    match cpu.step() {
        Ok(()) => println!("success"),
        Err(e) => println!("{:?}", e),
    }
}
