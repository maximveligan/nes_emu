use rom::Rom;

const UNMIRRORED_MASK: usize = 0x7FFF;
const MIRRORED_MASK: usize = 0x3FFF;
const NROM_PRG_ROM_START: u16 = 0x8000;

const SIXTEEN_KB: usize = 16384;

pub enum Mapper {
    Nrom(Nrom),
}

pub struct Nrom {
    mirrored: bool,
    rom: Rom,
}

impl Nrom {
    fn new(rom: Rom) -> Nrom {
        Nrom {
            mirrored: rom.prg_rom.len() <= SIXTEEN_KB,
            rom: rom,
        }
    }

    fn load(&self, address: u16) -> u8 {
        if address < NROM_PRG_ROM_START {
            0
        } else if self.mirrored {
            self.rom.prg_rom[address as usize & MIRRORED_MASK]
        } else {
            self.rom.prg_rom[address as usize & UNMIRRORED_MASK]
        }
    }

    fn store(&mut self, address: u16, val: u8) {
        println!(
            "Warning! Cannot store to NROM! Addr {:X} Val {}",
            address, val
        );
    }
}

impl Mapper {
    pub fn from_rom(rom: Rom) -> Mapper {
        match rom.header.mapper {
            0 => Mapper::Nrom(Nrom::new(rom)),
            _ => unimplemented!("Other mappers not supported"),
        }
    }

    pub fn load(&self, address: u16) -> u8 {
        match *self {
            Mapper::Nrom(ref nrom) => nrom.load(address),
        }
    }

    pub fn store(&mut self, address: u16, val: u8) {
        match *self {
            Mapper::Nrom(ref mut nrom) => nrom.store(address, val),
        }
    }
}
