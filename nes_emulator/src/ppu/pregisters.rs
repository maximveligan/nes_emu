use serde::Deserialize;
use serde::Serialize;

const X_SCROLL_MASK: u16 = 0b000010000011111;
const Y_SCROLL_MASK: u16 = 0b111101111100000;
const AT_BASE: u16 = 0x3C0;

bitfield! {
    #[derive(Serialize, Deserialize, Copy, Clone)]
    pub struct Ctrl(u8);
    pub nmi_on,     _ : 7;
    pub ppu_master, _ : 6;
    pub spr_size,   _ : 5;
    pub bg_tab,     _ : 4;
    pub sprite_tab, _ : 3;
    pub vert_inc,   _ : 2;
    pub nametable,  _ : 1, 0;
}

impl Ctrl {
    pub fn sprite_size(&self) -> u16 {
        if self.spr_size() {
            16
        } else {
            8
        }
    }

    //pt = pattern table
    //Addresses are always given either as 0x0000 or 0x1000
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

    pub fn store(&mut self, val: u8) {
        self.0 = val;
    }

    pub fn load(&self) -> u8 {
        self.0
    }
}

//yyy NN YYYYY XXXXX
//||| || ||||| +++++-- coarse X scroll
//||| || +++++-------- coarse Y scroll
//||| ++-------------- nametable select
//+++----------------- fine Y scroll
//Bit layouts follow the diagram above

bitfield! {
    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
    pub struct VramAddr(u16);
    pub u8, coarse_x, set_coarse_x: 4,  0;
    pub u8, coarse_y, set_coarse_y: 9,  5;
    pub u8, nt, set_nt:            11, 10;
    pub u8, fine_y, set_fine_y:    14, 12;
    pub u8, l_byte, set_l_byte:     7,  0;
    pub u8, h_byte, set_h_byte:    13,  8;
    pub u16, addr, _:              13,  0;
    pub u16, cur_tile, _:          11,  0;
    pub u8, _, set_last_bit:           14;
}

impl VramAddr {
    //The last addressable bit in the address is set to zero on high byte
    //writes
    pub fn set_h_byte_clear_bit(&mut self, val: u8) {
        self.set_h_byte(val);
        self.set_last_bit(false);
    }

    pub fn add_offset(&mut self, offset: u8) {
        self.0 = self.0.wrapping_add(offset as u16);
    }

    //Strip the fine y out, and add the base name table value of 0x2000
    pub fn nt_addr(&self) -> u16 {
        0x2000 | self.cur_tile()
    }

    pub fn at_addr(&self) -> u16 {
        //The 10 bit shift to the left OR'ed with self.nt() gives the base
        //attribute table offset. Each byte controls a 4x4 tile (32x32 pixel)
        //size, hence the division by 4. 32/4 = 8, therefore we need to
        // multiply coarse_y by 8, functionally equivalent to << 3.
        // Thus, the attribute table address is: 0010 nn bbbb yyy xxx
        //b = at_base bit
        //n = nametable bit (or at_base bit depending on what is 0 and 1)
        //y = coarse y / 4
        //x = coarse_x / 4
        let at_index = ((self.coarse_y() as u16) / 4) << 3
            | ((self.coarse_x() as u16) / 4);
        let nt_offset = (self.nt() as u16) << 10;
        0x2000 | AT_BASE | nt_offset | at_index
    }

    // Both of these scroll functions were taken from
    // https://wiki.nesdev.com/w/index.php/PPU_scrolling#Register_controls
    // Increments coarse_x by one at the end of each tile (every 8 cycles)
    // with correct wrap around logic to go to the next name table
    pub fn scroll_x(&mut self) {
        if self.coarse_x() == 31 {
            self.set_coarse_x(0);
            self.0 ^= 0x0400
        } else {
            let tmp = self.coarse_x();
            self.set_coarse_x(tmp + 1);
        }
    }

    // Increments the coarse_y by one at the end of each scanline before hblank
    // with correct wraparound logic to go to the next nametable.
    pub fn scroll_y(&mut self) {
        let fine_y = self.fine_y();
        if fine_y < 7 {
            self.set_fine_y(fine_y + 1);
        } else {
            self.set_fine_y(0);
            let coarse_y = self.coarse_y();

            if coarse_y == 29 {
                self.set_coarse_y(0);
                self.0 ^= 0x0800
            } else if coarse_y == 31 {
                self.set_coarse_y(0);
            } else {
                self.set_coarse_y(coarse_y + 1);
            }
        }
    }

    //Used to copy the tmp addr x values back into our v_addr register once a
    //scanline ends
    pub fn pull_x(&mut self, addr: VramAddr) {
        self.0 = (self.0 & !X_SCROLL_MASK) | (addr.0 & X_SCROLL_MASK);
    }

    //Used to copy the tmp addr y values back into v_addr once a frame
    //is drawn
    pub fn pull_y(&mut self, addr: VramAddr) {
        self.0 = (self.0 & !Y_SCROLL_MASK) | (addr.0 & Y_SCROLL_MASK);
    }
}

bitfield! {
    #[derive(Serialize, Deserialize, Copy, Clone)]
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
    #[derive(Serialize, Deserialize, Copy, Clone)]
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

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct PRegisters {
    pub ctrl: Ctrl,
    pub mask: Mask,
    pub status: Status,
    pub oam_addr: u8,
    pub scroll: u8,
    pub addr: VramAddr,
}

impl Default for PRegisters {
    fn default() -> PRegisters {
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
