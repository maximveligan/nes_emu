use serde::Deserialize;
use serde::Serialize;

const UNMIRRORED_MASK: usize = 0x7FFF;
const MIRRORED_MASK: usize = 0x3FFF;

const SIXTEEN_KB: usize = 0x4000;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Cnrom {
    mirrored: bool,
    use_chr_ram: bool,
    chr_rom_offset: usize,
}

impl Cnrom {
    pub fn new(prg_rom_size: usize, use_chr_ram: bool) -> Cnrom {
        Cnrom {
            mirrored: prg_rom_size <= SIXTEEN_KB,
            use_chr_ram,
            chr_rom_offset: 0,
        }
    }

    pub fn ld_prg(&self, address: u16, prg_rom: &[u8]) -> u8 {
        if address < 0x8000 {
            info!("Attempt to read from cnrom {:X}", address);
            0
        } else if self.mirrored {
            prg_rom[address as usize & MIRRORED_MASK]
        } else {
            prg_rom[address as usize & UNMIRRORED_MASK]
        }
    }

    pub fn store_prg(&mut self, address: u16, val: u8) {
        if address < 0x8000 {
            info!(
                "Attempted to write to addr {:X} with val {:X}",
                address, val
            );
        } else {
            // Cnrom only supports 3 bit wide registers for selecting banks
            self.chr_rom_offset = ((val as usize) & 0b11) * 0x2000;
        }
    }

    pub fn ld_chr(&self, address: u16, chr_rom: &[u8], chr_ram: &[u8]) -> u8 {
        if self.use_chr_ram {
            chr_ram[address as usize]
        } else {
            chr_rom[self.chr_rom_offset + address as usize]
        }
    }

    pub fn store_chr(&mut self, address: u16, val: u8, chr_ram: &mut Vec<u8>) {
        if self.use_chr_ram {
            chr_ram[address as usize] = val;
        } else {
            info!(
                "Attempt to store to cnrom address {:X} val {}",
                address, val
            );
        }
    }

    pub fn reset(&mut self) {
        self.chr_rom_offset = 0;
    }
}
