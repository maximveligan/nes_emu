// BattleToads and BattleToads Double Dragon boot and show intro screen,
// but then hang indefinetely. Almost certainly timing issues, as this mapper is
// very simple. Marble Madness works fine.

use rom::ScreenBank;
use rom::ScreenMode;
use serde::Deserialize;
use serde::Serialize;

const THIRTY_TWO_KB: usize = 0x8000;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Axrom {
    bank_select: u8,
    mirror_select: u8,
    last_page_start: usize,
}

impl Axrom {
    pub fn new(last_page_start: usize) -> Axrom {
        Axrom {
            bank_select: 0,
            mirror_select: 0,
            last_page_start,
        }
    }

    pub fn store_prg(&mut self, address: u16, val: u8) {
        if address >= 0x8000 {
            self.bank_select = val & 0b111;
            self.mirror_select = (val >> 4) & 1;
        } else {
            info!(
                "Writing to unmapped prg_rom address: {:X} val: {}",
                address, val
            );
        }
    }

    pub fn ld_prg(&self, address: u16, prg_rom: &[u8]) -> u8 {
        if address < 0x8000 {
            info!("Reading from unmapped prg_rom address: {:X}", address);
            0
        // Bank switched using 3 bits
        } else {
            prg_rom[(self.bank_select as usize * THIRTY_TWO_KB)
                + (address as usize - 0x8000)]
        }
    }

    pub fn ld_chr(&self, address: u16, chr_ram: &[u8]) -> u8 {
        chr_ram[address as usize]
    }

    pub fn store_chr(&mut self, address: u16, val: u8, chr_ram: &mut Vec<u8>) {
        chr_ram[address as usize] = val;
    }

    pub fn reset(&mut self) {
        self.bank_select = 0;
    }

    pub fn get_mirroring(&self) -> &ScreenMode {
        match self.mirror_select {
            0 => &ScreenMode::OneScreenSwap(ScreenBank::Lower),
            1 => &ScreenMode::OneScreenSwap(ScreenBank::Upper),
            _ => panic!("2 bit number can't be greater than 1"),
        }
    }
}
