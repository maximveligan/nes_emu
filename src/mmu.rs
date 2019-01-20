use ppu::Ppu;
use apu::Apu;
use cpu::ProgramCounter;

pub struct MemManageUnit {
    ppu: Ppu,
    apu: Apu
}

impl MemManageUnit {
    pub fn store_u8(&self) {}
    pub fn store_u16(&self) {}
    pub fn load_u8(&self, index: u16) -> u8 {unimplemented!()}
    pub fn load_u16(&self, index: u16) -> u16 {unimplemented!()}
}
