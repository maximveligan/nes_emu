#![feature(nll)]
#[macro_use]
extern crate nom;
extern crate bincode;
extern crate sdl2;
extern crate serde;
extern crate toml;
#[macro_use]
extern crate bitfield;
#[macro_use]
extern crate failure;

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

use std::io::Write;
use std::fs::File;
use std::io::Read;
use serde::Serialize;
use serde::Deserialize;
use cpu::Cpu;
use apu::Apu;
use ppu::Ppu;
use ppu::PpuRes;
use ppu::PpuState;
use rom::Rom;
use mapper::Mapper;
use mmu::Mmu;
use mmu::Ram;
use std::cell::RefCell;
use std::rc::Rc;
use cpu::Registers;
use rom::ScreenMode;
use mapper::MemType;
use failure::Error;

const SCREEN_SIZE: usize = 256 * 240 * 3;

#[derive(Serialize, Deserialize)]
pub struct State {
    ppu_state: PpuState,
    screen_mode: ScreenMode,
    chr_ram: Vec<u8>,
    cpu_regs: Registers,
    mapper: MemType,
    ram: Ram,
}

#[derive(Debug, Fail)]
pub enum StateFileError {
    #[fail(display = "Unable to parse state from file: {}", _0)]
    ParseError(std::boxed::Box<bincode::ErrorKind>),
    #[fail(display = "File error: {}", _0)]
    FileError(std::io::Error),
}

impl State {
    pub fn save_state(&self, save_name: &str) -> Result<usize, Error> {
        let bytes = bincode::serialize(&self)?;
        let mut buffer = File::create(save_name.to_string())?;
        Ok(buffer.write(&bytes)?)
    }

    pub fn load_state(path: &str) -> Result<(State, usize), Error> {
        let mut file = File::open(path.to_string())?;
        let mut buffer = Vec::new();
        let byte_read = file.read_to_end(&mut buffer)?;
        let state = bincode::deserialize(&buffer)?;
        Ok((state, byte_read))
    }
}

pub struct NesEmulator {
    pub cpu: Cpu,
}

impl NesEmulator {
    pub fn new(rom: Rom) -> NesEmulator {
        let mapper = Rc::new(RefCell::new(Mapper::from_rom(rom)));
        let cpu = Cpu::new(Mmu::new(
            Apu::new(),
            Ram::new(),
            Ppu::new(mapper.clone()),
            mapper,
        ));

        NesEmulator { cpu: cpu }
    }

    pub fn reset(&mut self) {
        self.cpu.mmu.mapper.borrow_mut().reset();
        self.cpu.mmu.ppu.reset();
        self.cpu.reset();
    }

    pub fn get_state(&self) -> State {
        State {
            ppu_state: self.cpu.mmu.ppu.get_state(),
            screen_mode: self.cpu.mmu.mapper.borrow().rom.header.screen,
            chr_ram: self.cpu.mmu.mapper.borrow().rom.chr_ram.clone(),
            cpu_regs: self.cpu.regs.clone(),
            mapper: self.cpu.mmu.mapper.borrow().mem_type,
            ram: self.cpu.mmu.ram.clone(),
        }
    }

    pub fn load_state(&mut self, state: State) -> Result<(), String> {
        self.cpu.mmu.ppu.set_state(state.ppu_state);
        self.cpu.mmu.mapper.borrow_mut().rom.header.screen = state.screen_mode;
        self.cpu.mmu.mapper.borrow_mut().rom.chr_ram = state.chr_ram;
        self.cpu.regs = state.cpu_regs;
        self.cpu.mmu.mapper.borrow_mut().mem_type = state.mapper;
        self.cpu.mmu.ram = state.ram;
        Ok(())
    }

    pub fn step(&mut self) -> Result<bool, String> {
        let cc = self.cpu.step();
        match self.cpu.mmu.ppu.emulate_cycles(cc) {
            Some(r) => match r {
                PpuRes::Nmi => {
                    self.cpu.proc_nmi();
                    Ok(false)
                }
                PpuRes::Draw => Ok(true),
            },
            None => Ok(false),
        }
    }

    pub fn next_frame(&mut self) -> Result<&[u8; SCREEN_SIZE], String> {
        while !self.step()? {}
        Ok(self.cpu.mmu.ppu.get_buffer())
    }
}
