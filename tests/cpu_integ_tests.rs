extern crate nes_emu;

use nes_emu::cpu::Cpu;
use nes_emu::apu::Apu;
use nes_emu::ppu::Ppu;
use nes_emu::rom::load_rom;
use nes_emu::mapper::Mapper;
use nes_emu::mmu::Mmu;
use nes_emu::mmu::Ram;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn run_nestest() {
    let rom = load_rom("./nes_test_roms/other/nestest.nes").expect("This is a hard coded good rom");

    let mapper = Rc::new(RefCell::new(Mapper::from_rom(rom)));
    let mut cpu = Cpu::new(Mmu::new(
        Apu::new(),
        Ram::new(),
        Ppu::new(mapper.clone()),
        mapper,
    ));
    cpu.regs.pc.set_addr(0xC000);
    let mut cycle_count: usize = 7;
    // TODO: Need to implement cycle checking and assertions per instruction
    // instead of just at the end
    loop {
        let cc = cpu.step();
        cycle_count += cc as usize;
        if cpu.regs.pc.get_addr() == 0xF7A5 {
            assert_eq!(cpu.regs.pc.get_addr(), 0xF7A5);
            assert_eq!(cpu.regs.acc, 0x11);
            assert_eq!(cpu.regs.x, 0xFF);
            assert_eq!(cpu.regs.y, 0x15);
            assert_eq!(cpu.regs.flags.as_byte(), 0x25);
            assert_eq!(cpu.regs.sp, 0xFB);
            assert_eq!(cycle_count, 26455);
            return;
        }
    }
}
