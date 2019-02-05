const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;
const VRAM_SIZE: usize = 0x4000;

pub struct Ppu {
    pub regs: PRegisters,
    vram: Vram,

    // multiply by 3 to account for r g b
    screen_buff: [[u8; SCREEN_WIDTH * 3]; SCREEN_HEIGHT],
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
