use mapper::Mapper;
use std::cell::RefCell;
use std::rc::Rc;
use pregisters::PRegisters;
use pregisters::VramAddr;
use sprite::Sprite;
use sprite::Priority;
use vram::*;

const PALETTE_RAM_I: u16 = 0x3F00;
const SPRITE_NUM: usize = 64;
const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

const AT_OFFSET: u16 = 0x03C0;

const CYC_PER_LINE: u16 = 340;

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
    0x99, 0xFF, 0xFC, 0xDD, 0xDD, 0xDD, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
];

#[derive(Copy, Clone)]
struct Rgb {
    data: [u8; 3],
}

#[derive(Debug)]
pub enum PpuRes {
    Nmi,
    Draw,
}

pub struct Ppu {
    pub regs: PRegisters,
    pub vram: Vram,

    // multiply by 3 to account for r g b
    screen_buff: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
    oam: [u8; 256],
    cc: u16,
    scanline: u16,
    // Internal registers
    write_latch: bool,
    ppudata_buff: u8,
    t_addr: VramAddr,
    fine_x: u8,
    // TODO: Scrolling is implemented solely through the address register
    // Fix this for more accurate emulation
    x_scroll: u16,
    y_scroll: u16,
}

impl Ppu {
    pub fn new(mapper: Rc<RefCell<Mapper>>) -> Ppu {
        Ppu {
            regs: PRegisters::new(),
            vram: Vram::new(mapper),
            screen_buff: [0; SCREEN_WIDTH * 3 * SCREEN_HEIGHT],
            oam: [0; 256],
            cc: 0,
            scanline: 0,
            write_latch: false,
            ppudata_buff: 0,
            x_scroll: 0,
            y_scroll: 0,
            fine_x: 0,
            t_addr: VramAddr(0),
        }
    }

    fn get_palette_color(&self, vram_index: u16) -> Rgb {
        let pal_index = self.vram.ld8(vram_index);
        Rgb {
            data: [
                PALETTE[pal_index as usize * 3],
                PALETTE[pal_index as usize * 3 + 1],
                PALETTE[pal_index as usize * 3 + 2],
            ],
        }
    }

    //Note: It is unclear what happens if we read from 4 outside of vblank
    pub fn ld(&mut self, address: u16) -> u8 {
        match address {
            0 => 0,
            1 => 0,
            2 => self.read_ppuctrl(),
            3 => 0,
            4 => self.oam[self.regs.oam_addr as usize],
            5 => 0,
            6 => 0,
            7 => self.read_ppudata(),
            _ => panic!("Somehow got to invalid register"),
        }
    }

    fn read_ppuctrl(&mut self) -> u8 {
        self.write_latch = false;
        let tmp = self.regs.status.load();
        self.regs.status.set_vblank(false);
        tmp
    }

    fn read_ppudata(&mut self) -> u8 {
        let addr = self.regs.addr.addr();
        let val = self.vram.ld8(addr);
        self.regs.addr.add_offset(self.regs.ctrl.vram_incr());
        if addr < 0x3F00 {
            let buff_val = self.ppudata_buff;
            self.ppudata_buff = val;
            buff_val
        } else {
            val
        }
    }

    pub fn store(&mut self, address: u16, val: u8) {
        match address {
            0 => self.write_ctrl(val),
            1 => self.regs.mask.store(val),
            2 => (),
            3 => self.write_oamaddr(val),
            4 => self.write_oamdata(val),
            5 => self.write_scroll(val),
            6 => self.write_ppuaddr(val),
            7 => self.write_ppudata(val),
            _ => panic!("Somehow got to invalid register"),
        }
    }

    fn write_ctrl(&mut self, val: u8) {
        //TODO: This is very hacky, all of this behaviour should be taken care
        //of by emulating the internal registers.
        self.regs.ctrl.store(val);
        self.x_scroll = (self.x_scroll & 0xFF) | self.regs.ctrl.x_scroll_base();
        self.y_scroll = (self.y_scroll & 0xFF) | self.regs.ctrl.y_scroll_base();
    }

    fn write_oamaddr(&mut self, val: u8) {
        self.regs.oam_addr = val;
    }

