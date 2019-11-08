extern crate nes_emu;

mod utils;

use wasm_bindgen::prelude::*;
use nes_emu::NesEmulator;
use nes_emu::rom::load_rom;
use std::collections::HashMap;
use nes_emu::controller::Button;

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
#[derive(PartialEq, PartialOrd, Eq, Hash)]
pub struct KeyCode(usize);

#[wasm_bindgen]
impl KeyCode {
    pub fn new(val: usize) -> KeyCode {
        KeyCode { 0: val }
    }
}

#[wasm_bindgen]
pub struct EmuInterface {
    ctrl0: HashMap<KeyCode, Button>,
    // ctrl1: HashMap<KeyCode, Button>,
    nes_emu: NesEmulator,
}

#[wasm_bindgen]
impl EmuInterface {
    pub fn new(buffer: &[u8]) -> Result<EmuInterface, JsValue> {
        let rom = match load_rom(buffer) {
            Ok(r) => r,
            Err(e) => return Err(JsValue::from_str(&e.to_string())),
        };
        let mut button_map_one = HashMap::new();
        // W A S D for directions, F is A, G is B, T is Start, Y is Select
        // TODO: add loadable config from JS
        button_map_one.insert(KeyCode(87), Button::Up);
        button_map_one.insert(KeyCode(83), Button::Down);
        button_map_one.insert(KeyCode(65), Button::Left);
        button_map_one.insert(KeyCode(68), Button::Right);
        button_map_one.insert(KeyCode(70), Button::A);
        button_map_one.insert(KeyCode(71), Button::B);
        button_map_one.insert(KeyCode(84), Button::Start);
        button_map_one.insert(KeyCode(89), Button::Select);

        Ok(EmuInterface {
            nes_emu: NesEmulator::new(rom),
            ctrl0: button_map_one,
            //ctrl1: ,
        })
    }

    pub fn get_frame(&mut self) -> BufferStruct {
        let buffer = self.nes_emu.next_frame();
        BufferStruct { pointer: buffer.as_ptr(), length: buffer.len() }
    }

    pub fn set_button(&mut self, key: KeyCode, state: bool) {
        if let Some(button) = self.ctrl0.get(&key) {
            self.nes_emu.cpu.mmu.ctrl0.set_button_state(*button, state);
        }
    }
}
