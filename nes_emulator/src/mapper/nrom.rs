use log::*;
use serde::Deserialize;
use serde::Serialize;

const UNMIRRORED_MASK: usize = 0x7FFF;
const MIRRORED_MASK: usize = 0x3FFF;
const NROM_PRG_ROM_START: u16 = 0x8000;

const SIXTEEN_KB: usize = 0x4000;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Nrom {
    mirrored: bool,
    use_chr_ram: bool,
}

impl Nrom {
    pub fn new(prg_rom_size: usize, use_chr_ram: bool) -> Nrom {
        Nrom {
            mirrored: prg_rom_size <= SIXTEEN_KB,
            use_chr_ram,
        }
    }

    pub fn ld_prg(&self, address: u16, prg_rom: &[u8]) -> u8 {
        if address < NROM_PRG_ROM_START {
            info!("Attempt to read from nrom {:X}", address);
            0
        } else if self.mirrored {
            prg_rom[address as usize & MIRRORED_MASK]
        } else {
            prg_rom[address as usize & UNMIRRORED_MASK]
        }
    }

    pub fn store_prg(&self, address: u16, val: u8) {
        info!(
            "Attempt to write to nrom address {:X}, val {}",
            address, val
        );
    }

    pub fn ld_chr(&self, address: u16, chr_rom: &[u8], chr_ram: &[u8]) -> u8 {
        if self.use_chr_ram {
            chr_ram[address as usize]
        } else {
            chr_rom[address as usize]
        }
    }

    pub fn store_chr(&mut self, address: u16, val: u8, chr_ram: &mut Vec<u8>) {
        if self.use_chr_ram {
            chr_ram[address as usize] = val;
        } else {
            info!("Attempt to store to nrom address {:X} val {}", address, val);
        }
    }
}
