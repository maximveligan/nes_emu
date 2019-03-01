extern crate nom;
extern crate image;
extern crate sdl2;
extern crate nes_emu;

use nes_emu::cpu::Cpu;
use nes_emu::apu::Apu;
use nes_emu::ppu::Ppu;
use nes_emu::rom::RomType;
use nes_emu::rom::Region;
use nes_emu::rom::parse_rom;
use std::fs::File;
use std::env;
use std::io::Read;
use nes_emu::mapper::Mapper;
use nes_emu::mmu::Mmu;
use nes_emu::mmu::Ram;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn run_nestest() {
    let mut raw_bytes = Vec::new();
    let rom = match File::open("nes_rom_tests/nestest.nes") {
        Ok(mut rom) => {
            rom.read_to_end(&mut raw_bytes)
                .expect("Something went wrong while reading the rom");
            parse_rom(&raw_bytes).unwrap().1
        }
        Err(err) => {
            println!("Unable to open file: {}", err);
            return;
        }
    };

    let mut mapper = Rc::new(RefCell::new(Mapper::from_rom(rom)));
    let mut cpu = Cpu::new(Mmu::new(
        Apu::new(),
        Ram::new(),
        Ppu::new(mapper.clone()),
        mapper,
    ));
    cpu.regs.pc.set_addr(0xC000);
    let mut cycle_count: usize = 0;
    // TODO: Need to implement cycle checking and assertions per instruction
    // instead of just at the end
    loop {
        match cpu.step(true) {
            Ok(cc) => {
                cycle_count += cc as usize;
            }
            Err(last_op) => {
                assert_eq!(cpu.regs.pc.get_addr(), 0xE546);
                assert_eq!(cpu.regs.acc, 0x00);
                assert_eq!(cpu.regs.x, 0x03);
                assert_eq!(cpu.regs.y, 0x77);
                assert_eq!(cpu.regs.flags, 0x67);
                assert_eq!(cpu.regs.sp, 0xFB);
                break;
            }
        }
    }
}
