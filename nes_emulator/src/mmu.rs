use crate::apu::Apu;
use crate::controller::Controller;
use crate::mapper::Mapper;
use crate::ppu::Ppu;
use cpu_6502::Memory;
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
use std::rc::Rc;

const WRAM_START: u16 = 0x0000;
const WRAM_END: u16 = 0x1FFF;
const PPU_START: u16 = 0x2000;
const PPU_END: u16 = 0x3FFF;
const ROM_START: u16 = 0x4020;
const ROM_END: u16 = 0xFFFF;

pub struct Mmu {
    pub ppu: Ppu,
    pub apu: Apu,
    pub ram: Ram,
    pub mapper: Rc<RefCell<Mapper>>,
    pub ctrl0: Controller,
    pub ctrl1: Controller,
    open_bus: u8,
}

impl Memory for Mmu {
    fn ld8(&mut self, address: u16) -> u8 {
        match address {
            WRAM_START..=WRAM_END => self.ram.load(address & 0x7FF),
            PPU_START..=PPU_END => {
                let ppu_reg = address & 0b111;

                match ppu_reg {
                    0 | 1 | 3 | 5 | 6 => self.open_bus,
                    2 => {
                        let (ppu_status, _) = self.ppu.ld(2);
                        self.open_bus = (ppu_status & 0b11100000)
                            | (self.open_bus & 0b00011111);
                        self.open_bus
                    }
                    4 => {
                        let (read_val, _) = self.ppu.ld(ppu_reg);
                        self.open_bus = read_val;
                        read_val & 0b11100011
                    }
                    7 => {
                        let (read_val, pal_read) = self.ppu.ld(ppu_reg);
                        if pal_read.expect("Can't get None here") {
                            (read_val & 0b00111111)
                                | (self.open_bus & 0b11000000)
                        } else {
                            self.open_bus = read_val;
                            read_val
                        }
                    }
                    _ => panic!("No other possible values here"),
                }
            }
            0x4015 => self.apu.load(address - 0x4000),
            0x4016 => self.ctrl0.ld8(),
            0x4017 => self.ctrl1.ld8(),
            0x4000..=0x4014 | 0x4018..=0x401F => {
                log::debug!("Tried to read from {:X}", address);
                0
            }
            ROM_START..=ROM_END => {
                let mapper = self.mapper.borrow();
                mapper.ld_prg(address)
            }
        }
    }

    fn ld16(&mut self, address: u16) -> u16 {
        let l_byte = self.ld8(address);
        let r_byte = self.ld8(address + 1);
        (r_byte as u16) << 8 | (l_byte as u16)
    }

    fn store(&mut self, address: u16, val: u8) {
        match address {
            WRAM_START..=WRAM_END => self.ram.store(address & 0x7FF, val),
            PPU_START..=PPU_END => self.ppu_store(address, val),
            0x4016 => self.ctrl_store(val),
            0x4000..=0x4017 => self.apu.store(address - 0x4000, val),
            0x4018..=0x401F => {
                log::debug!(
                    "Tried to write to {:X} with value {:X}",
                    address,
                    val
                );
            }
            ROM_START..=ROM_END => {
                self.mapper.borrow_mut().store_prg(address, val)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Ram(Box<[u8]>);

impl Default for Ram {
    fn default() -> Ram {
        Ram {
            0: Box::new([0; 0xFFF]),
        }
    }
}

impl Ram {
    fn load(&self, address: u16) -> u8 {
        self.0[address as usize]
    }

    fn store(&mut self, address: u16, val: u8) {
        self.0[address as usize] = val;
    }
}

impl Mmu {
    pub fn new(apu: Apu, ppu: Ppu, mapper: Rc<RefCell<Mapper>>) -> Mmu {
        Mmu {
            ppu,
            apu,
            mapper,
            ram: Ram::default(),
            ctrl0: Controller::default(),
            ctrl1: Controller::default(),
            open_bus: 0,
        }
    }

    fn ppu_store(&mut self, address: u16, val: u8) {
        self.open_bus = val;
        self.ppu.store((address - 0x2000) & 7, val);
    }

    fn ctrl_store(&mut self, val: u8) {
        self.ctrl0.store(val);
        self.ctrl1.store(val);
    }
}