    fn write_oamdata(&mut self, val: u8) {
        self.oam[self.regs.oam_addr as usize] = val;
        self.regs.oam_addr = self.regs.oam_addr.wrapping_add(1);
    }

    //TODO: This is innacurate, go back and emulate the internal
    // registers properly
    fn write_scroll(&mut self, val: u8) {
        if self.write_latch {
            self.y_scroll = val as u16;
        } else {
            self.x_scroll = val as u16;
        }
        self.write_latch = !self.write_latch;
    }

    fn write_ppuaddr(&mut self, val: u8) {
        if self.write_latch {
            self.regs.addr.set_l_byte(val);
            // This is innacurate, go back and emulate the internal registers
            let x_base = (self.regs.addr.nt() & 0x01) as u16 * 256;
            let y_base = (self.regs.addr.nt() & 0x02) as u16 * 240;
            self.x_scroll = (self.x_scroll & 0xff) | x_base;
            self.y_scroll = (self.y_scroll & 0xff) | y_base;
        } else {
            self.regs.addr.set_h_byte(val);
        }
        self.write_latch = !self.write_latch
    }

    fn write_ppudata(&mut self, val: u8) {
        let addr = self.regs.addr.addr();
        self.vram.store(addr, val);
        self.regs.addr.add_offset(self.regs.ctrl.vram_incr());
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: Rgb) {
        self.screen_buff[(y * SCREEN_WIDTH + x) * 3..][..3]
            .copy_from_slice(&color.data);
    }

    fn get_sprites(&mut self) -> [Option<u8>; 8] {
        let mut sprite_count = 0;
        let mut sprites = [None; 8];
        for sprite_index in 0..SPRITE_NUM {
            if sprite_count >= 8 {
                self.regs.status.set_sprite_o_f(true);
                return sprites;
            }
            let raw_y = self.oam[sprite_index * 4];
            let y_pos = raw_y.wrapping_add(1) as u16;
            if y_pos <= self.scanline
                && y_pos + self.regs.ctrl.sprite_size() > self.scanline
            {
                sprites[sprite_count] = Some(sprite_index as u8);
                sprite_count += 1;
            }
        }
        sprites
    }

    fn sprite_pixel(
        &mut self,
        x: u8,
        sprites: [Option<u8>; 8],
        bg_opaque: bool,
    ) -> Option<(Rgb, Priority)> {
        for sprite_index in sprites.iter() {
            if let Some(index) = sprite_index {
                let s = Sprite::new(*index as usize, &self.oam);
                if !s.in_bounding_box(
                    x,
                    self.scanline as u8,
                    self.regs.mask.left8_bg(),
                ) {
                    continue;
                }

                let ctrl = &self.regs.ctrl;
                let (pt_tile_i, x_offset) =
                    s.get_tile_values(ctrl, x, self.scanline);
                let tile_color = self.get_tile(pt_tile_i, x_offset);

                if tile_color == 0 {
                    continue;
                }

                if *index == 0 && bg_opaque {
                    self.regs.status.set_sprite_0_hit(true);
                }

                let sprite_color =
                    (s.attributes.palette()) + 4 << 2 | tile_color;
                return Some((
                    self.get_palette_color(
                        PALETTE_RAM_I + (sprite_color as u16),
                    ),
                    Priority::from_attr(s.attributes.priority() as u8),
                ));
            } else {
                return None;
            }
        }
        None
    }

    fn get_attr_color(
        &self,
        x_tile: u16,
        y_tile: u16,
        pt_index: u8,
        base: u16,
    ) -> Rgb {
        let at_index = (x_tile / 4) + ((y_tile / 4) * 8);
        let at_byte = self.vram.ld8(base + AT_OFFSET + (at_index as u16));

        let at_color = match (x_tile % 4 < 2, y_tile % 4 < 2) {
            (false, false) => (at_byte >> 6) & 0b11,
            (false, true) => (at_byte >> 2) & 0b11,
            (true, false) => (at_byte >> 4) & 0b11,
            (true, true) => at_byte & 0b11,
        };

        let tile_color = (at_color * 4) | pt_index;
        self.get_palette_color(PALETTE_RAM_I + (tile_color as u16))
    }

