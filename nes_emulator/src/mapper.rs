use crate::mapper::axrom::*;
use crate::mapper::cnrom::*;
use crate::mapper::nrom::*;
use crate::mapper::sxrom::*;
use crate::mapper::txrom::*;
use crate::mapper::unrom::*;
use crate::rom::Rom;
use crate::rom::ScreenMode;
use serde::Deserialize;
use serde::Serialize;

pub mod axrom;
pub mod cnrom;
pub mod nrom;
pub mod sxrom;
pub mod txrom;
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
    Axrom(Axrom),
    Txrom(Txrom),
    Cnrom(Cnrom),
}

impl Mapper {
    pub fn from_rom(mut rom: Rom) -> Mapper {
        let mem_type = match rom.header.mapper {
            0 => {
                let use_chr_ram = !rom.chr_ram.is_empty();
                MemType::Nrom(Nrom::new(rom.prg_rom.len(), use_chr_ram))
            }
            1 => {
                rom.fill_prg_ram();
                let use_chr_ram = !rom.chr_ram.is_empty();
                let last_page_start = rom.prg_rom.len() - 0x4000;
                MemType::Sxrom(Sxrom::new(use_chr_ram, last_page_start))
            }
            2 => {
                let last_page_start = rom.prg_rom.len() - 0x4000;
                MemType::Unrom(Unrom::new(last_page_start))
            }
            3 => {
                let use_chr_ram = !rom.chr_ram.is_empty();
                MemType::Cnrom(Cnrom::new(rom.prg_rom.len(), use_chr_ram))
            }
            4 => {
                let last_page_start = rom.prg_rom.len() - 0x4000;
                let use_chr_ram = !rom.chr_ram.is_empty();
                MemType::Txrom(Txrom::new(use_chr_ram, last_page_start))
            }
            7 => {
                let last_page_start = rom.prg_rom.len() - 0x8000;
                MemType::Axrom(Axrom::new(last_page_start))
            }
            m => panic!("Mapper {} not supported", m),
        };
        Mapper { rom, mem_type }
    }

    pub fn ld_prg(&self, addr: u16) -> u8 {
        match self.mem_type {
            MemType::Nrom(ref nrom) => nrom.ld_prg(addr, &self.rom.prg_rom),
            MemType::Unrom(ref unrom) => unrom.ld_prg(addr, &self.rom.prg_rom),
            MemType::Sxrom(ref sxrom) => {
                sxrom.ld_prg(addr, &self.rom.prg_rom, &self.rom.prg_ram)
            }
            MemType::Axrom(ref axrom) => axrom.ld_prg(addr, &self.rom.prg_rom),
            MemType::Txrom(ref _txrom) => panic!("Txrom not ready yet"),
            MemType::Cnrom(ref cnrom) => cnrom.ld_prg(addr, &self.rom.prg_rom),
        }
    }

    pub fn ld_chr(&self, addr: u16) -> u8 {
        match self.mem_type {
            MemType::Nrom(ref nrom) => {
                nrom.ld_chr(addr, &self.rom.chr_rom, &self.rom.chr_ram)
            }
            MemType::Unrom(ref unrom) => unrom.ld_chr(addr, &self.rom.chr_ram),
            MemType::Sxrom(ref sxrom) => {
                sxrom.ld_chr(addr, &self.rom.chr_rom, &self.rom.chr_ram)
            }
            MemType::Axrom(ref axrom) => axrom.ld_chr(addr, &self.rom.chr_ram),
            MemType::Txrom(ref _txrom) => panic!("Txrom not ready yet"),
            MemType::Cnrom(ref cnrom) => {
                cnrom.ld_chr(addr, &self.rom.chr_rom, &self.rom.chr_ram)
            }
        }
    }

    pub fn store_prg(&mut self, addr: u16, val: u8) {
        match self.mem_type {
            MemType::Unrom(ref mut unrom) => unrom.store_prg(addr, val),
            MemType::Sxrom(ref mut sxrom) => {
                sxrom.store_prg(addr, val, &mut self.rom.prg_ram)
            }
            MemType::Nrom(ref nrom) => nrom.store_prg(addr, val),
            MemType::Axrom(ref mut axrom) => axrom.store_prg(addr, val),
            MemType::Txrom(ref _txrom) => panic!("Txrom not ready yet"),
            MemType::Cnrom(ref mut cnrom) => cnrom.store_prg(addr, val),
        }
    }

    pub fn store_chr(&mut self, addr: u16, val: u8) {
        match self.mem_type {
            MemType::Unrom(ref mut unrom) => {
                unrom.store_chr(addr, val, &mut self.rom.chr_ram)
            }
            MemType::Sxrom(ref mut sxrom) => {
                sxrom.store_chr(addr, val, &mut self.rom.chr_ram)
            }
            MemType::Nrom(ref mut nrom) => {
                nrom.store_chr(addr, val, &mut self.rom.chr_ram)
            }
            MemType::Axrom(ref mut axrom) => {
                axrom.store_chr(addr, val, &mut self.rom.chr_ram)
            }
            MemType::Txrom(ref _txrom) => panic!("Txrom not ready yet"),
            MemType::Cnrom(ref mut cnrom) => cnrom.store_prg(addr, val),
        }
    }

    pub fn get_mirroring(&self) -> &ScreenMode {
        match self.mem_type {
            MemType::Unrom(_) | MemType::Nrom(_) | MemType::Cnrom(_) => {
                &self.rom.header.screen
            }
            MemType::Sxrom(ref sxrom) => sxrom.get_mirroring(),
            MemType::Axrom(ref axrom) => axrom.get_mirroring(),
            MemType::Txrom(ref txrom) => txrom.get_mirroring(),
        }
    }

    pub fn reset(&mut self) {
        match self.mem_type {
            MemType::Nrom(_) => (),
            MemType::Unrom(ref mut unrom) => unrom.reset(),
            MemType::Sxrom(ref mut sxrom) => sxrom.reset(),
            MemType::Axrom(ref mut axrom) => axrom.reset(),
            MemType::Cnrom(ref mut cnrom) => cnrom.reset(),
            MemType::Txrom(ref mut _txrom) => panic!("Txrom not ready yet"),
        }
    }
}
