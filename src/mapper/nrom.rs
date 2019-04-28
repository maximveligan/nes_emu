use serde::Serialize;
use serde::Deserialize;

const UNMIRRORED_MASK: usize = 0x7FFF;
const MIRRORED_MASK: usize = 0x3FFF;
const NROM_PRG_ROM_START: u16 = 0x8000;

const SIXTEEN_KB: usize = 0x4000;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Nrom {
    mirrored: bool,
}

impl Nrom {
    pub fn new(prg_rom_size: usize) -> Nrom {
        Nrom {
            mirrored: prg_rom_size <= SIXTEEN_KB,
        }
    }

    pub fn ld_prg(&self, address: u16, prg_rom: &Vec<u8>) -> u8 {
        if address < NROM_PRG_ROM_START {
            0
        } else if self.mirrored {
            prg_rom[address as usize & MIRRORED_MASK]
        } else {
            prg_rom[address as usize & UNMIRRORED_MASK]
        }
    }

    pub fn ld_chr(&self, address: u16, chr_rom: &Vec<u8>) -> u8 {
        chr_rom[address as usize]
    }
}
