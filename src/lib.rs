#![feature(nll)]
#[macro_use]
extern crate nom;
extern crate bincode;
extern crate sdl2;
extern crate serde;
extern crate toml;
extern crate env_logger;
#[macro_use]
extern crate bitfield;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

pub mod apu;
pub mod config;
pub mod controller;
pub mod cpu;
pub mod cpu_const;
pub mod mapper;
pub mod mmu;
pub mod ppu;
pub mod rom;
pub mod state;

use state::State;
use cpu::Cpu;
use apu::Apu;
use ppu::Ppu;
use ppu::PpuRes;
use rom::Rom;
use mapper::Mapper;
use mmu::Mmu;
use std::cell::RefCell;
use std::rc::Rc;

pub struct NesEmulator {
    pub cpu: Cpu,
}

impl NesEmulator {
    pub fn new(rom: Rom) -> NesEmulator {
        println!("{:?}", rom);
        let mapper = Rc::new(RefCell::new(Mapper::from_rom(rom)));
        let cpu =
            Cpu::new(Mmu::new(Apu::new(), Ppu::new(mapper.clone()), mapper));
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
            screen_mode: self.cpu.mmu.mapper.borrow().get_mirroring(),
            chr_ram: self.cpu.mmu.mapper.borrow().rom.chr_ram.clone(),
            cpu_regs: self.cpu.regs.clone(),
            mapper: self.cpu.mmu.mapper.borrow().mem_type.clone(),
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

    pub fn next_frame(&mut self) -> Result<Box<[u8]>, String> {
        while !self.step()? {}
        Ok(self.cpu.mmu.ppu.get_buffer())
    }
}
