pub struct Ppu {
    pub regs: PRegisters
}

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
            }
        }
    }

    pub fn load(&self, val: u16) -> u8 {
        unimplemented!();
    }
    pub fn store(&self, addr: u16, val: u8) {
        unimplemented!();
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
    pub oamdma: u8
}
