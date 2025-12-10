use crate::Memory;
use crate::cpu_const::*;
use log::Level;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Registers {
    pub acc: u8,
    pub x: u8,
    pub y: u8,
    pub pc: ProgramCounter,
    pub sp: u8,
    pub flags: Flags,
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} A:{:02X} X:{:02X} Y:{:02X} Flags:{:02X} SP:{:02X}",
            self.pc,
            self.acc,
            self.x,
            self.y,
            self.flags.as_byte(),
            self.sp
        )
    }
}

impl Registers {
    fn reset(&mut self, address: u16) {
        // According to the cpu reset registers test, SP should decrement by 3
        // and the interrupt flag should be set
        self.pc.set_addr(address);
        self.sp -= 3;
        self.flags.set_itr(true);
    }

    pub fn from_values(acc: u8, x: u8, y: u8, pc: u16, sp: u8, flags: u8) -> Self {
        Registers {
            acc,
            x,
            y,
            pc: ProgramCounter(pc),
            sp,
            flags: Flags(flags),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ProgramCounter(u16);

impl ProgramCounter {
    pub fn new(val: u16) -> ProgramCounter {
        ProgramCounter { 0: val }
    }

    fn add_unsigned(&mut self, offset: u16) {
        self.0 = self.0.wrapping_add(offset);
    }

    fn add_signed(&mut self, offset: i8) {
        self.0 = (self.0 as i32 + offset as i32) as u16;
    }

    pub fn set_addr(&mut self, addr: u16) {
        self.0 = addr;
    }
    pub fn get_addr(&self) -> u16 {
        self.0
    }
}

impl fmt::Debug for ProgramCounter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PC:{:04X}", self.0)
    }
}

bitfield! {
    #[derive(Serialize, Deserialize, Copy, Clone, Debug)]
    pub struct Flags(u8);
    carry, set_carry:       0;
    zero, set_zero:         1;
    itr, set_itr:           2;
    dec, set_dec:           3;
    brk, set_brk:           4;
    unused, set_unused:     5;
    overflow, set_overflow: 6;
    neg, set_neg:           7;
    pub as_byte, set_byte:      7, 0;
}

pub struct Cpu {
    pub regs: Registers,
    pub delta_cycles: usize,
    total_cycles: usize,
}

#[derive(Clone)]
pub enum Mode {
    Imm,
    ZP,
    ZPX,
    ZPY,
    Abs,
    AbsX,
    NoPBAbsX,
    AbsY,
    NoPBAbsY,
    JmpIndir,
    IndX,
    IndY,
    NoPBIndY,
}

impl Cpu {
    pub fn new<M: Memory>(mem: &mut M) -> Cpu {
        let mut cpu = Cpu {
            delta_cycles: 0,
            total_cycles: 0,
            regs: Registers {
                acc: 0,
                x: 0,
                y: 0,
                pc: ProgramCounter::new(0),
                sp: 0xFD,
                flags: Flags(0b00100100),
            },
        };
        cpu.regs.pc.set_addr(mem.ld16(RESET_VEC));
        cpu
    }

    pub fn from_registers(regs: Registers) -> Cpu {
        Cpu {
            delta_cycles: 0,
            total_cycles: 0,
            regs,
        }
    }

    pub fn reset<M: Memory>(&mut self, mem: &mut M) {
        self.delta_cycles = 0;
        self.total_cycles = 0;
        let addr = mem.ld16(RESET_VEC);
        self.regs.reset(addr);
    }

    fn check_pb<M: Memory>(&mut self, low: u8, high: u8, offset: u8, mem: &mut M) -> u16 {
        let (new_low, overflowed) = low.overflowing_add(offset);
        if overflowed {
            mem.ld8((high as u16) << 8 | new_low as u16);
            self.incr_cc();
        }
        ((high as u16) << 8 | (low as u16)) + offset as u16
    }

    fn incr_cc(&mut self) {
        self.delta_cycles += 1;
    }

