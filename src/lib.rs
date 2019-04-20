#![feature(nll)]
#[macro_use]
extern crate nom;
extern crate bincode;
extern crate sdl2;
extern crate serde;
extern crate toml;
#[macro_use]
extern crate bitfield;

pub mod apu;
pub mod config;
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

use serde::Serialize;
use serde::Deserialize;
use cpu::Cpu;
use apu::Apu;
use ppu::Ppu;
use ppu::PpuRes;
use rom::Rom;
use mapper::Mapper;
use mmu::Mmu;
use mmu::Ram;
use std::cell::RefCell;
use std::rc::Rc;
use pregisters::PRegisters;
use ppu::InternalRegs;
use cpu::Registers;
use rom::ScreenMode;
use mapper::MemType;

const SCREEN_SIZE: usize = 256 * 240 * 3;

// TODO: Figure out how to serialize arrays over 32 elements long
pub struct State {
    vram: [u8; 0x800],
    palette: [u8; 0x20],
    screen_mode: ScreenMode,
    chr_ram: Vec<u8>,
    ppu_regs: PRegisters,
    ppu_internal_regs: InternalRegs,
    cpu_regs: Registers,
    mapper: MemType,
}

impl State {
    pub fn save_state(&self) -> Result<(), std::io::Error> {
        unimplemented!("Writing the save state to a file isn't done yet");
    }

    // TODO: The error type should not be string, it should be whatever bincode
    // uses when it fails to parse a file
    pub fn load_state(path: String) -> Result<State, String> {
        unimplemented!("Loading state from a file isn't done yet");
    }
}

pub struct NesEmulator {
    pub cpu: Cpu,
    debug: bool,
}

impl NesEmulator {
    pub fn new(rom: Rom, debug: bool) -> NesEmulator {
        let mapper = Rc::new(RefCell::new(Mapper::from_rom(rom)));
        let cpu = Cpu::new(Mmu::new(
            Apu::new(),
            Ram::new(),
            Ppu::new(mapper.clone()),
            mapper,
        ));

        NesEmulator {
            cpu: cpu,
            debug: debug,
        }
    }

    pub fn get_state(&self) -> State {
        unimplemented!("Save states aren't done yet");
    }

    // TODO: The error type here needs to be well thought out, since there are
    // a lot of different ways to error
    pub fn load_state(state: State) -> Result<(), String> {
        unimplemented!("Loading in state isn't ready yet");
    }

    pub fn step(&mut self) -> Result<Option<&[u8; SCREEN_SIZE]>, String> {
        let cc = self.cpu.step(self.debug);
        match self.cpu.mmu.ppu.emulate_cycles(cc) {
            Some(r) => match r {
                PpuRes::Nmi => {
                    self.cpu.proc_nmi();
                    Ok(None)
                }
                PpuRes::Draw => Ok(Some(self.cpu.mmu.ppu.get_buffer())),
            },
            None => Ok(None),
        }
    }
}
