use std::io::Read;
use std::io::Write;
use mmu::Ram;
use cpu::Registers;
use rom::ScreenMode;
use mapper::MemType;
use failure::Error;
use serde::Serialize;
use serde::Deserialize;
use ppu::PpuState;

#[derive(Serialize, Deserialize)]
pub struct State {
    pub ppu_state: PpuState,
    pub screen_mode: ScreenMode,
    pub chr_ram: Vec<u8>,
    pub cpu_regs: Registers,
    pub mapper: MemType,
    pub ram: Ram,
}

#[derive(Debug, Fail)]
pub enum StateFileError {
    #[fail(display = "Unable to parse state from file: {}", _0)]
    ParseError(std::boxed::Box<bincode::ErrorKind>),
}

impl State {
    pub fn save<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
        match bincode::serialize_into(writer, &self) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from(StateFileError::ParseError(e))),
        }
    }

    pub fn load<T: Read>(reader: &mut T) -> Result<State, Error> {
        match bincode::deserialize_from(reader) {
            Ok(state) => Ok(state),
            Err(e) => Err(Error::from(StateFileError::ParseError(e))),
        }
    }
}