    fn address_mem<M: Memory>(&mut self, mode: Mode, mem: &mut M) -> u16 {
        match mode {
            Mode::Imm => {
                let tmp = self.regs.pc.get_addr();
                self.regs.pc.add_unsigned(1);
                tmp
            }
            Mode::ZP => self.ld8_pc_up(mem) as u16,
            Mode::ZPX => {
                let tmp = self.ld8_pc_up(mem);
                mem.ld8(tmp as u16);
                tmp.wrapping_add(self.regs.x) as u16
            }
            Mode::ZPY => {
                let tmp = self.ld8_pc_up(mem);
                mem.ld8(tmp as u16);
                tmp.wrapping_add(self.regs.y) as u16
            }
            Mode::Abs => self.ld16_pc_up(mem),
            Mode::AbsX => {
                let low = self.ld8_pc_up(mem);
                let high = self.ld8_pc_up(mem);
                self.check_pb(low, high, self.regs.x, mem)
            }
            Mode::AbsY => {
                let low = self.ld8_pc_up(mem);
                let high = self.ld8_pc_up(mem);
                self.check_pb(low, high, self.regs.y, mem)
            }
            Mode::NoPBAbsX => {
                let low = self.ld8_pc_up(mem);
                let high = self.ld8_pc_up(mem);
                let (over_low, _) = low.overflowing_add(self.regs.x);
                mem.ld8((high as u16) << 8 | over_low as u16);
                ((high as u16) << 8 | low as u16) + self.regs.x as u16
            }
            Mode::NoPBAbsY => {
                let low = self.ld8_pc_up(mem);
                let high = self.ld8_pc_up(mem);
                let (over_low, _) = low.overflowing_add(self.regs.y);
                mem.ld8((high as u16) << 8 | over_low as u16);
                ((high as u16) << 8 | low as u16) + self.regs.y as u16
            }
            Mode::JmpIndir => {
                let tmp = self.ld16_pc_up(mem);
                let low = mem.ld8(tmp);
                let high: u8 = if tmp & 0xFF == 0xFF {
                    mem.ld8(tmp - 0xFF)
                } else {
                    mem.ld8(tmp + 1)
                };
                (high as u16) << 8 | (low as u16)
            }
            Mode::IndX => {
                let tmp = self.ld8_pc_up(mem);
                mem.ld8(tmp as u16);
                let base_address = tmp.wrapping_add(self.regs.x) as u16;
                if base_address == 0xFF {
                    (mem.ld8(base_address) as u16) | (mem.ld8(0) as u16) << 8
                } else {
                    mem.ld16(base_address)
                }
            }
            Mode::IndY => {
                let base = self.ld8_pc_up(mem);
                let tmp = if base == 0xFF {
                    (mem.ld8(0xFF) as u16) | (mem.ld8(0) as u16) << 8
                } else {
                    mem.ld16(base as u16)
                };
                self.check_pb(tmp as u8, (tmp >> 8) as u8, self.regs.y, mem)
            }
            Mode::NoPBIndY => {
                let base = self.ld8_pc_up(mem);
                let tmp = if base == 0xFF {
                    (mem.ld8(0xFF) as u16) | (mem.ld8(0) as u16) << 8
                } else {
                    mem.ld16(base as u16)
                };
                let low = (tmp & 0xFF) as u8;
                mem.ld8((tmp & 0xFF00) | low.wrapping_add(self.regs.y) as u16);
                tmp.wrapping_add(self.regs.y as u16)
            }
        }
    }

    pub fn proc_nmi<M: Memory>(&mut self, mem: &mut M) {
        let flags = self.regs.flags;
        self.push_pc(mem);
        self.push(flags.as_byte(), mem);
        self.regs.pc.set_addr(mem.ld16(NMI_VEC));
    }

    fn read_op<M: Memory>(&mut self, mode: Mode, mem: &mut M) -> u8 {
        let addr = self.address_mem(mode, mem);
        mem.ld8(addr)
    }

    fn write_dma<M: Memory>(&mut self, high_nyb: u8, mem: &mut M) {
        // self.delta_cycles += 513 + (self.delta_cycles % 2);
        let page_num = (high_nyb as u16) << 8;
        for address in page_num..=page_num + 0xFF {
            let tmp = mem.ld8(address);
            mem.store(OAM_DATA, tmp);
        }
    }

    fn store<M: Memory>(&mut self, addr: u16, val: u8, mem: &mut M) {
        mem.store(addr, val);
        if addr == DMA_ADDR {
            self.write_dma(val, mem);
        }
    }

    fn and<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        let tmp = self.regs.acc & val;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn ora<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        let tmp = self.regs.acc | val;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn eor<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        let tmp = self.regs.acc ^ val;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn adc_val(&mut self, val: u8) {
        let acc = self.regs.acc;
        let tmp = acc as u16 + val as u16 + self.regs.flags.carry() as u16;
        self.regs.flags.set_carry(tmp > 0xFF);
        self.regs
            .flags
            .set_overflow(((acc as u16 ^ tmp) & (val as u16 ^ tmp) & 0x80) != 0);
        let tmp = tmp as u8;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn adc<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        self.adc_val(val);
    }

    fn sbc<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        self.adc_val(val ^ 0xFF);
    }

