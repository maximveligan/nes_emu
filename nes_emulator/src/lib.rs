pub mod apu;
pub mod controller;
pub mod mapper;
pub mod mmu;
pub mod ppu;
pub mod rom;
pub mod state;

use apu::Apu;
use cpu_6502::cpu::Cpu;
use mapper::Mapper;
use mmu::Mmu;
use ppu::Ppu;
use ppu::PpuRes;
use rom::Region;
use rom::Rom;
use state::State;
use std::cell::RefCell;
use std::rc::Rc;

const PAL_CPU_CLOCK_SPEED: usize = 1662607; // measured in hertz
const NTSC_CPU_CLOCK_SPEED: usize = 1789773; // measured in hertz

pub struct NesEmulator {
    pub cpu: Cpu<Mmu>,
}

pub enum PlayerController {
    One,
    Two,
}

impl NesEmulator {
    pub fn new(rom: Rom) -> NesEmulator {
        let cpu_clock_speed = match &rom.header.region {
            Region::PAL => PAL_CPU_CLOCK_SPEED,
            Region::NTSC => NTSC_CPU_CLOCK_SPEED,
        };
        let mapper = Rc::new(RefCell::new(Mapper::from_rom(rom)));
        let cpu = Cpu::new(Mmu::new(
            Apu::default(),
            Ppu::new(mapper.clone()),
            mapper,
            cpu_clock_speed,
        ));
        NesEmulator { cpu }
    }

    pub fn reset(&mut self) {
        self.cpu.mmu.mapper.borrow_mut().reset();
        self.cpu.mmu.ppu.reset();
        self.cpu.reset();
    }

    pub fn get_state(&self) -> State {
        State {
            ppu_state: self.cpu.mmu.ppu.get_state(),
            screen_mode: self.cpu.mmu.mapper.borrow().get_mirroring().clone(),
            chr_ram: self.cpu.mmu.mapper.borrow().rom.chr_ram.clone(),
            cpu_regs: self.cpu.regs.clone(),
            mapper: self.cpu.mmu.mapper.borrow().mem_type.clone(),
            ram: self.cpu.mmu.ram.clone(),
        }
    }

    pub fn load_state(&mut self, state: State) {
        self.cpu.mmu.ppu.set_state(state.ppu_state);
        self.cpu.mmu.mapper.borrow_mut().rom.header.screen = state.screen_mode;
        self.cpu.mmu.mapper.borrow_mut().rom.chr_ram = state.chr_ram;
        self.cpu.regs = state.cpu_regs;
        self.cpu.mmu.mapper.borrow_mut().mem_type = state.mapper;
        self.cpu.mmu.ram = state.ram;
    }

    pub fn step(&mut self) -> bool {
        let cc = self.cpu.step();
        match self.cpu.mmu.ppu.emulate_cycles(cc) {
            Some(r) => match r {
                PpuRes::Nmi => {
                    self.cpu.proc_nmi();
                    false
                }
                PpuRes::Draw => true,
            },
            None => false,
        }
    }

    pub fn next_frame(&mut self) -> &[u8] {
        while !self.step() {}
        self.cpu.mmu.ppu.get_buffer()
    }

    pub fn get_pixel_buffer(&self) -> &[u8] {
        self.cpu.mmu.ppu.get_buffer()
    }

    pub fn set_button(
        &mut self,
        button: crate::controller::Button,
        state: bool,
        controller: PlayerController,
    ) {
        match controller {
            PlayerController::One => {
                self.cpu.mmu.ctrl0.set_button_state(button, state)
            }
            PlayerController::Two => {
                self.cpu.mmu.ctrl0.set_button_state(button, state)
            }
        }
    }
}