    fn bg_pixel(&mut self, x: u16) -> Option<Rgb> {
        if x <= 8 && !self.regs.mask.left8_bg() {
            return None;
        }

        let x = x + self.x_scroll as u16;
        let y = self.scanline + self.y_scroll as u16;

        let x_tile = (x / 8) % 64;
        let y_tile = (y / 8) % 64;
        let base = match (x_tile >= 32, y_tile >= 30) {
            (false, false) => 0x2000,
            (true, false) => 0x2400,
            (false, true) => 0x2800,
            (true, true) => 0x2C00,
        };

        let x_tile = (x_tile % 32) as u16;
        let y_tile = (y_tile % 32) as u16;

        let x_pixel = (x % 8) as u8;
        let y_pixel = y % 8;

        let vram_index = x_tile + (y_tile * 32);
        // Get my tile number
        let tile_num = self.vram.ld8(base + vram_index);

        //
        // * 16 because each tile is 16 bytes long
        let pt_i = self.get_tile(
            self.regs.ctrl.nt_pt_addr() + y_pixel + (tile_num as u16 * 16),
            x_pixel,
        );
        match pt_i {
            0 => None,
            _ => Some(self.get_attr_color(x_tile, y_tile, pt_i, base)),
        }
    }

    fn get_tile(&self, pt_index: u16, x: u8) -> u8 {
        let left_byte = self.vram.ld8(pt_index);
        // Plus 8 to get the offset for the other sliver
        let right_byte = self.vram.ld8(pt_index + 8);
        let l_bit = ((left_byte << x & 0x80) != 0) as u8;
        let r_bit = ((right_byte << x & 0x80) != 0) as u8;
        r_bit << 1 | l_bit
    }

    fn pull_scanline(&mut self) {
        // U bg refers to universal background
        let u_bg_i = self.vram.ld8(0x3F00);
        let u_bg_color = Rgb {
            data: [
                PALETTE[u_bg_i as usize * 3],
                PALETTE[u_bg_i as usize * 3 + 1],
                PALETTE[u_bg_i as usize * 3 + 2],
            ],
        };

        let sprites = self.get_sprites();

        for x in 0u16..SCREEN_WIDTH as u16 {
            let mut bg_color = None;
            let mut sprite_color = None;
            if self.regs.mask.show_bg() {
                bg_color = self.bg_pixel(x);
            }

            if self.regs.mask.show_sprites() {
                sprite_color =
                    self.sprite_pixel(x as u8, sprites, bg_color.is_some());
            }

            let color = match (bg_color, sprite_color) {
                (None, None) => u_bg_color,
                (Some(bg_c), None) => bg_c,
                (None, Some((spr_c, _))) => spr_c,
                (Some(bg_c), Some((spr_c, p))) => match p {
                    Priority::Foreground => spr_c,
                    Priority::Background => bg_c,
                },
            };

            self.put_pixel(x as usize, self.scanline as usize, color);
        }
    }

    pub fn get_buffer(&self) -> &[u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3] {
        &self.screen_buff
    }

    fn step(&mut self) {
        self.cc += 1;
        if self.cc > CYC_PER_LINE {
            self.cc = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
            }
        }
    }

    fn tick(&mut self) -> Option<PpuRes> {
        let res = match self.scanline {
            0...239 => {
                if self.cc == 260 {
                    self.pull_scanline();
                }
                None
            }
            240 => {
                if self.cc == 0 {
                    Some(PpuRes::Draw)
                } else {
                    None
                }
            }
            241 => {
                if self.cc == 1 {
                    self.regs.status.set_vblank(true);
                    if self.regs.ctrl.nmi_on() {
                        Some(PpuRes::Nmi)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            242...260 => None,
            261 => {
                self.regs.status.set_vblank(false);
                self.regs.status.set_sprite_o_f(false);
                self.regs.status.set_sprite_0_hit(false);
                None
            }
            _ => panic!(
                "Scanline can't get here {}. Check emulate_cycles",
                self.scanline
            ),
        };
        self.step();
        res
    }

    pub fn emulate_cycles(&mut self, cyc_elapsed: u16) -> Option<PpuRes> {
        let mut ppu_res = None;
        for _ in 0..(cyc_elapsed * 3) {
            if let Some(res) = self.tick() {
                ppu_res = Some(res);
            }
        }
        ppu_res
    }
}
