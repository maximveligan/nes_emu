use ppu::Ppu;
use apu::Apu;
use mapper::Mapper;

const WRAM_START: u16 = 0x0000;
const WRAM_END: u16 = 0x1FFF;
const PPU_START: u16 = 0x2000;
const PPU_END: u16 = 0x3FFF;
const APU_START: u16 = 0x4000;
const APU_END: u16 = 0x401F;
const ROM_START: u16 = 0x4020;
const ROM_END: u16 = 0xFFFF;

pub struct MemManageUnit {
    pub ppu: Ppu,
    pub apu: Apu,
    pub ram: Ram,
    pub mapper: Mapper,
}

pub struct Ram([u8; 0xFFF]);

impl Ram {
    pub fn new() -> Ram {
        Ram { 0: [0; 0xFFF] }
    }

    fn load(&self, address: u16) -> u8 {
        self.0[address as usize]
    }

    fn store(&mut self, address: u16, val: u8) {
        self.0[address as usize] = val;
    }
}

impl MemManageUnit {
    pub fn new(mapper: Mapper) -> MemManageUnit {
        MemManageUnit {
            ppu: Ppu::new(),
            apu: Apu(),
            ram: Ram::new(),
            mapper: mapper,
        }
    }

    pub fn store_u8(&mut self, address: u16, val: u8) {
        match address {
            WRAM_START...WRAM_END => self.ram.store(address & 0x7FF, val),
            PPU_START...PPU_END => self.ppu.store((address - 0x2000) & 7, val),
            0x4016 => unimplemented!("Player1 controller"),
            0x4017 => unimplemented!("Player2 controller"),
            APU_START...APU_END => self.apu.store(address - 0x4000, val),
            ROM_START...ROM_END => println!(
                "Warning! Attempt to write to rom at address {:X}",
                address
            ),
            _ => panic!("Undefined load"),
        }
    }

    pub fn load_u8(&self, address: u16) -> u8 {
        match address {
            WRAM_START...WRAM_END => self.ram.load(address & 0x7FF),
            PPU_START...PPU_END => self.ppu.load((address - 0x2000) & 7),
            0x4016 => unimplemented!("Player1 controller"),
            0x4016 => unimplemented!("Player1 controller"),
            APU_START...APU_END => self.apu.load(address - 0x4000),
            ROM_START...ROM_END => self.mapper.load(address),
            _ => panic!("Undefined load"),
        }
    }

    pub fn load_u16(&self, address: u16) -> u16 {
        let l_byte = self.load_u8(address);
        let r_byte = self.load_u8(address + 1);
        (r_byte as u16) << 8 | (l_byte as u16)
    }
}
