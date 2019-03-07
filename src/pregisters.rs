const NMI: u8 = 0b1000_0000;
const PPU_MASTER: u8 = 0b0100_0000;
const SPR_SIZE: u8 = 0b0010_0000;
const BG_TAB: u8 = 0b0001_0000;
const SPRITE_TAB: u8 = 0b0000_1000;
const VERT_INC: u8 = 0b0000_0100;
const BASE_ADDR: u8 = 0b0000_0011;

pub struct Ctrl(u8);

impl Ctrl {
    fn new() -> Ctrl {
        Ctrl { 0: 0 }
    }

    pub fn nmi_on(&self) -> bool {
        self.0 & NMI != 0
    }

    fn ppu_master(&self) -> bool {
        self.0 & PPU_MASTER != 0
    }

    fn sprite_size(&self) -> u8 {
        if self.0 & SPR_SIZE != 0 {
            16
        } else {
            8
        }
    }

    fn table_addr(&self, val: u8) -> u16 {
        if self.0 & val != 0 {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn nt_pt_addr(&self) -> u16 {
        self.table_addr(BG_TAB)
    }

    fn sprite_pt_addr(&self) -> u16 {
        self.table_addr(SPRITE_TAB)
    }

    pub fn vram_incr(&self) -> u8 {
        if self.0 & VERT_INC != 0 {
            32
        } else {
            1
        }
    }

    pub fn base_nt_addr(&self) -> u16 {
        let tmp = self.0 & BASE_ADDR;
        match tmp {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => panic!("Other values shouldn't be possible"),
        }
    }

    pub fn store(&mut self, val: u8) {
        self.0 = val;
    }

    pub fn load(&self) -> u8 {
        self.0
    }
}

pub struct VramAddr(u16);

impl VramAddr {
    pub fn store(&mut self, val: u8, write: u8) {
        match write {
            0 => {
                self.0 |= ((val as u16) << 8);
            }
            1 => {
                self.0 |= (val as u16);
            }
            i => panic!("Write has to be either 1 or 2, got {}", i),
        }
    }

    pub fn read(&self) -> u16 {
        self.0 % 0x4000
    }

    pub fn add_offset(&mut self, offset: u8) {
        self.0 = self.0.wrapping_add(offset as u16);
    }
}

pub struct Status(u8);

impl Status {
    fn set_flag(&mut self, on: bool, flag: u8) {
        if on {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }

    pub fn set_vblank(&mut self, on: bool) {
        self.set_flag(on, 0b1000_0000);
    }

    fn get_vblank(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    fn set_sprite0(&mut self, on: bool) {
        self.set_flag(on, 0b0100_0000);
    }

    fn set_sprite_of(&mut self, on: bool) {
        self.set_flag(on, 0b0010_0000);
    }

    pub fn load(&mut self) -> u8 {
        let tmp = self.0;
        self.set_vblank(false);
        tmp
    }
}

pub struct Mask(u8);

impl Mask {
    fn get_flag(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    fn is_grey_scale(&self) -> bool {
        self.get_flag(0b0000_0001)
    }

    fn left8_bg(&self) -> bool {
        self.get_flag(0b0000_0010)
    }

    fn left8_sprite(&self) -> bool {
        self.get_flag(0b0000_0100)
    }

    pub fn show_bg(&self) -> bool {
        self.get_flag(0b0000_1000)
    }

    pub fn show_sprites(&self) -> bool {
        self.get_flag(0b0001_0000)
    }

    fn emphasize_r(&self) -> bool {
        self.get_flag(0b0010_0000)
    }
    fn emphasize_g(&self) -> bool {
        self.get_flag(0b0100_0000)
    }
    fn emphasize_b(&self) -> bool {
        self.get_flag(0b1000_0000)
    }
    pub fn store(&mut self, val: u8) {
        self.0 = val;
    }
}

pub struct PRegisters {
    pub ctrl: Ctrl,
    pub mask: Mask,
    pub status: Status,
    pub oam_addr: u8,
    pub scroll: u8,
    pub addr: VramAddr,
}

impl PRegisters {
    pub fn new() -> PRegisters {
        PRegisters {
            ctrl: Ctrl::new(),
            mask: Mask(0),
            status: Status(0),
            oam_addr: 0,
            scroll: 0,
            addr: VramAddr(0),
        }
    }
}
