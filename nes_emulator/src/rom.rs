use failure::Error;
use nom::*;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

const PRG_ROM_PAGE_SIZE: usize = 16384;
const PRG_RAM_PAGE_SIZE: usize = 8192;
const CHR_ROM_PAGE_SIZE: usize = 8192;
const CHR_RAM_PAGE_SIZE: usize = 8192;
const TRAINER_LEN: usize = 512;

#[derive(Debug, failure::Fail)]
pub enum LoadRomError {
    #[fail(display = "Rom not supported: {}", _0)]
    Unsupported(String),
    #[fail(display = "Parse error: invalid rom")]
    ParseError,
}

fn parse_rom(src: &[u8]) -> IResult<&[u8], Rom> {
    do_parse!(
        src,
        tag!(b"NES\x1A")
            >> prg_pgs: be_u8
            >> chr_pgs: be_u8
            >> flag6: be_u8
            >> flag7: be_u8
            >> prg_ram_pgs: be_u8
            >> flag9: be_u8
            >> flag10: be_u8
            >> take!(5)
            >> cond!((flag6 & 0b100) != 0, take!(TRAINER_LEN))
            >> prg_rom: take!(prg_pgs as usize * PRG_ROM_PAGE_SIZE)
            >> chr_rom: take!(chr_pgs as usize * CHR_ROM_PAGE_SIZE)
            >> (Rom {
                header: Header {
                    mapper: flag7 & 0xF0 | ((flag6 & 0xF0) >> 4),
                    screen: if flag6 & 0b1000 != 0 {
                        ScreenMode::FourScreen
                    } else if flag6 & 0b01 == 1 {
                        ScreenMode::Vertical
                    } else {
                        ScreenMode::Horizontal
                    },
                    save_ram: flag6 & 0b10 != 0,
                    vs_unisystem: flag7 & 0b01 == 1,
                    playchoice10: flag7 & 0b10 != 0,
                    region: if flag9 & 0b01 == 1 {
                        Region::PAL
                    } else {
                        Region::NTSC
                    },
                    flag10,
                    rom_type: if flag7 & 0b1100 == 0b1000 {
                        RomType::Nes2
                    } else {
                        RomType::INes
                    },
                },
                prg_rom: prg_rom.into(),
                chr_rom: chr_rom.into(),
                prg_ram_size: if prg_ram_pgs != 0 {
                    PRG_RAM_PAGE_SIZE * prg_ram_pgs as usize
                } else {
                    PRG_RAM_PAGE_SIZE as usize
                },
                prg_ram: Vec::new(),
                chr_ram: if chr_pgs == 0 {
                    vec![0; CHR_RAM_PAGE_SIZE]
                } else {
                    Vec::new()
                },
            })
    )
}

pub fn load_rom(rom_bytes: &[u8]) -> Result<Rom, Error> {
    let rom = match parse_rom(rom_bytes) {
        Ok((_, rom)) => rom,
        Err(e) => {
            log::debug!("Nom parse error message {}", e.to_string());
            return Err(Error::from(LoadRomError::ParseError));
        }
    };

    rom.check_invalid()?;
    Ok(rom)
}

// Almost no roms use flag10, as such pulled as u8
pub struct Header {
    pub rom_type: RomType,
    pub mapper: u8,
    pub screen: ScreenMode,
    pub save_ram: bool,
    vs_unisystem: bool,
    playchoice10: bool,
    pub region: Region,
    flag10: u8,
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Header:\n\
             Type-{:?}, Mapper-{}, ScreenMode-{:?}, SRAM-{}\n\
             VS Unisystem-{}, Playchoice10-{}, Region-{:?}, flag10-{}\n",
            self.rom_type,
            self.mapper,
            self.screen,
            self.save_ram,
            self.vs_unisystem,
            self.playchoice10,
            self.region,
            self.flag10
        )
    }
}

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub chr_ram: Vec<u8>,
    prg_ram_size: usize,
    pub header: Header,
}

impl Rom {
    fn check_invalid(&self) -> Result<(), LoadRomError> {
        if self.header.rom_type == RomType::Nes2 {
            return Err(LoadRomError::Unsupported(
                "Unsupported rom type NES2.0!".to_string(),
            ));
        }

        if self.header.region == Region::PAL {
            return Err(LoadRomError::Unsupported(
                "Unsupported region PAL!".to_string(),
            ));
        }

        Ok(())
    }

    pub fn fill_prg_ram(&mut self) {
        self.prg_ram = vec![0u8; self.prg_ram_size];
    }
}

impl fmt::Debug for Rom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}\
             Prg Rom Size (kb) {}\n\
             Prg Ram Size (kb) {}\n\
             Chr Rom Size (kb) {}\n\
             Chr Ram Size (kb) {}",
            self.header,
            self.prg_rom.len() / 1024,
            self.prg_ram_size / 1024,
            self.chr_rom.len() / 1024,
            self.chr_ram.len() / 1024,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScreenMode {
    FourScreen,
    Vertical,
    Horizontal,
    OneScreenSwap(ScreenBank),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScreenBank {
    Lower,
    Upper,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RomType {
    INes,
    Nes2,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Region {
    NTSC,
    PAL,
}
