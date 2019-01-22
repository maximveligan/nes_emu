use ppu::Ppu;
use apu::Apu;

pub struct MemManageUnit {
    pub ppu: Ppu,
    pub apu: Apu,
    pub ram: Ram,
    pub rom: Rom
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

pub struct Rom([u8; 0xFFF]);

impl Rom {
    pub fn new() -> Rom {
        Rom { 0: [0; 0xFFF] }
    }
}

impl MemManageUnit {
    pub fn store_u8(&self) {}
    pub fn store_u16(&self) {}

    pub fn load_u8(&self, address: u16) -> u8 {
        match address {
            0x0000...0x1FFF => self.ram.load(address & 0x7FF),
            0x2000...0x3FFF => self.ppu.load((address - 0x2000) % 8),
            0x4016 => unimplemented!("Player1 controller"),
            0x4016 => unimplemented!("Player1 controller"),
            0x4000...0x401F => self.apu.load(address - 0x4000),
            0x4020...0xFFFF => unimplemented!("ROM reads/writes"),
            _ => unimplemented!("Undefined load")
        }
    }

    pub fn load_u16(&self, address: u16) -> u16 {
        let l_byte = self.load_u8(address);
        let r_byte = self.load_u8(address + 1);
        (r_byte as u16) << 8 | (l_byte as u16)
    }
}
