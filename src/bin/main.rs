extern crate nes_emu;
use std::env;
use nes_emu::start_emulator;

fn main() {
    start_emulator(env::args().nth(1));
}
