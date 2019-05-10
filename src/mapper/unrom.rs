use serde::Serialize;
use serde::Deserialize;

const SIXTEEN_KB: usize = 0x4000;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Unrom {
    bank_select: u8,
    last_page_start: usize,
}

impl Unrom {
    pub fn new(last_page_start: usize) -> Unrom {
        Unrom {
            bank_select: 0,
            last_page_start,
        }
    }

    pub fn store_prg(&mut self, address: u16, val: u8) {
        if address >= 0x8000 {
            self.bank_select = val & 0b111;
        } else {
            info!(
                "Writing to unmapped prg_rom address: {:X} val: {}",
                address, val
            );
        }
    }

    pub fn ld_prg(&self, address: u16, prg_rom: &Vec<u8>) -> u8 {
        if address < 0x8000 {
            info!("Reading from unmapped prg_rom address: {:X}", address);
            0
        // Bank switched using 3 bits
        } else if address < 0xC000 {
            prg_rom[(self.bank_select as usize * SIXTEEN_KB)
                + (address as usize - 0x8000)]
        // Hard wired to last 16KB
        } else {
            prg_rom[(7 * SIXTEEN_KB) + (address as usize - 0xC000)]
        }
    }

    pub fn ld_chr(&self, address: u16, chr_ram: &Vec<u8>) -> u8 {
        chr_ram[address as usize]
    }

    pub fn store_chr(&mut self, address: u16, val: u8, chr_ram: &mut Vec<u8>) {
        chr_ram[address as usize] = val;
    }

    pub fn reset(&mut self) {
        self.bank_select = 0;
    }
}
