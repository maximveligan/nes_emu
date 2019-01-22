mod cpu;
mod mmu;
mod apu;
mod ppu;
mod cpu_const;

use cpu::Cpu;
use cpu::Registers;
use mmu::MemManageUnit;
use mmu::Ram;
use mmu::Rom;
use apu::Apu;
use ppu::Ppu;
use ppu::PRegisters;
use cpu::ProgramCounter;

fn main() {
    let mut cpu = Cpu {
        cycle_count: 0,
        regs: Registers {
            acc: 0,
            x: 0,
            y: 0,
            pc: ProgramCounter::new(0),
            sp: 0,
            flags: 0,
        },
        mem: MemManageUnit {
            ppu: Ppu {
                regs: PRegisters {
                    ppuctrl: 0,
                    ppumask: 0,
                    ppustatus: 0,
                    oamaddr: 0,
                    oamdata: 0,
                    ppuscroll: 0,
                    ppuaddr: 0,
                    ppudata: 0,
                    oamdma: 0,
                }
            },
            apu: Apu(),
            ram: Ram::new(),
            rom: Rom::new()
        }
    };
    cpu.decode_op(0xea);
}
