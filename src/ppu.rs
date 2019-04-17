use mapper::Mapper;
use std::cell::RefCell;
use std::rc::Rc;
use pregisters::PRegisters;
use pregisters::VramAddr;
use pregisters::Ctrl;
use sprite::Sprite;
use sprite::Priority;
use vram::*;

const SPRITE_NUM: usize = 64;
const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;
const PRERENDER: u16 = 261;

pub const PALETTE: [u8; 192] = [
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

//TODO: Add loadable palettes. This palette here is more accurate to the og NES
//colors but in my opinion looks works.
//pub const PALETTE: [u8; 192] = [
//    84, 84, 84, 0, 30, 116, 8, 16, 144, 48, 0, 136, 68, 0, 100, 92, 0, 48, 84,
//    4, 0, 60, 24, 0, 32, 42, 0, 8, 58, 0, 0, 64, 0, 0, 60, 0, 0, 50, 60, 0, 0,
//    0, 0, 0, 0, 0, 0, 0, 152, 150, 152, 8, 76, 196, 48, 50, 236, 92, 30, 228,
//    136, 20, 176, 160, 20, 100, 152, 34, 32, 120, 60, 0, 84, 90, 0, 40, 114, 0,
//    8, 124, 0, 0, 118, 40, 0, 102, 120, 0, 0, 0, 0, 0, 0, 0, 0, 0, 236, 238,
//    236, 76, 154, 236, 120, 124, 236, 176, 98, 236, 228, 84, 236, 236, 88, 180,
//    236, 106, 100, 212, 136, 32, 160, 170, 0, 116, 196, 0, 76, 208, 32, 56,
//    204, 108, 56, 180, 204, 60, 60, 60, 0, 0, 0, 0, 0, 0, 236, 238, 236, 168,
//    204, 236, 188, 188, 236, 212, 178, 236, 236, 174, 236, 236, 174, 212, 236,
//    180, 176, 228, 196, 144, 204, 210, 120, 180, 222, 120, 168, 226, 144, 152,
//    226, 180, 160, 214, 228, 160, 162, 160, 0, 0, 0, 0, 0, 0,
//];

#[derive(Copy, Clone)]
struct Rgb {
    data: [u8; 3],
}

#[derive(Debug)]
pub enum PpuRes {
    Nmi,
    Draw,
}

#[derive(Copy, Clone)]
//These are 1 bit latches, so I'm using booleans to store them
struct AtLatch {
    low_b: bool,
    high_b: bool,
}

struct AtShift {
    low_tile: u8,
    high_tile: u8,
}

#[derive(Copy, Clone)]
struct BgLatch {
    low_tile: u8,
    high_tile: u8,
}

struct BgShift {
    low_tile: u16,
    high_tile: u16,
}

struct InternalRegs {
    at_latch: AtLatch,
    at_shift: AtShift,
    bg_latch: BgLatch,
    bg_shift: BgShift,
}

impl InternalRegs {
    fn new() -> InternalRegs {
        InternalRegs {
            at_latch: AtLatch {
                low_b: false,
                high_b: false,
            },
            at_shift: AtShift {
                low_tile: 0,
                high_tile: 0,
            },
            bg_latch: BgLatch {
                low_tile: 0,
                high_tile: 0,
            },
            bg_shift: BgShift {
                low_tile: 0,
                high_tile: 0,
            },
        }
    }

    fn reload(&mut self, at_entry: u8) {
        self.bg_shift.low_tile =
            (self.bg_shift.low_tile & 0xFF00) | self.bg_latch.low_tile as u16;
        self.bg_shift.high_tile =
            (self.bg_shift.high_tile & 0xFF00) | self.bg_latch.high_tile as u16;
        self.at_latch.low_b = (at_entry & 1) == 1;
        self.at_latch.high_b = ((at_entry >> 1) & 1) == 1;
    }

    fn shift(&mut self) {
        self.at_shift.low_tile =
            (self.at_shift.low_tile << 1) | self.at_latch.low_b as u8;
        self.at_shift.high_tile =
            (self.at_shift.high_tile << 1) | self.at_latch.high_b as u8;
        self.bg_shift.low_tile <<= 1;
        self.bg_shift.high_tile <<= 1;
    }
}

pub struct Ppu {
    pub regs: PRegisters,
    vram: Vram,
    // multiply by 3 to account for r g b
    screen_buff: [u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3],
    oam: [u8; 256],
    tmp_oam: [Option<Sprite>; 8],
    main_oam: [Option<Sprite>; 8],
    cc: u16,
    scanline: u16,
    // Internal registers
    //
    // Write latch is another 1 bit latch that stores data on which write we
    // are on
    write_latch: bool,
    // Data buff is used for the 1 byte delay when reading data from port 7
    ppudata_buff: u8,
    // Temporary address used to reload x and y scroll values and also for
    // intermediary storage for writes to port 5
    t_addr: VramAddr,
    // Fine x scrolling is not part of the 16 bit internal v_addr, so the NES
    // has a separate fine x register for inner tile scrolling
    fine_x: u8,
    // Used to force an nmi when in vblank and a write to CTRL enables NMI
    trip_nmi: bool,
    // Used to correctly emulate the race condition when reading from STATUS
    // disables NMI for that frame
    vblank_off: bool,
    // Contains the attribute table data for the NEXT tile
    at_entry: u8,
    // Contains the shift and latch registers the NES uses for rendering
    internal_regs: InternalRegs,
}

impl Ppu {
    pub fn new(mapper: Rc<RefCell<Mapper>>) -> Ppu {
        Ppu {
            trip_nmi: false,
            vblank_off: false,
            regs: PRegisters::new(),
            vram: Vram::new(mapper),
            screen_buff: [0; SCREEN_WIDTH * 3 * SCREEN_HEIGHT],
            oam: [0; 256],
            tmp_oam: [None; 8],
            main_oam: [None; 8],
            cc: 0,
            scanline: 0,
            write_latch: false,
            ppudata_buff: 0,
            fine_x: 0,
            t_addr: VramAddr(0),
            at_entry: 0,
            internal_regs: InternalRegs::new(),
        }
    }

    pub fn pull_nt(&self, offset: u16) {
        for index in offset..(offset + 0x3C0) {
            println!("{:02X}", self.vram.ld8(index as u16));
        }
    }

    pub fn pull_at(&self, offset: u16) {
        for index in offset..(offset + 0xC00) {
            println!("{:02X}", self.vram.ld8(index as u16));
        }
    }

    fn get_palette_color(&self, vram_offset: u8) -> Rgb {
        let pal_index = self.vram.ld8(0x3F00 + vram_offset as u16);
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
            2 => self.read_ppustatus(),
            3 => 0,
            4 => self.oam[self.regs.oam_addr as usize],
            5 => 0,
            6 => 0,
            7 => self.read_ppudata(),
            _ => panic!("Somehow got to invalid register"),
        }
    }

    fn read_ppustatus(&mut self) -> u8 {
        self.write_latch = false;
        let tmp = self.regs.status.load();
        self.regs.status.set_vblank(false);
        self.vblank_off = true;
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
            0 => {
                self.write_ctrl(val);
            }
            1 => {
                self.regs.mask.store(val);
            }
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
        let ctrl = Ctrl(val);
        if !self.regs.ctrl.nmi_on() && ctrl.nmi_on() {
            self.trip_nmi = true;
        }

        self.regs.ctrl = ctrl;
        self.t_addr.set_nt(self.regs.ctrl.nametable());
    }

    fn write_oamaddr(&mut self, val: u8) {
        self.regs.oam_addr = val;
    }

    fn write_oamdata(&mut self, val: u8) {
        self.oam[self.regs.oam_addr as usize] = val;
        self.regs.oam_addr = self.regs.oam_addr.wrapping_add(1);
    }

    fn write_scroll(&mut self, val: u8) {
        if self.write_latch {
            self.t_addr.set_fine_y(val);
            self.t_addr.set_coarse_y(val >> 3);
        } else {
            self.fine_x = val & 0b111;
            self.t_addr.set_coarse_x(val >> 3)
        }
        self.write_latch = !self.write_latch;
    }

    fn write_ppuaddr(&mut self, val: u8) {
        if self.write_latch {
            self.t_addr.set_l_byte(val);
            self.regs.addr = self.t_addr.clone();
        } else {
            self.t_addr.set_h_byte_clear_bit(val);
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

    fn step_sprites(&mut self) {
        match self.cc {
            1 => {
                self.tmp_oam = [None; 8];
                if self.is_prerender() {
                    self.regs.status.set_sprite_o_f(false);
                    self.regs.status.set_sprite_0_hit(false);
                }
            }
            257 => self.get_sprites(),
            321 => {
                //TODO: This is more or less a hack for going from secondary
                // oam to primary oam This needs to be rewritten
                // to more accurately emulate sprite evaluation
                self.main_oam = self.tmp_oam.clone();
            }
            _ => (),
        }
    }

    fn get_sprites(&mut self) {
        self.tmp_oam = [None; 8];
        let mut sprite_count = 0;
        for sprite_index in 0..SPRITE_NUM {
            if sprite_count >= 8 {
                self.regs.status.set_sprite_o_f(true);
                return;
            }
            let sprite_y = self.oam[sprite_index * 4] as u16;
            if sprite_y <= self.scanline
                && sprite_y + self.regs.ctrl.sprite_size() > self.scanline
            {
                self.tmp_oam[sprite_count] =
                    Some(Sprite::new(sprite_index, &self.oam));
                sprite_count += 1;
            }
        }
    }

    fn sprite_pixel(
        &mut self,
        x: u8,
        bg_opaque: bool,
    ) -> (u8, Option<Priority>) {
        for sprite in self.main_oam.iter() {
            if !self.regs.mask.show_sprites() {
                return (0, None);
            }

            if let Some(s) = sprite {
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

                let left_byte = self.vram.ld8(pt_tile_i);
                // Plus 8 to get the offset for the other sliver
                let right_byte = self.vram.ld8(pt_tile_i + 8);
                let l_bit = ((left_byte << x_offset & 0x80) != 0) as u8;
                let r_bit = ((right_byte << x_offset & 0x80) != 0) as u8;
                let tile_color = r_bit << 1 | l_bit;

                if tile_color == 0 {
                    continue;
                }

                if s.index == 0 && bg_opaque {
                    self.regs.status.set_sprite_0_hit(true);
                }

                let sprite_color =
                    (s.attributes.palette()) + 4 << 2 | tile_color;
                return (
                    sprite_color,
                    Some(Priority::from_attr(s.attributes.priority() as u8)),
                );
            } else {
                return (0, None);
            }
        }
        (0, None)
    }

    fn step_bg_regs(&mut self) {
        match self.cc {
            2...256 | 322...337 => match self.cc % 8 {
                1 => {
                    self.internal_regs.reload(self.at_entry);
                }
                //This is over simplified to make it faster. If there are
                //games that write to vram during rendering and the emulator
                //does not work, check this part out first.
                0 => {
                    let nt_entry = self.vram.ld8(self.regs.addr.nt_addr());
                    self.at_entry = self.vram.ld8(self.regs.addr.at_addr());

                    if self.regs.addr.coarse_y() % 4 >= 2 {
                        self.at_entry >>= 4;
                    }

                    if self.regs.addr.coarse_x() % 4 >= 2 {
                        self.at_entry >>= 2;
                    }
                    let pt_index = self.regs.ctrl.nt_pt_addr()
                        + (nt_entry as u16 * 16)
                        + self.regs.addr.fine_y() as u16;
                    self.internal_regs.bg_latch.low_tile =
                        self.vram.ld8(pt_index);
                    self.internal_regs.bg_latch.high_tile =
                        self.vram.ld8(pt_index + 8);
                    if self.regs.mask.show_bg() {
                        if self.cc == 256 {
                            self.regs.addr.scroll_y();
                        } else {
                            self.regs.addr.scroll_x();
                        }
                    }
                }
                _ => (),
            },
            257 => {
                self.internal_regs.reload(self.at_entry);
                if self.regs.mask.show_bg() {
                    self.regs.addr.pull_x(self.t_addr);
                }
            }

            280...304 => {
                if self.is_prerender() && self.regs.mask.show_bg() {
                    self.regs.addr.pull_y(self.t_addr);
                }
            }

            _ => (),
        }
    }

    fn bg_pixel(&self, x: u8) -> u8 {
        if (x <= 8 && !self.regs.mask.left8_bg()) || !self.regs.mask.show_bg() {
            return 0;
        }
        let bg_off = 15 - self.fine_x;
        let at_off = 7 - self.fine_x;
        let c = ((((self.internal_regs.bg_shift.high_tile >> (bg_off)) & 1)
            as u8)
            << 1)
            | (((self.internal_regs.bg_shift.low_tile >> (bg_off)) & 1) as u8);

        if c == 0 {
            return 0;
        }
        (((((self.internal_regs.at_shift.high_tile >> (at_off)) & 1) as u8)
            << 1)
            | (((self.internal_regs.at_shift.low_tile >> (at_off)) & 1) as u8))
            << 2
            | c
    }

    fn is_prerender(&self) -> bool {
        self.scanline == PRERENDER
    }

    //TODO: Split this logic into 2 functions to separate out the shift since
    //it's not part of rendering pixels
    fn render_pixel(&mut self) {
        match self.cc {
            2...257 | 322...337 => {
                let x = self.cc - 2;
                if x < 256 && !self.is_prerender() {
                    let bg_color = self.bg_pixel(x as u8);
                    let (spr_color, priority) =
                        self.sprite_pixel(x as u8, bg_color != 0);

                    let color = match (bg_color, spr_color) {
                        (0, 0) => 0,
                        (bg_c, 0) => bg_c,
                        (0, spr_c) => spr_c,
                        (bg_c, spr_c) => {
                            match priority.expect("Cannot get none here") {
                                Priority::Foreground => spr_c,
                                Priority::Background => bg_c,
                            }
                        }
                    };

                    self.put_pixel(
                        x as usize,
                        self.scanline as usize,
                        self.get_palette_color(color),
                    );
                }
                self.internal_regs.shift();
            }
            _ => (),
        }
    }

    pub fn get_buffer(&self) -> &[u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3] {
        &self.screen_buff
    }

    fn step_cc(&mut self) {
        self.cc += 1;
        if self.cc >= 341 {
            self.cc %= 341;
            self.scanline += 1;
            if self.scanline > PRERENDER {
                self.scanline = 0;
            }
        }
    }

    fn step(&mut self) -> Option<PpuRes> {
        let mut res = match self.scanline {
            0...239 => {
                self.step_sprites();
                self.render_pixel();
                self.step_bg_regs();
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
                if self.cc == 1 && !self.vblank_off {
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
            PRERENDER => {
                if self.cc == 1 {
                    self.regs.status.set_vblank(false);
                }
                self.step_sprites();
                self.render_pixel();
                self.step_bg_regs();
                None
            }
            _ => panic!(
                "Scanline can't get here {}. Check emulate_cycles",
                self.scanline
            ),
        };

        // This is the logic for forcing nmi
        if self.trip_nmi && self.regs.status.vblank() && !self.vblank_off {
            match res {
                None => {
                    res = Some(PpuRes::Nmi);
                }
                _ => panic!("This shouldn't be possible"),
            }
        };

        self.trip_nmi = false;
        self.vblank_off = false;

        self.step_cc();
        res
    }

    pub fn emulate_cycles(&mut self, cyc_elapsed: u16) -> Option<PpuRes> {
        let mut ppu_res = None;
        for _ in 0..(cyc_elapsed * 3) {
            if let Some(res) = self.step() {
                ppu_res = Some(res);
            }
        }
        ppu_res
    }
}
