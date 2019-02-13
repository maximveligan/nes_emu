use mapper::Mapper;
use std::cell::RefCell;
use std::rc::Rc;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;
const VRAM_SIZE: usize = 0x4000;

const PAT_TAB_START: u16 = 0x0000;
const PAT_TAB_END: u16 = 0x1FFF;
const NAMETABLE_0: u16 = 0x2000;
const NAMETABLE_1: u16 = 0x2400;
const NAMETABLE_2: u16 = 0x2800;
const NAMETABLE_3: u16 = 0x2C00;
const NAME_TAB_MIRROR: u16 = 0x3000;
const PALETTE_RAM_I: usize = 0x3F00;
const PALETTE_MIRROR: usize = 0x3F20;

static PALETTE: [u8; 192] = [
    0x80, 0x80, 0x80, 0x00, 0x3D, 0xA6, 0x00, 0x12, 0xB0, 0x44, 0x00, 0x96,
    0xA1, 0x00, 0x5E, 0xC7, 0x00, 0x28, 0xBA, 0x06, 0x00, 0x8C, 0x17, 0x00,
    0x5C, 0x2F, 0x00, 0x10, 0x45, 0x00, 0x05, 0x4A, 0x00, 0x00, 0x47, 0x2E,
    0x00, 0x41, 0x66, 0x00, 0x00, 0x00, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
    0xC7, 0xC7, 0xC7, 0x00, 0x77, 0xFF, 0x21, 0x55, 0xFF, 0x82, 0x37, 0xFA,
    0xEB, 0x2F, 0xB5, 0xFF, 0x29, 0x50, 0xFF, 0x22, 0x00, 0xD6, 0x32, 0x00,
    0xC4, 0x62, 0x00, 0x35, 0x80, 0x00, 0x05, 0x8F, 0x00, 0x00, 0x8A, 0x55,
    0x00, 0x99, 0xCC, 0x21, 0x21, 0x21, 0x09, 0x09, 0x09, 0x09, 0x09, 0x09,
    0xFF, 0xFF, 0xFF, 0x0F, 0xD7, 0xFF, 0x69, 0xA2, 0xFF, 0xD4, 0x80, 0xFF,
    0xFF, 0x45, 0xF3, 0xFF, 0x61, 0x8B, 0xFF, 0x88, 0x33, 0xFF, 0x9C, 0x12,
    0xFA, 0xBC, 0x20, 0x9F, 0xE3, 0x0E, 0x2B, 0xF0, 0x35, 0x0C, 0xF0, 0xA4,
    0x05, 0xFB, 0xFF, 0x5E, 0x5E, 0x5E, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D,
    0xFF, 0xFF, 0xFF, 0xA6, 0xFC, 0xFF, 0xB3, 0xEC, 0xFF, 0xDA, 0xAB, 0xEB,
    0xFF, 0xA8, 0xF9, 0xFF, 0xAB, 0xB3, 0xFF, 0xD2, 0xB0, 0xFF, 0xEF, 0xA6,
    0xFF, 0xF7, 0x9C, 0xD7, 0xE8, 0x95, 0xA6, 0xED, 0xAF, 0xA2, 0xF2, 0xDA,
    0x99, 0xFF, 0xFC, 0xDD, 0xDD, 0xDD, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11
];

const SPRITE_NUM: usize = 64;
const SPRITE_ATTR: usize = 4;

pub struct Ppu {
    pub regs: PRegisters,
    vram: Vram,

    // multiply by 3 to account for r g b
    screen_buff: [[u8; SCREEN_WIDTH * 3]; SCREEN_HEIGHT],
    oam: [[u8; SPRITE_ATTR]; SPRITE_NUM],
}

struct Vram {
    vram: [u8; VRAM_SIZE],
    mapper: Rc<RefCell<Mapper>>,
}

impl Vram {
    pub fn new(palette: &[u8], mapper: Rc<RefCell<Mapper>>) -> Vram {
        Vram {
            vram: [0; VRAM_SIZE],
            mapper: mapper,
        }
    }

    fn load_u8(&self, addr: u16) -> u8 {
        match addr {
            PAT_TAB_START...PAT_TAB_END => {
                let mut mapper = self.mapper.borrow_mut();
                mapper.load_chr(addr)
            }
//        NAMETABLE_0
//        NAMETABLE_1
//        NAMETABLE_2
//        NAMETABLE_3
//        NAME_TAB_MIRROR
//        PALETTE_RAM_I
//        PALETTE_MIRROR
            _ => panic!(),
        }
    }

    fn store(addr: u16, val: u8) {
    }
}

impl Ppu {
    pub fn new(mapper: Rc<RefCell<Mapper>>) -> Ppu {
        Ppu {
            regs: PRegisters {
                ppuctrl: 0,
                ppumask: 0,
                ppustatus: 0,
                oamaddr: 0,
                oamdata: 0,
                ppuscroll: 0,
                ppuaddr: 0,
                ppudata: 0,
                oamdma: 0,
            },
            vram: Vram::new(&PALETTE, mapper),
            screen_buff: [[0; SCREEN_WIDTH * 3]; SCREEN_HEIGHT],
            oam: [[0; SPRITE_ATTR]; SPRITE_NUM],
        }
    }

    //TODO: NOT ACCURATE, HERE FOR PLACE HOLDER
    pub fn load(&self, address: u16) -> u8 {
        match address {
            0 => self.regs.ppuctrl,
            1 => self.regs.ppumask,
            2 => self.regs.ppustatus,
            3 => 0,
            4 => panic!("Cannot read oamdata"),
            5 => 0,
            6 => 0,
            7 => self.regs.ppudata,
            _ => panic!("Somehow got to invalid register"),
        }
    }

    //TODO: NOT ACCURATE, HERE FOR PLACE HOLDER
    pub fn store(&mut self, address: u16, val: u8) {
        match address {
            0 => {
                self.regs.ppuctrl = val;
            }
            1 => {
                self.regs.ppumask = val;
            }
            2 => (),
            3 => {
                self.regs.oamaddr = val;
            }
            4 => {
                self.regs.oamdata = val;
            }
            5 => {
                self.regs.ppuscroll = val;
            }
            6 => {
                self.regs.ppuaddr = val;
            }
            7 => {
                self.regs.ppudata = val;
            }
            _ => panic!("Somehow got to invalid register"),
        }
    }
}

pub struct PRegisters {
    pub ppuctrl: u8,
    pub ppumask: u8,
    pub ppustatus: u8,
    pub oamaddr: u8,
    pub oamdata: u8,
    pub ppuscroll: u8,
    pub ppuaddr: u8,
    pub ppudata: u8,
    pub oamdma: u8,
}
