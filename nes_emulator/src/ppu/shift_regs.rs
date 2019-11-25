use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Copy, Clone)]
//These are 1 bit latches, so I'm using booleans to store them
pub struct AtLatch {
    low_b: bool,
    high_b: bool,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct AtShift {
    low_tile: u8,
    high_tile: u8,
}

impl AtShift {
    pub fn get_color(&self, c: u8, at_off: u8) -> u8 {
        (((((self.high_tile >> (at_off)) & 1) as u8) << 1)
            | (((self.low_tile >> (at_off)) & 1) as u8))
            << 2
            | c
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct BgLatch {
    low_tile: u8,
    high_tile: u8,
}

impl BgLatch {
    pub fn fill(&mut self, low: u8, high: u8) {
        self.low_tile = low;
        self.high_tile = high;
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct BgShift {
    low_tile: u16,
    high_tile: u16,
}

impl BgShift {
    pub fn get_color(&self, bg_off: u8) -> u8 {
        ((((self.high_tile >> (bg_off)) & 1) as u8) << 1)
            | (((self.low_tile >> (bg_off)) & 1) as u8)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct InternalRegs {
    pub at_latch: AtLatch,
    pub at_shift: AtShift,
    pub bg_latch: BgLatch,
    pub bg_shift: BgShift,
}

impl InternalRegs {
    pub fn new() -> InternalRegs {
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

    pub fn reload(&mut self, at_entry: u8) {
        self.bg_shift.low_tile =
            (self.bg_shift.low_tile & 0xFF00) | self.bg_latch.low_tile as u16;
        self.bg_shift.high_tile =
            (self.bg_shift.high_tile & 0xFF00) | self.bg_latch.high_tile as u16;
        self.at_latch.low_b = (at_entry & 1) == 1;
        self.at_latch.high_b = ((at_entry >> 1) & 1) == 1;
    }

    pub fn shift(&mut self) {
        self.at_shift.low_tile =
            (self.at_shift.low_tile << 1) | self.at_latch.low_b as u8;
        self.at_shift.high_tile =
            (self.at_shift.high_tile << 1) | self.at_latch.high_b as u8;
        self.bg_shift.low_tile <<= 1;
        self.bg_shift.high_tile <<= 1;
    }
}
