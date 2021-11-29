use mapper::Mapper;
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
use std::rc::Rc;

use ppu::pregisters::Ctrl;
use ppu::pregisters::PRegisters;
use ppu::pregisters::VramAddr;
use ppu::shift_regs::InternalRegs;
use ppu::sprite::Priority;
use ppu::sprite::Sprite;
use ppu::vram::*;

pub mod pregisters;
pub mod shift_regs;
pub mod sprite;
pub mod vram;

const SPRITE_NUM: usize = 64;
const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;
const PRERENDER: u16 = 261;

pub const PALETTE: [u32; 64] = [
    0x808080, 0x003DA6, 0x0012B0, 0x440096, 0xA1005E, 0xC70028, 0xBA0600,
    0x8C1700, 0x5C2F00, 0x104500, 0x054A00, 0x00472E, 0x004166, 0x000000,
    0x050505, 0x050505, 0xC7C7C7, 0x0077FF, 0x2155FF, 0x8237FA, 0xEB2FB5,
    0xFF2950, 0xFF2200, 0xD63200, 0xC46200, 0x358000, 0x058F00, 0x008A55,
    0x0099CC, 0x212121, 0x090909, 0x090909, 0xFFFFFF, 0x0FD7FF, 0x69A2FF,
    0xD480FF, 0xFF45F3, 0xFF618B, 0xFF8833, 0xFF9C12, 0xFABC20, 0x9FE30E,
    0x2BF035, 0x0CF0A4, 0x05FBFF, 0x5E5E5E, 0x0D0D0D, 0x0D0D0D, 0xFFFFFF,
    0xA6FCFF, 0xB3ECFF, 0xDAABEB, 0xFFA8F9, 0xFFABB3, 0xFFD2B0, 0xFFEFA6,
    0xFFF79C, 0xD7E895, 0xA6EDAF, 0xA2F2DA, 0x99FFFC, 0xDDDDDD, 0x111111,
    0x111111,
];

#[derive(Debug)]
pub enum PpuRes {
    Nmi,
    Draw,
}

#[derive(Copy, Clone)]
struct Rgb {
    data: [u8; 3],
}

#[derive(Serialize, Deserialize)]
pub struct PpuState {
    vram: Box<[u8]>,
    palette: [u8; 0x20],
    ppu_regs: PRegisters,
    ppu_render_regs: InternalRegs,
    cc: u16,
    scanline: u16,
    write_latch: bool,
    t_addr: VramAddr,
    trip_nmi: bool,
    vblank_off: bool,
    at_entry: u8,
}

pub struct Ppu {
    pub regs: PRegisters,
    vram: Vram,
    // multiply by 3 to account for r g b
    screen_buff: Box<[u8]>,
    oam: [u8; 256],
    tmp_oam: Vec<Sprite>,
    main_oam: Vec<Sprite>,
    cc: u16,
    scanline: u16,
    // Internal registers
    //
    // Write latch is another 1 bit latch that stores data on which write we
    // are on
    write_latch: bool,
    // Data buff is used for the 1 byte delay when reading data from port 7
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
    odd_frame: bool,
}

impl Ppu {
    pub fn new(mapper: Rc<RefCell<Mapper>>) -> Ppu {
        Ppu {
            trip_nmi: false,
            vblank_off: false,
            regs: PRegisters::default(),
            vram: Vram::new(mapper),
            screen_buff: Box::new([0; SCREEN_WIDTH * 3 * SCREEN_HEIGHT]),
            oam: [0; 256],
            tmp_oam: Vec::with_capacity(8),
            main_oam: Vec::with_capacity(8),
            cc: 0,
            scanline: 0,
            write_latch: false,
            fine_x: 0,
            t_addr: VramAddr(0),
            at_entry: 0,
            internal_regs: InternalRegs::default(),
            odd_frame: false,
        }
    }

    pub fn get_state(&self) -> PpuState {
        PpuState {
            vram: self.vram.vram.clone(),
            palette: self.vram.palette,
            ppu_regs: self.regs,
            ppu_render_regs: self.internal_regs,
            cc: self.cc,
            scanline: self.scanline,
            write_latch: self.write_latch,
            t_addr: self.t_addr,
            trip_nmi: self.trip_nmi,
            vblank_off: self.vblank_off,
            at_entry: self.at_entry,
        }
    }

    pub fn set_state(&mut self, ppu_state: PpuState) {
        self.vram.vram = ppu_state.vram;
        self.vram.palette = ppu_state.palette;
        self.regs = ppu_state.ppu_regs;
        self.internal_regs = ppu_state.ppu_render_regs;
        self.cc = ppu_state.cc;
        self.scanline = ppu_state.scanline;
        self.write_latch = ppu_state.write_latch;
        self.t_addr = ppu_state.t_addr;
        self.trip_nmi = ppu_state.trip_nmi;
        self.vblank_off = ppu_state.vblank_off;
        self.at_entry = ppu_state.at_entry;
    }

    pub fn reset(&mut self) {
        self.trip_nmi = false;
        self.vblank_off = false;
        self.regs = PRegisters::default();
        self.vram.reset();
        self.screen_buff = Box::new([0; SCREEN_WIDTH * 3 * SCREEN_HEIGHT]);
        self.oam = [0; 256];
        self.tmp_oam = Vec::with_capacity(8);
        self.main_oam = Vec::with_capacity(8);
        self.cc = 0;
        self.scanline = 0;
        self.write_latch = false;
        self.fine_x = 0;
        self.t_addr = VramAddr(0);
        self.at_entry = 0;
        self.internal_regs = InternalRegs::default();
    }

