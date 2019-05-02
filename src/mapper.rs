use serde::Serialize;
use serde::Deserialize;
use rom::Rom;
use rom::ScreenMode;
use mapper::sxrom::*;
use mapper::unrom::*;
use mapper::nrom::*;

pub mod sxrom;
pub mod nrom;
pub mod unrom;

pub struct Mapper {
    pub mem_type: MemType,
    pub rom: Rom,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MemType {
    Nrom(Nrom),
    Sxrom(Sxrom),
    Unrom(Unrom),
}

impl Mapper {
    pub fn from_rom(mut rom: Rom) -> Mapper {
        let mem_type = match rom.header.mapper {
            0 => {
                let use_chr_ram = rom.chr_ram.len() != 0;
                MemType::Nrom(Nrom::new(rom.prg_rom.len(), use_chr_ram))
            }
            1 => {
                rom.fill_prg_ram();
                let use_chr_ram = rom.chr_ram.len() != 0;
                let last_page_start = rom.prg_rom.len() - 0x4000;
                MemType::Sxrom(Sxrom::new(use_chr_ram, last_page_start))
            }
            2 => MemType::Unrom(Unrom::new()),
            m => panic!("Mapper {} not supported", m),
        };
        Mapper {
            rom: rom,
            mem_type: mem_type,
        }
    }

    pub fn ld_prg(&self, addr: u16) -> u8 {
        match self.mem_type {
            MemType::Nrom(ref nrom) => nrom.ld_prg(addr, &self.rom.prg_rom),
            MemType::Unrom(ref unrom) => unrom.ld_prg(addr, &self.rom.prg_rom),
            MemType::Sxrom(ref sxrom) => sxrom.ld_prg(addr, &self.rom.prg_rom, &self.rom.prg_ram),
        }
    }

    pub fn ld_chr(&self, addr: u16) -> u8 {
        match self.mem_type {
            MemType::Nrom(ref nrom) => nrom.ld_chr(addr, &self.rom.chr_rom, &self.rom.chr_ram),
            MemType::Unrom(ref unrom) => unrom.ld_chr(addr, &self.rom.chr_ram),
            MemType::Sxrom(ref sxrom) => sxrom.ld_chr(addr, &self.rom.chr_rom, &self.rom.chr_ram),
        }
    }

    pub fn store_prg(&mut self, addr: u16, val: u8) {
        match self.mem_type {
             MemType::Unrom(ref mut unrom) => unrom.store_prg(addr, val),
             MemType::Sxrom(ref mut sxrom) => sxrom.store_prg(addr, val, &mut self.rom.prg_ram),
             _ => ()
        }
    }

    pub fn store_chr(&mut self, addr: u16, val: u8) {
        match self.mem_type {
             MemType::Unrom(ref mut unrom) => unrom.store_chr(addr, val, &mut self.rom.chr_ram),
             MemType::Sxrom(ref mut sxrom) => sxrom.store_chr(addr, val, &mut self.rom.chr_ram),
             MemType::Nrom(ref mut nrom) => nrom.store_chr(addr, val, &mut self.rom.chr_ram),
        }
    }

    pub fn get_mirroring(&self) -> ScreenMode {
        match self.mem_type {
             MemType::Unrom(_) | MemType::Nrom(_) => self.rom.header.screen.clone(),
             MemType::Sxrom(ref sxrom) => sxrom.get_mirroring(),
        }
    }

    pub fn reset(&mut self) {
        match self.mem_type {
            MemType::Nrom(_) => (),
            MemType::Unrom(ref mut unrom) => unrom.reset(),
            MemType::Sxrom(ref mut sxrom) => sxrom.reset(),
        }
    }
}