    fn lda<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        self.regs.acc = val;
        self.set_zero_neg(val);
    }

    fn ldx<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        self.regs.x = val;
        self.set_zero_neg(val);
    }

    fn ldy<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        self.regs.y = val;
        self.set_zero_neg(val);
    }

    fn ror_acc(&mut self) {
        let (tmp, n_flag) = Cpu::get_ror(self.regs.flags.carry(), self.regs.acc);
        self.regs.flags.set_carry(n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn ror_addr<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let val = mem.ld8(addr);
        let (tmp, n_flag) = Cpu::get_ror(self.regs.flags.carry(), val);
        self.store(addr, val, mem);
        self.regs.flags.set_carry(n_flag);
        self.set_zero_neg(tmp);
        self.store(addr, tmp, mem);
    }

    fn get_ror(carry_flag: bool, val: u8) -> (u8, bool) {
        ((val >> 1) | ((carry_flag as u8) << 7), (val & 0b01) != 0)
    }

    fn rol_acc(&mut self) {
        let (tmp, n_flag) = Cpu::get_rol(self.regs.flags.carry(), self.regs.acc);
        self.regs.flags.set_carry(n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn rol_addr<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let val = mem.ld8(addr);
        mem.store(addr, val);
        let (tmp, n_flag) = Cpu::get_rol(self.regs.flags.carry(), val);
        self.regs.flags.set_carry(n_flag);
        self.set_zero_neg(tmp);
        self.store(addr, tmp, mem);
    }

    fn get_rol(carry_flag: bool, val: u8) -> (u8, bool) {
        ((val << 1) | (carry_flag as u8), (val & 0x80) != 0)
    }

    fn asl_acc(&mut self) {
        let acc = self.regs.acc;
        self.regs.flags.set_carry((acc >> 7) != 0);
        let tmp = acc << 1;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn asl_addr<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let val = mem.ld8(addr);
        self.store(addr, val, mem);
        self.regs.flags.set_carry((val >> 7) != 0);
        let tmp = val << 1;
        self.set_zero_neg(tmp);
        self.store(addr, tmp, mem);
    }

    fn lsr_acc(&mut self) {
        let acc = self.regs.acc;
        self.regs.flags.set_carry((acc & 0b01) != 0);
        let tmp = acc >> 1;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn lsr_addr<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let val = mem.ld8(addr);
        self.store(addr, val, mem);
        self.regs.flags.set_carry((val & 0b01) != 0);
        let tmp = val >> 1;
        self.set_zero_neg(tmp);
        self.store(addr, tmp, mem);
    }

    fn cpx<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        let tmp = self.regs.x as i16 - val as i16;
        self.regs.flags.set_carry(tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn cpy<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        let tmp = self.regs.y as i16 - val as i16;
        self.regs.flags.set_carry(tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn cmp<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        let tmp = self.regs.acc as i16 - val as i16;
        self.regs.flags.set_carry(tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn generic_branch<M: Memory>(&mut self, flag: bool, mem: &mut M) {
        let val = self.ld8_pc_up(mem) as i8;
        if flag {
            let addr = self.regs.pc.get_addr();
            mem.ld8(addr);
            let low = (addr & 0x00FF) as u8;

            let overflow_low;
            let carry;

            if val >= 0 {
                let us_v = val as u8;
                (overflow_low, carry) = low.overflowing_add(us_v);
            } else {
                let us_v = (val * -1) as u8;
                (overflow_low, carry) = low.overflowing_sub(us_v);
            }

            if carry {
                self.incr_cc();
                mem.ld8((addr & 0xFF00) | overflow_low as u16);
            }

            self.regs.pc.add_signed(val as i8);
            self.incr_cc();
            let new_addr = self.regs.pc.get_addr();
            mem.ld8(new_addr);
        }
    }

    fn bit<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        let acc = self.regs.acc;
        self.regs.flags.set_zero((val & acc) == 0);
        self.regs.flags.set_overflow((val & 0x40) != 0);
        self.regs.flags.set_neg((val & 0x80) != 0);
    }

    fn dec<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let val: u8 = mem.ld8(addr).wrapping_sub(1);
        self.set_zero_neg(val);
        self.store(addr, val + 1, mem);
        self.store(addr, val, mem);
    }

    fn inc<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let val: u8 = mem.ld8(addr).wrapping_add(1);
        self.set_zero_neg(val);
        self.store(addr, val - 1, mem);
        self.store(addr, val, mem);
    }

    fn sta<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let tmp = self.regs.acc;
        self.store(addr, tmp, mem);
    }

    fn stx<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let tmp = self.regs.x;
        self.store(addr, tmp, mem);
    }

    fn sty<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let tmp = self.regs.y;
        self.store(addr, tmp, mem);
    }

    fn jmp<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        self.regs.pc.set_addr(addr);
    }

    fn aac<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        let acc = self.regs.acc;
        self.regs.acc = acc & val;
        self.set_zero_neg(self.regs.acc);
        self.regs.flags.set_carry(self.regs.flags.neg());
    }

    fn aax<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let tmp = self.regs.acc & self.regs.x;
        self.store(addr, tmp, mem);
    }

    fn arr<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        self.and(mode, mem);
        let (acc, _) = Cpu::get_ror(self.regs.flags.carry(), self.regs.acc);
        self.regs.acc = acc;
        let b5 = ((acc >> 5) & 1) == 1;
        let b6 = ((acc >> 6) & 1) == 1;
        self.regs.flags.set_carry(b6);
        self.regs.flags.set_overflow(b5 ^ b6);
        self.set_zero_neg(self.regs.acc);
    }

    fn alr<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        self.and(mode, mem);
        self.lsr_acc();
    }

    fn lax<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        self.regs.acc = val;
        self.regs.x = val;
        self.set_zero_neg(val);
    }

    fn axs<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let val = self.read_op(mode, mem);
        let tmp = self.regs.x & self.regs.acc;
        let (tmp, carry) = tmp.overflowing_sub(val);
        // No idea why this is !carry
        self.regs.flags.set_carry(!carry);
        self.regs.x = tmp;
        self.set_zero_neg(tmp);
    }

    //TODO this is dec followed by cmp, refactor this to use those functions
    fn dcp<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let val: u8 = mem.ld8(addr).wrapping_sub(1);
        self.set_zero_neg(val);
        self.store(addr, val, mem);
        let tmp = self.regs.acc as i16 - val as i16;
        self.regs.flags.set_carry(tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    //TODO This one can also probably be refactored
    fn isc<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let val: u8 = mem.ld8(addr).wrapping_add(1);
        self.set_zero_neg(val);
        self.store(addr, val, mem);
        self.adc_val(val ^ 0xFF);
    }

    //TODO same as this one
    fn slo<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let val = mem.ld8(addr);
        self.regs.flags.set_carry((val >> 7) != 0);
        let tmp = val << 1;
        self.store(addr, tmp, mem);

        let tmp = self.regs.acc | tmp;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn rla<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let (tmp, n_flag) = Cpu::get_rol(self.regs.flags.carry(), mem.ld8(addr));
        self.regs.flags.set_carry(n_flag);
        self.store(addr, tmp, mem);

        let tmp = self.regs.acc & tmp;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn sre<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let val = mem.ld8(addr);
        self.regs.flags.set_carry((val & 0b01) != 0);
        let tmp = val >> 1;
        self.store(addr, tmp, mem);

        let tmp = self.regs.acc ^ tmp;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn rra<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        let addr = self.address_mem(mode, mem);
        let (tmp, n_flag) = Cpu::get_ror(self.regs.flags.carry(), mem.ld8(addr));
        self.regs.flags.set_carry(n_flag);
        self.set_zero_neg(tmp);
        self.store(addr, tmp, mem);
        self.adc_val(tmp);
    }

    fn tax(&mut self) {
        let acc = self.regs.acc;
        self.regs.x = acc;
        self.set_zero_neg(acc);
    }

    fn atx<M: Memory>(&mut self, mode: Mode, mem: &mut M) {
        self.lda(mode, mem);
        self.tax();
    }

    fn sna<M: Memory>(&mut self, mode: Mode, and_reg: u8, addr_mode_reg: u8, mem: &mut M) {
        let mut addr = self.address_mem(mode, mem);
        if (addr - addr_mode_reg as u16) & 0xFF00 != addr & 0xFF00 {
            addr &= (and_reg as u16) << 8;
        }
        self.store(addr, and_reg & (((addr >> 8) as u8).wrapping_add(1)), mem);
    }

    fn brk<M: Memory>(&mut self, mem: &mut M) {
        // Dummy read
        mem.ld8(self.regs.pc.get_addr());
        self.regs.pc.add_signed(1);
        self.push_pc(mem);
        self.push(self.regs.flags.as_byte() | 0b10000, mem);
        self.regs.flags.set_itr(true);
        self.regs.pc.set_addr(mem.ld16(IRQ_VEC));
    }

    fn rts<M: Memory>(&mut self, mem: &mut M) {
        mem.ld8(self.regs.sp as u16 | 0x100);
        self.pull_pc(mem);
        mem.ld8(self.regs.pc.get_addr());
        self.regs.pc.add_unsigned(1);
    }

    fn rti<M: Memory>(&mut self, mem: &mut M) {
        self.pull_status(mem);
        self.pull_pc(mem);
    }

    fn jsr<M: Memory>(&mut self, mem: &mut M) {
        let low = self.ld8_pc_up(mem);
        mem.ld8(self.regs.sp as u16 | 0x100);
        self.push_pc(mem);
        let high = self.ld8_pc_up(mem);
        self.regs.pc.set_addr((high as u16) << 8 | low as u16);
    }

    fn pla<M: Memory>(&mut self, mem: &mut M) {
        mem.ld8(self.regs.sp as u16 | 0x100);
        let acc = self.pop(mem);
        self.regs.acc = acc;
        self.set_zero_neg(acc);
    }

    fn txa(&mut self) {
        let x = self.regs.x;
        self.regs.acc = x;
        self.set_zero_neg(x);
    }
    fn tay(&mut self) {
        let acc = self.regs.acc;
        self.regs.y = acc;
        self.set_zero_neg(acc);
    }
    fn tya(&mut self) {
        let y = self.regs.y;
        self.regs.acc = y;
        self.set_zero_neg(y);
    }
    fn dex(&mut self) {
        self.regs.x = self.regs.x.wrapping_sub(1);
        self.set_zero_neg(self.regs.x);
    }
    fn inx(&mut self) {
        self.regs.x = self.regs.x.wrapping_add(1);
        self.set_zero_neg(self.regs.x);
    }
    fn dey(&mut self) {
        self.regs.y = self.regs.y.wrapping_sub(1);
        self.set_zero_neg(self.regs.y);
    }
    fn iny(&mut self) {
        self.regs.y = self.regs.y.wrapping_add(1);
        self.set_zero_neg(self.regs.y);
    }
    fn tsx(&mut self) {
        self.regs.x = self.regs.sp;
        self.set_zero_neg(self.regs.sp);
    }

    fn push<M: Memory>(&mut self, val: u8, mem: &mut M) {
        let addr = self.regs.sp as u16 | 0x100;
        self.store(addr, val, mem);
        self.regs.sp -= 1;
    }

    fn pop<M: Memory>(&mut self, mem: &mut M) -> u8 {
        self.regs.sp += 1;
        mem.ld8(self.regs.sp as u16 | 0x100)
    }

    fn pull_pc<M: Memory>(&mut self, mem: &mut M) {
        let low = self.pop(mem);
        let high = self.pop(mem);
        self.regs.pc.set_addr(((high as u16) << 8) | low as u16);
    }

    fn pull_status<M: Memory>(&mut self, mem: &mut M) {
        mem.ld8(self.regs.sp as u16 | 0x100);
        let tmp = self.pop(mem);
        self.regs.flags.set_byte(tmp);
        self.regs.flags.set_unused(true);
        self.regs.flags.set_brk(false);
    }

    fn push_pc<M: Memory>(&mut self, mem: &mut M) {
        let high = self.regs.pc.get_addr() >> 8;
        let low = self.regs.pc.get_addr();
        self.push(high as u8, mem);
        self.push(low as u8, mem);
    }

    fn set_zero_neg(&mut self, val: u8) {
        self.regs.flags.set_neg(val >> 7 == 1);
        self.regs.flags.set_zero(val == 0);
    }

    pub fn step<M: Memory>(&mut self, mem: &mut M) -> usize {
        let byte = self.ld8_pc_up(mem);
        self.delta_cycles = 0;
        self.delta_cycles += CYCLES[byte as usize] as usize;
        self.execute_op(byte, mem);
        if log_enabled!(Level::Debug) {
            debug!(
                "INST: {:X} {:?} CYC:{}",
                byte,
                self.regs.clone(),
                self.total_cycles
            );
        }
        self.total_cycles += self.delta_cycles as usize;
        self.delta_cycles
    }

    fn ld8_pc_up<M: Memory>(&mut self, mem: &mut M) -> u8 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add_unsigned(1);
        mem.ld8(ram_ptr)
    }

    fn ld16_pc_up<M: Memory>(&mut self, mem: &mut M) -> u16 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add_unsigned(2);
        let tmp = mem.ld16(ram_ptr);
        tmp
    }

    pub fn execute_op<M: Memory>(&mut self, op: u8, mem: &mut M) {
        match op {
            INC_ABSX => self.inc(Mode::NoPBAbsX, mem),
            DEC_ABSX => self.dec(Mode::NoPBAbsX, mem),
            STA_ABSX => self.sta(Mode::NoPBAbsX, mem),
            ROR_ABSX => self.ror_addr(Mode::NoPBAbsX, mem),
            LSR_ABSX => self.lsr_addr(Mode::NoPBAbsX, mem),
            ROL_ABSX => self.rol_addr(Mode::NoPBAbsX, mem),
            0xDF => self.dcp(Mode::NoPBAbsX, mem),
            ASL_ABSX => self.asl_addr(Mode::NoPBAbsX, mem),
            0x7F => self.rra(Mode::NoPBAbsX, mem),
            0x3F => self.rla(Mode::NoPBAbsX, mem),
            0x1F => self.slo(Mode::NoPBAbsX, mem),
            0xFF => self.isc(Mode::NoPBAbsX, mem),
            0x5F => self.sre(Mode::NoPBAbsX, mem),

            0xDB => self.dcp(Mode::NoPBAbsY, mem),
            STA_ABSY => self.sta(Mode::NoPBAbsY, mem),
            0xFB => self.isc(Mode::NoPBAbsY, mem),
            0x1B => self.slo(Mode::NoPBAbsY, mem),
            0x3B => self.rla(Mode::NoPBAbsY, mem),
            0x5B => self.sre(Mode::NoPBAbsY, mem),
            0x7B => self.rra(Mode::NoPBAbsY, mem),

            INC_ZPX => self.inc(Mode::ZPX, mem),
            SBC_ZPX => self.sbc(Mode::ZPX, mem),
            DEC_ZPX => self.dec(Mode::ZPX, mem),
            CMP_ZPX => self.cmp(Mode::ZPX, mem),
            LDA_ZPX => self.lda(Mode::ZPX, mem),
            LDY_ZPX => self.ldy(Mode::ZPX, mem),
            STA_ZPX => self.sta(Mode::ZPX, mem),
            STY_ZPX => self.sty(Mode::ZPX, mem),
            ADC_ZPX => self.adc(Mode::ZPX, mem),
            ROR_ZPX => self.ror_addr(Mode::ZPX, mem),
            EOR_ZPX => self.eor(Mode::ZPX, mem),
            LSR_ZPX => self.lsr_addr(Mode::ZPX, mem),
            ROL_ZPX => self.rol_addr(Mode::ZPX, mem),
            AND_ZPX => self.and(Mode::ZPX, mem),
            ORA_ZPX => self.ora(Mode::ZPX, mem),
            ASL_ZPX => self.asl_addr(Mode::ZPX, mem),
            0xD7 => self.dcp(Mode::ZPX, mem),
            0xF7 => self.isc(Mode::ZPX, mem),
            0x17 => self.slo(Mode::ZPX, mem),
            0x37 => self.rla(Mode::ZPX, mem),
            0x57 => self.sre(Mode::ZPX, mem),
            0x77 => self.rra(Mode::ZPX, mem),

            INC_ABS => self.inc(Mode::Abs, mem),
            SBC_ABS => self.sbc(Mode::Abs, mem),
            CPX_ABS => self.cpx(Mode::Abs, mem),
            LDX_ABS => self.ldx(Mode::Abs, mem),
            DEC_ABS => self.dec(Mode::Abs, mem),
            CMP_ABS => self.cmp(Mode::Abs, mem),
            CPY_ABS => self.cpy(Mode::Abs, mem),
            LDA_ABS => self.lda(Mode::Abs, mem),
            LDY_ABS => self.ldy(Mode::Abs, mem),
            STA_ABS => self.sta(Mode::Abs, mem),
            STX_ABS => self.stx(Mode::Abs, mem),
            STY_ABS => self.sty(Mode::Abs, mem),
            ADC_ABS => self.adc(Mode::Abs, mem),
            ROR_ABS => self.ror_addr(Mode::Abs, mem),
            EOR_ABS => self.eor(Mode::Abs, mem),
            LSR_ABS => self.lsr_addr(Mode::Abs, mem),
            JMP_ABS => self.jmp(Mode::Abs, mem),
            ROL_ABS => self.rol_addr(Mode::Abs, mem),
            AND_ABS => self.and(Mode::Abs, mem),
            BIT_ABS => self.bit(Mode::Abs, mem),
            ORA_ABS => self.ora(Mode::Abs, mem),
            ASL_ABS => self.asl_addr(Mode::Abs, mem),
            0x8F => self.aax(Mode::Abs, mem),
            0xAF => self.lax(Mode::Abs, mem),
            0xCF => self.dcp(Mode::Abs, mem),
            0xEF => self.isc(Mode::Abs, mem),
            0x0F => self.slo(Mode::Abs, mem),
            0x2F => self.rla(Mode::Abs, mem),
            0x4F => self.sre(Mode::Abs, mem),
            0x6F => self.rra(Mode::Abs, mem),
            // TOP: Triple NOP
            0x0C => {
                self.address_mem(Mode::Abs, mem);
            }

            EOR_ZP => self.eor(Mode::ZP, mem),
            ROR_ZP => self.ror_addr(Mode::ZP, mem),
            LSR_ZP => self.lsr_addr(Mode::ZP, mem),
            ROL_ZP => self.rol_addr(Mode::ZP, mem),
            AND_ZP => self.and(Mode::ZP, mem),
            BIT_ZP => self.bit(Mode::ZP, mem),
            ORA_ZP => self.ora(Mode::ZP, mem),
            ASL_ZP => self.asl_addr(Mode::ZP, mem),
            0x87 => self.aax(Mode::ZP, mem),
            0xA7 => self.lax(Mode::ZP, mem),
            0xC7 => self.dcp(Mode::ZP, mem),
            0xE7 => self.isc(Mode::ZP, mem),
            0x07 => self.slo(Mode::ZP, mem),
            0x27 => self.rla(Mode::ZP, mem),
            0x47 => self.sre(Mode::ZP, mem),
            0x67 => self.rra(Mode::ZP, mem),
            INC_ZP => self.inc(Mode::ZP, mem),
            CPX_ZP => self.cpx(Mode::ZP, mem),
            LDX_ZP => self.ldx(Mode::ZP, mem),
            DEC_ZP => self.dec(Mode::ZP, mem),
            CMP_ZP => self.cmp(Mode::ZP, mem),
            CPY_ZP => self.cpy(Mode::ZP, mem),
            LDA_ZP => self.lda(Mode::ZP, mem),
            LDY_ZP => self.ldy(Mode::ZP, mem),
            STA_ZP => self.sta(Mode::ZP, mem),
            STX_ZP => self.stx(Mode::ZP, mem),
            STY_ZP => self.sty(Mode::ZP, mem),
            ADC_ZP => self.adc(Mode::ZP, mem),
            SBC_ZP => self.sbc(Mode::ZP, mem),

            LDY_IMM => self.ldy(Mode::Imm, mem),
            SBC_IMM | 0xEB => self.sbc(Mode::Imm, mem),
            0x6B => self.arr(Mode::Imm, mem),
            0xCB => self.axs(Mode::Imm, mem),
            ADC_IMM => self.adc(Mode::Imm, mem),
            0x4B => self.alr(Mode::Imm, mem),
            EOR_IMM => self.eor(Mode::Imm, mem),
            0xAB => self.atx(Mode::Imm, mem),
            AND_IMM => self.and(Mode::Imm, mem),
            ORA_IMM => self.ora(Mode::Imm, mem),
            CPX_IMM => self.cpx(Mode::Imm, mem),
            LDX_IMM => self.ldx(Mode::Imm, mem),
            0x0B | 0x2B => self.aac(Mode::Imm, mem),
            CMP_IMM => self.cmp(Mode::Imm, mem),
            CPY_IMM => self.cpy(Mode::Imm, mem),
            LDA_IMM => self.lda(Mode::Imm, mem),

            SBC_ABSX => self.sbc(Mode::AbsX, mem),
            CMP_ABSX => self.cmp(Mode::AbsX, mem),
            LDA_ABSX => self.lda(Mode::AbsX, mem),
            LDY_ABSX => self.ldy(Mode::AbsX, mem),
            ADC_ABSX => self.adc(Mode::AbsX, mem),
            EOR_ABSX => self.eor(Mode::AbsX, mem),
            AND_ABSX => self.and(Mode::AbsX, mem),
            ORA_ABSX => self.ora(Mode::AbsX, mem),
            0x9C => self.sna(Mode::AbsX, self.regs.y, self.regs.x, mem), //sya
            // The memory access here is necessary, since even though we don't use the u16 retrieved,
            // we need to emulate the absolute X retrieval in case there is a page boundary cross
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                self.address_mem(Mode::AbsX, mem);
            }

            SBC_ABSY => self.sbc(Mode::AbsY, mem),
            LDX_ABSY => self.ldx(Mode::AbsY, mem),
            CMP_ABSY => self.cmp(Mode::AbsY, mem),
            LDA_ABSY => self.lda(Mode::AbsY, mem),
            ADC_ABSY => self.adc(Mode::AbsY, mem),
            EOR_ABSY => self.eor(Mode::AbsY, mem),
            AND_ABSY => self.and(Mode::AbsY, mem),
            ORA_ABSY => self.ora(Mode::AbsY, mem),
            0xBF => self.lax(Mode::AbsY, mem),
            0x9E => self.sna(Mode::AbsY, self.regs.x, self.regs.y, mem), //sxa

            EOR_INDY => self.eor(Mode::IndY, mem),
            AND_INDY => self.and(Mode::IndY, mem),
            ORA_INDY => self.ora(Mode::IndY, mem),
            SBC_INDY => self.sbc(Mode::IndY, mem),
            CMP_INDY => self.cmp(Mode::IndY, mem),
            LDA_INDY => self.lda(Mode::IndY, mem),
            ADC_INDY => self.adc(Mode::IndY, mem),
            0xB3 => self.lax(Mode::IndY, mem),

            SBC_INDX => self.sbc(Mode::IndX, mem),
            CMP_INDX => self.cmp(Mode::IndX, mem),
            LDA_INDX => self.lda(Mode::IndX, mem),
            STA_INDX => self.sta(Mode::IndX, mem),
            ADC_INDX => self.adc(Mode::IndX, mem),
            EOR_INDX => self.eor(Mode::IndX, mem),
            AND_INDX => self.and(Mode::IndX, mem),
            ORA_INDX => self.ora(Mode::IndX, mem),
            0x83 => self.aax(Mode::IndX, mem),
            0xA3 => self.lax(Mode::IndX, mem),
            0xC3 => self.dcp(Mode::IndX, mem),
            0xE3 => self.isc(Mode::IndX, mem),
            0x03 => self.slo(Mode::IndX, mem),
            0x23 => self.rla(Mode::IndX, mem),
            0x43 => self.sre(Mode::IndX, mem),
            0x63 => self.rra(Mode::IndX, mem),

            LDX_ZPY => self.ldx(Mode::ZPY, mem),
            STX_ZPY => self.stx(Mode::ZPY, mem),
            0x97 => self.aax(Mode::ZPY, mem),
            0xB7 => self.lax(Mode::ZPY, mem),

            STA_INDY => self.sta(Mode::NoPBIndY, mem),
            0xD3 => self.dcp(Mode::NoPBIndY, mem),
            0xF3 => self.isc(Mode::NoPBIndY, mem),
            0x13 => self.slo(Mode::NoPBIndY, mem),
            0x33 => self.rla(Mode::NoPBIndY, mem),
            0x53 => self.sre(Mode::NoPBIndY, mem),
            0x73 => self.rra(Mode::NoPBIndY, mem),

            ROR_ACC => {
                mem.ld8(self.regs.pc.get_addr());
                self.ror_acc();
            }
            LSR_ACC => {
                mem.ld8(self.regs.pc.get_addr());
                self.lsr_acc();
            }
            ROL_ACC => {
                mem.ld8(self.regs.pc.get_addr());
                self.rol_acc();
            }
            ASL_ACC => {
                mem.ld8(self.regs.pc.get_addr());
                self.asl_acc();
            }
            RTS => {
                mem.ld8(self.regs.pc.get_addr());
                self.rts(mem);
            }
            RTI => {
                mem.ld8(self.regs.pc.get_addr());
                self.rti(mem);
            }
            SED => {
                mem.ld8(self.regs.pc.get_addr());
                self.regs.flags.set_dec(true);
            }
            CLC => {
                mem.ld8(self.regs.pc.get_addr());
                self.regs.flags.set_carry(false);
            }
            SEC => {
                mem.ld8(self.regs.pc.get_addr());
                self.regs.flags.set_carry(true);
            }
            CLI => {
                mem.ld8(self.regs.pc.get_addr());
                self.regs.flags.set_itr(false);
            }
            SEI => {
                mem.ld8(self.regs.pc.get_addr());
                self.regs.flags.set_itr(true);
            }
            CLV => {
                mem.ld8(self.regs.pc.get_addr());
                self.regs.flags.set_overflow(false);
            }
            CLD => {
                mem.ld8(self.regs.pc.get_addr());
                self.regs.flags.set_dec(false);
            }
            NOP | 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => {
                mem.ld8(self.regs.pc.get_addr());
            }
            // DOP: Double NOP
            0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 | 0x82 | 0x89 | 0xC2 | 0xD4 | 0xE2
            | 0xF4 | 0x04 => {
                self.ld8_pc_up(mem);
            }
            BRK => self.brk(mem),
            TAX => {
                mem.ld8(self.regs.pc.get_addr());
                self.tax();
            }
            TXA => {
                mem.ld8(self.regs.pc.get_addr());
                self.txa();
            }
            TAY => {
                mem.ld8(self.regs.pc.get_addr());
                self.tay();
            }
            TYA => {
                mem.ld8(self.regs.pc.get_addr());
                self.tya();
            }
            DEX => {
                mem.ld8(self.regs.pc.get_addr());
                self.dex();
            }
            INX => {
                mem.ld8(self.regs.pc.get_addr());
                self.inx();
            }
            DEY => {
                mem.ld8(self.regs.pc.get_addr());
                self.dey();
            }
            INY => {
                mem.ld8(self.regs.pc.get_addr());
                self.iny();
            }
            TSX => {
                mem.ld8(self.regs.pc.get_addr());
                self.tsx();
            }
            TXS => {
                mem.ld8(self.regs.pc.get_addr());
                self.regs.sp = self.regs.x;
            }
            PHA => {
                mem.ld8(self.regs.pc.get_addr());
                self.push(self.regs.acc, mem);
            }
            PLA => {
                mem.ld8(self.regs.pc.get_addr());
                self.pla(mem);
            }
            PHP => {
                mem.ld8(self.regs.pc.get_addr());
                self.push(self.regs.flags.as_byte() | 0b10000, mem);
            }
            PLP => {
                mem.ld8(self.regs.pc.get_addr());
                self.pull_status(mem);
            }
            BVS => self.generic_branch(self.regs.flags.overflow(), mem),
            BVC => self.generic_branch(!self.regs.flags.overflow(), mem),
            BMI => self.generic_branch(self.regs.flags.neg(), mem),
            BPL => self.generic_branch(!self.regs.flags.neg(), mem),
            BNE => self.generic_branch(!self.regs.flags.zero(), mem),
            BEQ => self.generic_branch(self.regs.flags.zero(), mem),
            BCS => self.generic_branch(self.regs.flags.carry(), mem),
            BCC => self.generic_branch(!self.regs.flags.carry(), mem),
            JSR => self.jsr(mem),
            JMP_IND => self.jmp(Mode::JmpIndir, mem),
            _ => panic!("Unsupported op {:X} {:?}", op, self.regs),
        }
    }
}
