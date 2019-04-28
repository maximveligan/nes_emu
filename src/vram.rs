use mapper::Mapper;
use std::cell::RefCell;
use std::rc::Rc;
use rom::ScreenMode;
use rom::ScreenBank;

const VRAM_SIZE: usize = 0x800;
const PALETTE_RAM_I: u16 = 0x3F00;
const PALETTE_MIRROR_END: u16 = 0x3FFF;

const NT_0: u16 = 0x000;
const NT_0_END: u16 = 0x3FF;
const NT_1: u16 = 0x400;
const NT_1_END: u16 = 0x7FF;
const NT_2: u16 = 0x800;
const NT_2_END: u16 = 0xBFF;
const NT_3: u16 = 0xC00;
const NT_3_END: u16 = 0xFFF;

pub struct Vram {
    pub vram: Box<[u8]>,
    mapper: Rc<RefCell<Mapper>>,
    pub palette: [u8; 0x20],
}

impl Vram {
    pub fn new(mapper: Rc<RefCell<Mapper>>) -> Vram {
        Vram {
            vram: Box::new([0; VRAM_SIZE]),
            mapper: mapper,
            palette: [0; 0x20],
        }
    }

    pub fn reset(&mut self) {
        self.vram = Box::new([0; VRAM_SIZE]);
        self.palette = [0; 0x20];
    }

    pub fn ld8(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x1FFF => self.mapper.borrow_mut().ld_chr(addr),
            0x2000...0x3EFF => self.vram[self.nt_mirror(addr & 0xFFF)],
            PALETTE_RAM_I...PALETTE_MIRROR_END => {
                if addr == 0x3F10
                    || addr == 0x3F14
                    || addr == 0x3F18
                    || addr == 0x3F1C
                {
                    self.palette[(addr & 0x0F) as usize]
                } else {
                    self.palette[(addr & 0x1F) as usize]
                }
            }
            _ => panic!(),
        }
    }

    pub fn store(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x1FFF => {
                self.mapper.borrow_mut().store_chr(addr, val);
            }
            0x2000...0x3EFF => {
                self.vram[self.nt_mirror(addr & 0xFFF)] = val;
            }
            PALETTE_RAM_I...PALETTE_MIRROR_END => {
                if addr == 0x3F10
                    || addr == 0x3F14
                    || addr == 0x3F18
                    || addr == 0x3F1C
                {
                    self.palette[(addr & 0x0F) as usize] = val;
                } else {
                    self.palette[(addr & 0x1F) as usize] = val;
                }
            }
            _ => panic!(),
        }
    }

    // Helper function that resolves the nametable mirroring and returns an
    // index usable for VRAM array indexing
    fn nt_mirror(&self, addr: u16) -> usize {
        match self.mapper.borrow().get_mirroring() {
            ScreenMode::Horizontal => match addr {
                NT_0...NT_0_END => addr as usize,
                NT_1...NT_2_END => (addr - 0x400) as usize,
                NT_3...NT_3_END => (addr - 0x800) as usize,
                _ => panic!("Horizontal: addr outside of nt passed"),
            },
            ScreenMode::Vertical => match addr {
                NT_0...NT_1_END => addr as usize,
                NT_2...NT_3_END => (addr - 0x800) as usize,
                _ => panic!("Vertical: addr outside of nt passed"),
            },
            ScreenMode::OneScreenSwap(bank) => {
                let addr = addr & 0x3FF;
                match bank {
                    ScreenBank::Lower => addr as usize,
                    ScreenBank::Upper => addr as usize + 0x400,
                }
            }
            ScreenMode::FourScreen => {
                unimplemented!("Four Screen mode not supported yet")
            }
        }
    }
}
