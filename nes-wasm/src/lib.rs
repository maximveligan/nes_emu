extern crate nes_emu;

mod utils;

use wasm_bindgen::prelude::*;
use nes_emu::NesEmulator;
use nes_emu::rom::load_rom;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct BufferStruct {
    pub pointer: *const u8,
    pub length: usize,
}

#[wasm_bindgen]
pub struct EmuInterface {
    // ctrl0
    // ctrl1
    nes_emu: NesEmulator,
}

#[wasm_bindgen]
impl EmuInterface {
    pub fn new(buffer: &[u8]) -> Result<EmuInterface, JsValue> {
        let rom = match load_rom(buffer) {
            Ok(r) => r,
            Err(e) => return Err(JsValue::from_str(&e.to_string())),
        };
        Ok(EmuInterface {
            nes_emu: NesEmulator::new(rom),
        })
    }

    pub fn get_frame(&mut self) -> BufferStruct {
        let buffer = self.nes_emu.next_frame();
        BufferStruct { pointer: buffer.as_ptr(), length: buffer.len() }
    }
}
