use rom::Rom;

const NROM_PRG_RAM_START: u16 = 0x6000;
const NROM_PRG_RAM_END: u16 = 0x7FFF;
const NROM_PRG_ROM_START1: u16 = 0x8000;
const NROM_PRG_ROM_END1: u16 = 0xBFFF;
const NROM_PRG_ROM_START2: u16 = 0xC000;
const NROM_PRG_ROM_END2: u16 = 0xFFFF;

const SIXTEEN_KB: usize = 16384;

pub enum Mapper {
    Nrom(Nrom),
}

struct Nrom {
    battery_backed: bool,
    mirrored: bool,
    rom: Rom,
}

impl Nrom {
    fn new(rom: Rom) -> Nrom {
        Nrom {
            battery_backed: rom.header.save_ram,
            mirrored: rom.prg_rom.len() <= SIXTEEN_KB,
            rom: rom,
        }
    }

    fn load(&self, address: u16) -> Result<u8, String> {
        //TODO: Check if inclusive
        if address >= NROM_PRG_RAM_START && address <= NROM_PRG_RAM_END {
            if self.battery_backed {
                //TODO: Check address offset
                Ok(self.rom.prg_rom[address as usize])
            } else {
                Err(format!("Warning! Rom is not battery backed! Attempt to load from address {:X}", address))
            }
        } else if self.mirrored {
            if address >= NROM_PRG_ROM_START2 && address <= NROM_PRG_ROM_END2 {
                Ok(self.rom.prg_rom[(address - NROM_PRG_ROM_START1) as usize])
            } else if address >= NROM_PRG_ROM_START1
                && address <= NROM_PRG_ROM_START2
            {
                Ok(self.rom.prg_rom[address as usize])
            } else {
                Err(format!(
                    "Warning! Load attempt from outside of rom bounds! {:X}",
                    address
                ))
            }
        } else if address >= NROM_PRG_ROM_START1
            && address <= NROM_PRG_ROM_START2
        {
            Ok(self.rom.prg_rom[address as usize])
        } else {
            Err(format!(
                "Warning! Load attempt from outside of rom bounds! {:X}",
                address
            ))
        }
    }

    fn store(&mut self, address: u16, val: u8) -> Result<(), String> {
        if !self.battery_backed {
            Err(format!(
                "Attempted to write to ram without battery backing! {:X}",
                address
            ))
        } else if address >= NROM_PRG_RAM_START && address <= NROM_PRG_RAM_END {
            unimplemented!("Open save file and write to it");
            Ok(())
        } else {
            Err(format!(
                "Attempted to write outside of program ram range! {:X}",
                address
            ))
        }
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
            Mapper::Nrom(ref rom) => unimplemented!(),
        }
    }

    pub fn store(&self, addres: u16, val: u8) {
        match *self {
            Mapper::Nrom(ref rom) => unimplemented!(),
        }
    }
}
