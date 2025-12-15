pub mod apu;
pub mod controller;
pub mod mapper;
pub mod mmu;
pub mod ppu;
pub mod rom;
pub mod state;

use crate::mmu::OAM_DATA;
use apu::Apu;
use cpu_6502::cpu::Cpu;
use cpu_6502::cpu_const::NMI_VEC;
use mapper::Mapper;
use mmu::Mmu;
use ppu::Ppu;
use rom::Region;
use rom::Rom;
use state::State;
use std::cell::RefCell;
use std::rc::Rc;

const _PAL_CPU_CLOCK_SPEED: usize = 1662607; // measured in hertz
const NTSC_CPU_CLOCK_SPEED: usize = 1789773; // measured in hertz

pub struct NesEmulator {
    pub cpu: Cpu,
    pub mmu: Mmu,
}

pub enum PlayerController {
    One,
    Two,
}

impl NesEmulator {
    pub fn new(rom: Rom) -> NesEmulator {
        let _ = match &rom.header.region {
            Region::PAL => {
                todo!("Unsupported ROM loaded, emulator doesn't support PAL!")
            }
            Region::NTSC => NTSC_CPU_CLOCK_SPEED,
        };
        let mapper = Rc::new(RefCell::new(Mapper::from_rom(rom)));
        let mut mmu =
            Mmu::new(Apu::default(), Ppu::new(mapper.clone()), mapper);
        let cpu = Cpu::new(&mut mmu);

        // Creating a new CPU also loads the interrupt vector, which increments
        // the cycle counter by 2, the ppu needs to catch up

        NesEmulator { mmu, cpu }
    }

    pub fn reset(&mut self) {
        self.mmu.mapper.borrow_mut().reset();
        self.cpu.reset(&mut self.mmu);
        self.mmu.ppu.reset();
    }

    pub fn get_state(&self) -> State {
        State {
            ppu_state: self.mmu.ppu.get_state(),
            screen_mode: self.mmu.mapper.borrow().get_mirroring().clone(),
            chr_ram: self.mmu.mapper.borrow().rom.chr_ram.clone(),
            cpu_regs: self.cpu.regs,
            mapper: self.mmu.mapper.borrow().mem_type.clone(),
            ram: self.mmu.ram.clone(),
        }
    }

    pub fn load_state(&mut self, state: State) {
        self.mmu.ppu.set_state(state.ppu_state);
        self.mmu.mapper.borrow_mut().rom.header.screen = state.screen_mode;
        self.mmu.mapper.borrow_mut().rom.chr_ram = state.chr_ram;
        self.cpu.regs = state.cpu_regs;
        self.mmu.mapper.borrow_mut().mem_type = state.mapper;
        self.mmu.ram = state.ram;
    }

    pub fn step(&mut self) -> bool {
        self.cpu.step(&mut self.mmu);

        if self.mmu.ppu.nmi_pending {
            self.cpu.intr_handler(&mut self.mmu, NMI_VEC);
            self.mmu.ppu.nmi_pending = false;
        } else if self.mmu.ppu.queued_nmi {
            self.cpu.step(&mut self.mmu);
            self.cpu.intr_handler(&mut self.mmu, NMI_VEC);
            self.mmu.ppu.nmi_pending = false;
            self.mmu.ppu.queued_nmi = false;
        }

        if let Some(val) = self.mmu.oam_dma {
            if self.cpu.dma(val, &mut self.mmu, OAM_DATA) {
                self.mmu.ppu.emulate_cycles(1);
            }
            self.mmu.oam_dma = None;
        }

        let draw_frame = self.mmu.ppu.frame_ready;
        if draw_frame {
            self.mmu.ppu.frame_ready = false;
            draw_frame
        } else {
            false
        }
    }

    pub fn next_frame(&mut self) -> &[u8] {
        while !self.step() {}
        self.mmu.ppu.get_buffer()
    }

    pub fn cur_frame(&self) -> &[u8] {
        self.mmu.ppu.get_buffer()
    }

    pub fn get_pixel_buffer(&self) -> &[u8] {
        self.mmu.ppu.get_buffer()
    }

    pub fn set_button(
        &mut self,
        button: crate::controller::Button,
        state: bool,
        controller: PlayerController,
    ) {
        match controller {
            PlayerController::One => {
                self.mmu.ctrl0.set_button_state(button, state)
            }
            PlayerController::Two => {
                self.mmu.ctrl0.set_button_state(button, state)
            }
        }
    }
}
