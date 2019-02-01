mod apu;
mod cpu;
mod cpu_const;
mod mmu;
mod ppu;

use cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    match cpu.step() {
        Ok(()) => println!("success"),
        Err(e) => println!("{:?}", e),
    }
}
