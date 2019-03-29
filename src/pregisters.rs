bitfield! {
    pub struct Ctrl(u8);
    pub nmi_on,     _ : 7;
    pub ppu_master, _ : 6;
    pub spr_size,   _ : 5;
    pub bg_tab,     _ : 4;
    pub sprite_tab, _ : 3;
    pub vert_inc,   _ : 2;
}

impl Ctrl {
    pub fn sprite_size(&self) -> u16 {
        if self.spr_size() {
            16
        } else {
            8
        }
    }

    pub fn nt_pt_addr(&self) -> u16 {
        self.bg_tab() as u16 * 0x1000
    }

    pub fn sprite_pt_addr(&self) -> u16 {
        self.sprite_tab() as u16 * 0x1000
    }

    pub fn vram_incr(&self) -> u8 {
        if self.vert_inc() {
            32
        } else {
            1
        }
    }

    pub fn base_nt_addr(&self) -> u16 {
        let tmp = self.0 & 0b11;
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

bitfield! {
    pub struct VramAddr(u16);
    pub u16, addr,           _: 13, 0;
    pub u8, h_byte, set_h_byte: 13, 8;
    pub u8, l_byte, set_l_byte:  7, 0;
}

impl VramAddr {
    pub fn add_offset(&mut self, offset: u8) {
        self.0 = self.0.wrapping_add(offset as u16);
    }
}

bitfield! {
    pub struct Status(u8);
    pub vblank, set_vblank: 7;
    pub sprite_0_hit, set_sprite_0_hit: 6;
    pub sprite_o_f, set_sprite_o_f: 5;
}

impl Status {
    pub fn load(&mut self) -> u8 {
        let tmp = self.0;
        self.set_vblank(false);
        tmp
    }
}

bitfield! {
    pub struct Mask(u8);
    pub is_grey_scale, _ : 0;
    pub left8_bg,      _ : 1;
    pub left8_sprite,  _ : 2;
    pub show_bg,       _ : 3;
    pub show_sprites,  _ : 4;
    pub emphasize_r,   _ : 5;
    pub emphasize_g,   _ : 6;
    pub emphasize_b,   _ : 7;
}

impl Mask {
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
            ctrl: Ctrl(0),
            mask: Mask(0),
            status: Status(0),
            oam_addr: 0,
            scroll: 0,
            addr: VramAddr(0),
        }
    }
}
