#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate bitfield;
pub mod cpu;
pub mod cpu_const;

pub trait Memory {
    fn ld8(&mut self, addr: u16) -> u8;
    fn ld16(&mut self, addr: u16) -> u16;
    fn store(&mut self, addr: u16, val: u8);
}
