use crate::mapper::Mapper;
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ppu::pregisters::Ctrl;
use crate::ppu::pregisters::PRegisters;
use crate::ppu::pregisters::VramAddr;
use crate::ppu::shift_regs::InternalRegs;
use crate::ppu::sprite::Priority;
use crate::ppu::sprite::Sprite;
use crate::ppu::vram::*;

pub mod pregisters;
pub mod shift_regs;
pub mod sprite;
pub mod vram;

const SPRITE_NUM: usize = 64;
const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;
const PRERENDER: u16 = 261;
// The PPU's vblank flag reaches the CPU-facing NMI line after several dots.
const NMI_EDGE_DELAY_DOTS: u8 = 5;
const RENDERING_SIGNAL_DELAY_DOTS: u8 = 1;

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
    nmi_delay: u8,
    rendering_for_skip: bool,
    rendering_delay: u8,
    at_entry: u8,
    oam_eval_n: u8,
    oam_eval_m: u8,
    oam_eval_latch: u8,
}

pub struct Ppu {
    pub regs: PRegisters,
    pub frame_ready: bool,
    pub nmi_pending: bool,
    nmi_delay: u8,
    rendering_for_skip: bool,
    rendering_delay: u8,
    vram: Vram,
    // multiply by 3 to account for r g b
    screen_buff: Box<[u8]>,
    oam: [u8; 256],
    tmp_oam: Vec<Sprite>,
    main_oam: Vec<Sprite>,
    // Sprite evaluation walks primary OAM over dots 65-256. `n` selects the
    // sprite and `m` selects its byte; odd dots read OAM and even dots
    // process the latched byte.
    oam_eval_n: u8,
    oam_eval_m: u8,
    oam_eval_latch: u8,
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
    // When an NMI is tripped, the NMI shouldn't execute until after the NEXT
    // instruction. This flag is used to differentiate between a regular NMI
    // and a forced NMI.
    pub queued_nmi: bool,
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
            oam_eval_n: 0,
            oam_eval_m: 0,
            oam_eval_latch: 0,
            cc: 0,
            scanline: 0,
            write_latch: false,
            fine_x: 0,
            t_addr: VramAddr(0),
            at_entry: 0,
            internal_regs: InternalRegs::default(),
            odd_frame: false,
            frame_ready: false,
            nmi_pending: false,
            nmi_delay: 0,
            rendering_for_skip: false,
            rendering_delay: 0,
            queued_nmi: false,
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
            nmi_delay: self.nmi_delay,
            rendering_for_skip: self.rendering_for_skip,
            rendering_delay: self.rendering_delay,
            at_entry: self.at_entry,
            oam_eval_n: self.oam_eval_n,
            oam_eval_m: self.oam_eval_m,
            oam_eval_latch: self.oam_eval_latch,
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
        self.nmi_delay = ppu_state.nmi_delay;
        self.rendering_for_skip = ppu_state.rendering_for_skip;
        self.rendering_delay = ppu_state.rendering_delay;
        self.at_entry = ppu_state.at_entry;
        self.oam_eval_n = ppu_state.oam_eval_n;
        self.oam_eval_m = ppu_state.oam_eval_m;
        self.oam_eval_latch = ppu_state.oam_eval_latch;
    }

    pub fn reset(&mut self) {
        self.trip_nmi = false;
        self.vblank_off = false;
        self.nmi_delay = 0;
        self.rendering_for_skip = false;
        self.rendering_delay = 0;
        self.regs = PRegisters::default();
        self.vram.reset();
        self.screen_buff = Box::new([0; SCREEN_WIDTH * 3 * SCREEN_HEIGHT]);
        self.oam = [0; 256];
        self.tmp_oam = Vec::with_capacity(8);
        self.main_oam = Vec::with_capacity(8);
        self.oam_eval_n = 0;
        self.oam_eval_m = 0;
        self.oam_eval_latch = 0;
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
        if self.scanline == 241 && self.cc <= 3 {
            log::debug!(
                "PPUSTATUS read near vblank: scanline={} dot={} value={:02X} nmi_pending={}",
                self.scanline,
                self.cc,
                tmp,
                self.nmi_pending,
            );
        }
        self.regs.status.set_vblank(false);
        // A status read in the vblank race window can cancel the NMI edge.
        if self.nmi_delay > 0 && self.scanline == 241 && self.cc <= 3 {
            log::debug!(
                "PPUSTATUS read cancels pending NMI edge: scanline={} dot={}",
                self.scanline,
                self.cc,
            );
            self.nmi_delay = 0;
        }
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
                let old_bg = self.regs.mask.show_bg();
                let old_rendering = old_bg || self.regs.mask.show_sprites();
                self.regs.mask.store(val);
                let new_rendering =
                    self.regs.mask.show_bg() || self.regs.mask.show_sprites();
                if new_rendering != self.rendering_for_skip {
                    self.rendering_delay = RENDERING_SIGNAL_DELAY_DOTS;
                } else {
                    self.rendering_delay = 0;
                }
                log::debug!(
                    "PPUMASK write: scanline={} dot={} value={:02X} old_bg={} new_bg={} old_rendering={} new_rendering={}",
                    self.scanline,
                    self.cc,
                    val,
                    old_bg,
                    self.regs.mask.show_bg(),
                    old_rendering,
                    new_rendering,
                );
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
        self.trip_nmi = !self.regs.ctrl.nmi_on() && ctrl.nmi_on();
        if !ctrl.nmi_on()
            && self.nmi_delay > 0
            && self.scanline == 241
            && self.cc <= 3
        {
            log::debug!(
                "PPUCTRL write cancels pending NMI edge: scanline={} dot={}",
                self.scanline,
                self.cc,
            );
            self.nmi_delay = 0;
        }
        log::debug!(
            "PPUCTRL write: scanline={} dot={} value={:02X} old_nmi={} new_nmi={} vblank={} vblank_off={} trip_nmi={}",
            self.scanline,
            self.cc,
            val,
            self.regs.ctrl.nmi_on(),
            ctrl.nmi_on(),
            self.regs.status.vblank(),
            self.vblank_off,
            self.trip_nmi,
        );
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
        let rendering_enabled =
            self.regs.mask.show_bg() || self.regs.mask.show_sprites();
        match self.cc {
            1 => {
                self.tmp_oam.clear();
                self.oam_eval_n = 0;
                self.oam_eval_m = 0;
                self.oam_eval_latch = 0;
                if self.is_prerender() {
                    self.regs.status.set_sprite_o_f(false);
                    self.regs.status.set_sprite_0_hit(false);
                }
            }
            65..=256 if !self.is_prerender() => {
                self.step_sprite_evaluation();
            }
            257 if rendering_enabled => {
                self.regs.oam_addr = 0;
            }
            258..=320 if rendering_enabled => {
                self.regs.oam_addr = 0;
                let fetch_dot = (self.cc - 257) % 8;
                let sprite_index = ((self.cc - 257) / 8) as usize;
                if let Some(sprite) = self.tmp_oam.get_mut(sprite_index) {
                    if fetch_dot == 4 || fetch_dot == 6 {
                        let address = sprite
                            .get_pt_address(&self.regs.ctrl, self.scanline);
                        if fetch_dot == 4 {
                            sprite.low_byte = self.vram.ld8(address);
                        } else {
                            sprite.high_byte = self.vram.ld8(address + 8);
                        }
                    }
                }
            }
            321 => {
                self.main_oam = self.tmp_oam.clone();
            }
            _ => (),
        }
    }

    fn step_sprite_evaluation(&mut self) {
        if !self.regs.mask.show_bg() && !self.regs.mask.show_sprites() {
            return;
        }

        // Evaluation reads primary OAM on odd dots and processes that byte on
        // the following even dot.
        if self.cc % 2 == 1 {
            if (self.oam_eval_n as usize) < SPRITE_NUM {
                let oam_index =
                    self.oam_eval_n as usize * 4 + self.oam_eval_m as usize;
                self.oam_eval_latch = self.oam[oam_index];
            }
            return;
        }

        if self.oam_eval_n as usize >= SPRITE_NUM {
            return;
        }

        let in_range = (self.oam_eval_latch as u16) <= self.scanline
            && (self.oam_eval_latch as u16 + self.regs.ctrl.sprite_size())
                > self.scanline;

        // Finish copying a selected sprite even if its Y byte filled the
        // eighth slot. Overflow evaluation begins at the next sprite.
        let copying_selected_sprite = self.oam_eval_m != 0
            && self.tmp_oam.last().is_some_and(|sprite| {
                sprite.index == self.oam_eval_n
            });
        if copying_selected_sprite {
            if let Some(sprite) = self.tmp_oam.last_mut() {
                sprite.set_oam_byte(self.oam_eval_m, self.oam_eval_latch);
            }
            if self.oam_eval_m == 3 {
                self.oam_eval_n += 1;
                self.oam_eval_m = 0;
            } else {
                self.oam_eval_m += 1;
            }
            return;
        }

        if self.tmp_oam.len() < 8 {
            if in_range {
                self.tmp_oam.push(Sprite::with_y(
                    self.oam_eval_n as usize,
                    self.oam_eval_latch,
                ));
                self.oam_eval_m = 1;
            } else {
                self.oam_eval_n += 1;
            }
            return;
        }

        // After eight sprites, the PPU keeps scanning. The first extra
        // in-range value sets overflow. On misses, n and m advance
        // independently, reproducing the diagonal OAM scan bug.
        if in_range {
            self.regs.status.set_sprite_o_f(true);
            if self.oam_eval_m == 3 {
                self.oam_eval_n += 1;
                self.oam_eval_m = 0;
            } else {
                self.oam_eval_m += 1;
            }
        } else {
            self.oam_eval_n += 1;
            self.oam_eval_m = (self.oam_eval_m + 1) & 3;
        }
    }

    fn sprite_pixel(
        &mut self,
        x: u8,
        bg_opaque: bool,
    ) -> (u8, Option<Priority>) {
        for sprite in self.main_oam.iter() {
            if !self.regs.mask.show_sprites()
                || (x < 8 && !self.regs.mask.left8_sprite())
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

            if sprite.index == 0 && bg_opaque && x != 255 {
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
        if (x < 8 && !self.regs.mask.left8_bg()) || !self.regs.mask.show_bg() {
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
        if self.odd_frame && self.is_prerender() && self.cc == 340 {
            log::debug!(
                "Odd-frame skip decision: scanline={} dot=339 rendering_enabled={}",
                self.scanline,
                self.rendering_for_skip,
            );
        }
        if self.odd_frame
            && self.is_prerender()
            && self.cc == 340
            && self.rendering_for_skip
        {
            self.scanline = 0;
            self.cc = 0;
            self.odd_frame = !self.odd_frame;
        } else if self.cc > 340 {
            self.cc = 0;
            self.scanline += 1;
            if self.scanline > PRERENDER {
                self.odd_frame = !self.odd_frame;
                self.scanline = 0;
            }
        }
    }

    fn step(&mut self) {
        if self.nmi_delay > 0 {
            self.nmi_delay -= 1;
            if self.nmi_delay == 0 {
                self.nmi_pending = true;
                log::debug!(
                    "NMI delay elapsed: scanline={} dot={} nmi_pending=true",
                    self.scanline,
                    self.cc,
                );
            }
        }

        match self.scanline {
            0..=239 => {
                self.step_sprites();
                self.render_pixel();
                self.step_bg_regs();
            }
            240 => {
                if self.cc == 0 {
                    self.frame_ready = true;
                }
            }
            241 => {
                if self.cc == 1 {
                    if !self.vblank_off {
                        self.regs.status.set_vblank(true);
                    }
                    if !self.vblank_off && self.regs.ctrl.nmi_on() {
                        self.nmi_delay = NMI_EDGE_DELAY_DOTS;
                    }
                    log::debug!(
                        "VBLANK edge: scanline={} dot={} suppressed={} nmi_enabled={} nmi_delay={} nmi_pending={}",
                        self.scanline,
                        self.cc,
                        self.vblank_off,
                        self.regs.ctrl.nmi_on(),
                        self.nmi_delay > 0,
                        self.nmi_pending,
                    );
                }
            }
            242..=260 => (),
            PRERENDER => {
                if self.cc == 1 {
                    self.regs.status.set_vblank(false);
                    self.regs.status.set_sprite_0_hit(false);
                    self.regs.status.set_sprite_o_f(false);
                }
                self.step_sprites();
                self.render_pixel();
                self.step_bg_regs();
            }
            _ => panic!(
                "Scanline can't get here {}. Check emulate_cycles",
                self.scanline
            ),
        };

        // This is the logic for forcing nmi
        let force_nmi =
            self.trip_nmi && self.regs.status.vblank() && !self.vblank_off;
        if force_nmi {
            log::debug!(
                "Queue forced NMI: scanline={} dot={} vblank={} vblank_off={}",
                self.scanline,
                self.cc,
                self.regs.status.vblank(),
                self.vblank_off,
            );
        }
        self.queued_nmi |= force_nmi;

        self.trip_nmi = false;
        self.vblank_off = false;

        self.step_cc();

        if self.rendering_delay > 0 {
            self.rendering_delay -= 1;
            if self.rendering_delay == 0 {
                self.rendering_for_skip =
                    self.regs.mask.show_bg() || self.regs.mask.show_sprites();
            }
        }
    }

    pub fn emulate_cycles(&mut self, cyc_elapsed: usize) {
        for _ in 0..(cyc_elapsed * 3) {
            self.step();
        }
    }
}