    fn get_palette_color(&self, vram_offset: u8) -> Rgb {
        let pal_index = (self.vram.ld8(0x3F00 + vram_offset as u16)) & 0x3F;
        let num = PALETTE[pal_index as usize];
        Rgb {
            data: [
                ((num & 0xFF0000) >> 16) as u8,
                ((num & 0x00FF00) >> 8) as u8,
                (num & 0xFF) as u8,
            ],
        }
    }

    pub fn ld(&mut self, address: u16) -> (u8, Option<bool>) {
        match address {
            2 => (self.read_ppustatus(), None),
            4 => (self.oam[self.regs.oam_addr as usize], None),
            7 => {
                let (val, pal_read) = self.read_ppudata();
                (val, Some(pal_read))
            }
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

    // This bool tells us if this read was from a palette or not
    fn read_ppudata(&mut self) -> (u8, bool) {
        let addr = self.regs.addr.addr();
        let val = self.vram.buffered_ld8(addr);
        self.regs.addr.add_offset(self.regs.ctrl.vram_incr());
        (val, (0x3F00..=0x3FFF).contains(&addr))
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
            self.regs.addr = self.t_addr;
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
                self.tmp_oam.clear();
                if self.is_prerender() {
                    self.regs.status.set_sprite_o_f(false);
                    self.regs.status.set_sprite_0_hit(false);
                }
            }
            257 => {
                self.get_sprites();
                self.regs.oam_addr = 0;
            }
            258..=320 => {
                self.regs.oam_addr = 0;
            }
            321 => {
                self.main_oam = self.tmp_oam.clone();
                for sprite in self.main_oam.iter_mut() {
                    let address =
                        sprite.get_pt_address(&self.regs.ctrl, self.scanline);
                    sprite.low_byte = self.vram.ld8(address);
                    sprite.high_byte = self.vram.ld8(address + 8);
                }
            }
            _ => (),
        }
    }

    fn get_sprites(&mut self) {
        self.tmp_oam.clear();
        for sprite_index in 0..SPRITE_NUM {
            if self.tmp_oam.len() == 8 {
                self.regs.status.set_sprite_o_f(true);
                return;
            }
            let sprite_y = self.oam[sprite_index * 4] as u16;
            if sprite_y <= self.scanline
                && sprite_y + self.regs.ctrl.sprite_size() > self.scanline
            {
                self.tmp_oam.push(Sprite::new(sprite_index, &self.oam));
            }
        }
    }

    fn sprite_pixel(
        &mut self,
        x: u8,
        bg_opaque: bool,
    ) -> (u8, Option<Priority>) {
        for sprite in self.main_oam.iter() {
            if !self.regs.mask.show_sprites()
                || (sprite.x < 8 && !self.regs.mask.left8_sprite())
                || !sprite.in_bounding_box(x)
            {
                continue;
            }

            let x_offset = if sprite.attributes.flip_x() {
                7 - (x.wrapping_sub(sprite.x))
            } else {
                x.wrapping_sub(sprite.x)
            };

            let l_bit = ((sprite.low_byte << x_offset & 0x80) != 0) as u8;
            let r_bit = ((sprite.high_byte << x_offset & 0x80) != 0) as u8;
            let tile_color = r_bit << 1 | l_bit;

            if tile_color == 0 {
                continue;
            }

            if sprite.index == 0 && bg_opaque && sprite.x != 255 {
                self.regs.status.set_sprite_0_hit(true);
            }

            let sprite_color =
                ((sprite.attributes.palette()) + 4) << 2 | tile_color;
            return (
                sprite_color,
                Some(Priority::from_attr(sprite.attributes.priority() as u8)),
            );
        }
        (0, None)
    }

    fn step_bg_regs(&mut self) {
        match self.cc {
            2..=256 | 322..=337 => match self.cc % 8 {
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
                    self.internal_regs.bg_latch.fill(
                        self.vram.ld8(pt_index),
                        self.vram.ld8(pt_index + 8),
                    );

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

            280..=304 => {
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
        let c = self.internal_regs.bg_shift.get_color(bg_off);
        if c == 0 {
            return 0;
        }
        self.internal_regs.at_shift.get_color(c, at_off)
    }

    fn is_prerender(&self) -> bool {
        self.scanline == PRERENDER
    }

    //TODO: Split this logic into 2 functions to separate out the shift since
    //it's not part of rendering pixels
    fn render_pixel(&mut self) {
        match self.cc {
            2..=257 | 322..=337 => {
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

    pub fn get_buffer(&self) -> &[u8] {
        &self.screen_buff
    }

    fn step_cc(&mut self) {
        self.cc += 1;
        if self.odd_frame && self.is_prerender() && (self.cc == 340) {
            self.scanline = 0;
            self.cc = 0;
            self.odd_frame = !self.odd_frame;
        } else if self.cc > 340 {
            self.cc = 0;
            self.scanline += 1;
            if self.scanline > PRERENDER {
                if self.regs.mask.show_bg() {
                    self.odd_frame = !self.odd_frame;
                }
                self.scanline = 0;
            }
        }
    }

    fn step(&mut self) -> Option<PpuRes> {
        let mut res = match self.scanline {
            0..=239 => {
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
            242..=260 => None,
            PRERENDER => {
                if self.cc == 1 {
                    self.regs.status.set_vblank(false);
                    self.regs.status.set_sprite_0_hit(false);
                    self.regs.status.set_sprite_o_f(false);
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
