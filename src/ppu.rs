const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;
const VRAM_SIZE: usize = 0x4000;

const PAT_TAB_0: usize = 0x0000;
const PAT_TAB_1: usize = 0x1000;
const NAMETABLE_0: usize = 0x2000;
const NAMETABLE_1: usize = 0x2400;
const NAMETABLE_2: usize = 0x2800;
const NAMETABLE_3: usize = 0x2C00;
const NAME_TAB_MIRROR: usize = 0x3000;
const PALETTE_RAM_I: usize = 0x3F00;
const PALETTE_MIRROR: usize = 0x3F20;

const SPRITE_NUM: usize = 64;
const SPRITE_ATTR: usize = 4;

pub struct Ppu {
    pub regs: PRegisters,
    vram: Vram,

    // multiply by 3 to account for r g b
    screen_buff: [[u8; SCREEN_WIDTH * 3]; SCREEN_HEIGHT],
    oam: [[u8; SPRITE_ATTR]; SPRITE_NUM],
}

struct Vram([u8; VRAM_SIZE]);

impl Ppu {
    pub fn new() -> Ppu {
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
            vram: Vram([0; 0x4000]),
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
