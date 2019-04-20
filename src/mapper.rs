use serde::Serialize;
use serde::Deserialize;
use rom::Rom;
use rom::ScreenMode;

const UNMIRRORED_MASK: usize = 0x7FFF;
const MIRRORED_MASK: usize = 0x3FFF;
const NROM_PRG_ROM_START: u16 = 0x8000;

const SIXTEEN_KB: usize = 0x4000;

pub struct Mapper {
    mem_type: MemType,
    rom: Rom,
}

#[derive(Serialize, Deserialize)]
pub enum MemType {
    Nrom(Nrom),
    Unrom(Unrom),
}

#[derive(Serialize, Deserialize)]
pub struct Unrom {
    bank_select: u8,
}

impl Unrom {
    fn new() -> Unrom {
        Unrom { bank_select: 0 }
    }

    fn store_prg(&mut self, address: u16, val: u8) {
        if address >= 0x8000 {
            self.bank_select = val & 0b111;
        }
    }

    fn ld_prg(&self, address: u16, prg_rom: &Vec<u8>) -> u8 {
        if address < 0x8000 {
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

    fn ld_chr(&self, address: u16, chr_ram: &Vec<u8>) -> u8 {
        chr_ram[address as usize]
    }

    fn store_chr(&mut self, address: u16, val: u8, chr_ram: &mut Vec<u8>) {
        chr_ram[address as usize] = val;
    }

    fn reset(&mut self) {
        self.bank_select = 0;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Nrom {
    mirrored: bool,
}

impl Nrom {
    fn new(prg_rom_size: usize) -> Nrom {
        Nrom {
            mirrored: prg_rom_size <= SIXTEEN_KB,
        }
    }

    fn ld_prg(&self, address: u16, prg_rom: &Vec<u8>) -> u8 {
        if address < NROM_PRG_ROM_START {
            0
        } else if self.mirrored {
            prg_rom[address as usize & MIRRORED_MASK]
        } else {
            prg_rom[address as usize & UNMIRRORED_MASK]
        }
    }

    fn store_prg(&mut self, address: u16, val: u8) {
        println!(
            "Warning! Cannot store to NROM prg rom! Addr {:X} Val {}",
            address, val
        );
    }

    fn ld_chr(&self, address: u16, chr_rom: &Vec<u8>) -> u8 {
        chr_rom[address as usize]
    }

    fn store_chr(&mut self, address: u16, val: u8) {
        println!(
            "Warning! Cannot store to NROM chr rom! Addr {:X} Val {}",
            address, val
        );
    }

    fn reset(&self) {}
}

impl Mapper {
    pub fn from_rom(rom: Rom) -> Mapper {
        let mem_type = match rom.header.mapper {
            0 => MemType::Nrom(Nrom::new(rom.prg_rom.len())),
            2 => MemType::Unrom(Unrom::new()),
            m => panic!("Mapper {} not supported", m),
        };
        Mapper {
            rom: rom,
            mem_type: mem_type,
        }
    }

    pub fn ld_prg(&self, address: u16) -> u8 {
        match self.mem_type {
            MemType::Nrom(ref nrom) => nrom.ld_prg(address, &self.rom.prg_rom),
            MemType::Unrom(ref unrom) => {
                unrom.ld_prg(address, &self.rom.prg_rom)
            }
        }
    }

    pub fn ld_chr(&self, address: u16) -> u8 {
        match self.mem_type {
            MemType::Nrom(ref nrom) => nrom.ld_chr(address, &self.rom.chr_rom),
            MemType::Unrom(ref unrom) => {
                unrom.ld_chr(address, &self.rom.chr_ram)
            }
        }
    }

    pub fn store_prg(&mut self, address: u16, val: u8) {
        match self.mem_type {
            MemType::Nrom(ref mut nrom) => nrom.store_prg(address, val),
            MemType::Unrom(ref mut unrom) => unrom.store_prg(address, val),
        }
    }

    pub fn store_chr(&mut self, address: u16, val: u8) {
        match self.mem_type {
            MemType::Nrom(ref mut nrom) => nrom.store_chr(address, val),
            MemType::Unrom(ref mut unrom) => {
                unrom.store_chr(address, val, &mut self.rom.chr_ram)
            }
        }
    }

    pub fn get_mirroring(&self) -> &ScreenMode {
        match self.mem_type {
            MemType::Nrom(_) | MemType::Unrom(_) => &self.rom.header.screen,
        }
    }

    pub fn reset(&mut self) {
        match self.mem_type {
            MemType::Nrom(ref nrom) => nrom.reset(),
            MemType::Unrom(ref mut unrom) => unrom.reset(),
        }
    }
}
