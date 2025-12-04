use crate::mapper::MemType;
use crate::mmu::Ram;
use crate::ppu::PpuState;
use crate::rom::ScreenMode;
use anyhow::Result;
use bincode::error::DecodeError;
use bincode::error::EncodeError;
use cpu_6502::cpu::Registers;
use serde::Deserialize;
use serde::Serialize;
use std::io::Read;
use std::io::Write;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateFileError {
    #[error("Encountered an error while loading state: {0}")]
    LoadState(DecodeError),
    #[error("Encountered an error while saving state: {0}")]
    SaveState(EncodeError),
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub ppu_state: PpuState,
    pub screen_mode: ScreenMode,
    pub chr_ram: Vec<u8>,
    pub cpu_regs: Registers,
    pub mapper: MemType,
    pub ram: Ram,
}

impl State {
    pub fn save<T: Write>(&self, writer: &mut T) -> Result<()> {
        match bincode::serde::encode_into_std_write(
            &self,
            writer,
            bincode::config::standard(),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(StateFileError::SaveState(e).into()),
        }
    }

    pub fn load<T: Read>(reader: &mut T) -> Result<State> {
        match bincode::serde::decode_from_std_read(
            reader,
            bincode::config::standard(),
        ) {
            Ok(state) => Ok(state),
            Err(e) => Err(StateFileError::LoadState(e).into()),
        }
    }
}
