use cpu_const::*;
use log::Level;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use Memory;

#[derive(Serialize, Deserialize, Clone)]
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
}

#[derive(Serialize, Deserialize, Clone)]
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

pub struct Cpu<M: Memory> {
    pub regs: Registers,
    pub cycle_count: u16,
    pub mmu: M,
    cc: usize,
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

impl <M: Memory> Cpu<M> {
    pub fn new(mmu: M) -> Cpu<M> {
        let mut cpu = Cpu {
            cycle_count: 0,
            cc: 0,
            regs: Registers {
                acc: 0,
                x: 0,
                y: 0,
                pc: ProgramCounter::new(0),
                sp: 0xFD,
                flags: Flags(0b00100100),
            },
            mmu: mmu,
        };
        cpu.regs.pc.set_addr(cpu.mmu.ld16(RESET_VEC, 0));
        cpu
    }

    pub fn reset(&mut self) {
        self.cycle_count = 0;
        self.cc = 0;
        let addr = self.mmu.ld16(RESET_VEC, self.cc);
        self.regs.reset(addr);
    }

    fn check_pb(&mut self, base: u16, base_offset: u16) {
        if (base & 0xFF00) != (base_offset & 0xFF00) {
            self.incr_cc();
        }
    }

    fn incr_cc(&mut self) {
        self.cycle_count += 1;
    }

    fn address_mem(&mut self, mode: Mode) -> u16 {
        match mode {
            Mode::Imm => {
                let tmp = self.regs.pc.get_addr();
                self.regs.pc.add_unsigned(1);
                tmp
            }
            Mode::ZP => self.ld8_pc_up() as u16,
            Mode::ZPX => {
                let tmp = self.ld8_pc_up();
                tmp.wrapping_add(self.regs.x) as u16
            }
            Mode::ZPY => {
                let tmp = self.ld8_pc_up();
                tmp.wrapping_add(self.regs.y) as u16
            }
            Mode::Abs => self.ld16_pc_up(),
            Mode::AbsX => {
                let base = self.ld16_pc_up();
                let tmp = base + self.regs.x as u16;
                self.check_pb(base, tmp);
                tmp
            }
            Mode::AbsY => {
                let base = self.ld16_pc_up();
                let tmp = base.wrapping_add(self.regs.y as u16);
                self.check_pb(base, tmp);
                tmp
            }
            Mode::NoPBAbsX => {
                let base = self.ld16_pc_up();
                let tmp = base + self.regs.x as u16;
                tmp
            }
            Mode::NoPBAbsY => {
                let base = self.ld16_pc_up();
                let tmp = base.wrapping_add(self.regs.y as u16);
                tmp
            }
            Mode::JmpIndir => {
                let tmp = self.ld16_pc_up();
                let low = self.mmu.ld8(tmp, self.cc);
                let high: u8 = if tmp & 0xFF == 0xFF {
                    self.mmu.ld8(tmp - 0xFF, self.cc)
                } else {
                    self.mmu.ld8(tmp + 1, self.cc)
                };
                (high as u16) << 8 | (low as u16)
            }
            Mode::IndX => {
                let tmp = self.ld8_pc_up();
                let base_address = tmp.wrapping_add(self.regs.x) as u16;
                if base_address == 0xFF {
                    (self.mmu.ld8(0, self.cc) as u16) << 8
                        | (self.mmu.ld8(base_address, self.cc) as u16)
                } else {
                    self.mmu.ld16(base_address, self.cc)
                }
            }
            Mode::IndY => {
                let base = self.ld8_pc_up();
                let tmp = if base == 0xFF {
                    (self.mmu.ld8(0, self.cc) as u16) << 8
                        | (self.mmu.ld8(0xFF, self.cc) as u16)
                } else {
                    self.mmu.ld16(base as u16, self.cc)
                };
                let addr = tmp.wrapping_add(self.regs.y as u16);
                self.check_pb(tmp, addr);
                addr
            }
            Mode::NoPBIndY => {
                let base = self.ld8_pc_up();
                let tmp = if base == 0xFF {
                    (self.mmu.ld8(0, self.cc) as u16) << 8
                        | (self.mmu.ld8(0xFF, self.cc) as u16)
                } else {
                    self.mmu.ld16(base as u16, self.cc)
                };
                let addr = tmp.wrapping_add(self.regs.y as u16);
                addr
            }
        }
    }

    pub fn proc_nmi(&mut self) {
        let flags = self.regs.flags;
        self.push_pc();
        self.push(flags.as_byte());
        self.regs.pc.set_addr(self.mmu.ld16(NMI_VEC, self.cc));
    }

    fn read_op(&mut self, mode: Mode) -> u8 {
        let addr = self.address_mem(mode);
        self.mmu.ld8(addr, self.cc)
    }

    fn write_dma(&mut self, high_nyb: u8) {
        self.cycle_count += 513 + (self.cycle_count % 2);
        let page_num = (high_nyb as u16) << 8;
        for address in page_num..=page_num + 0xFF {
            let tmp = self.mmu.ld8(address, self.cc);
            self.mmu.store(OAM_DATA, tmp, self.cc);
        }
    }

    fn store(&mut self, addr: u16, val: u8) {
        if addr == DMA_ADDR {
            self.write_dma(val);
        } else {
            self.mmu.store(addr, val, self.cc);
        }
    }

    fn and(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let tmp = self.regs.acc & val;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn ora(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let tmp = self.regs.acc | val;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn eor(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let tmp = self.regs.acc ^ val;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn adc_val(&mut self, val: u8) {
        let acc = self.regs.acc;
        let tmp = acc as u16 + val as u16 + self.regs.flags.carry() as u16;
        self.regs.flags.set_carry(tmp > 0xFF);
        self.regs.flags.set_overflow(
            ((acc as u16 ^ tmp) & (val as u16 ^ tmp) & 0x80) != 0,
        );
        let tmp = tmp as u8;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn adc(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        self.adc_val(val);
    }

    fn sbc(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        self.adc_val(val ^ 0xFF);
    }

    fn lda(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        self.regs.acc = val;
        self.set_zero_neg(val);
    }

    fn ldx(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        self.regs.x = val;
        self.set_zero_neg(val);
    }

    fn ldy(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        self.regs.y = val;
        self.set_zero_neg(val);
    }

    fn ror_acc(&mut self) {
        let (tmp, n_flag) =
            Cpu::<M>::get_ror(self.regs.flags.carry(), self.regs.acc);
        self.regs.flags.set_carry(n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn ror_addr(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let (tmp, n_flag) =
            Cpu::<M>::get_ror(self.regs.flags.carry(), self.mmu.ld8(addr, self.cc));
        self.regs.flags.set_carry(n_flag);
        self.set_zero_neg(tmp);
        self.store(addr, tmp);
    }

    fn get_ror(carry_flag: bool, val: u8) -> (u8, bool) {
        ((val >> 1) | ((carry_flag as u8) << 7), (val & 0b01) != 0)
    }

    fn rol_acc(&mut self) {
        let (tmp, n_flag) =
            Cpu::<M>::get_rol(self.regs.flags.carry(), self.regs.acc);
        self.regs.flags.set_carry(n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn rol_addr(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let (tmp, n_flag) =
            Cpu::<M>::get_rol(self.regs.flags.carry(), self.mmu.ld8(addr, self.cc));
        self.regs.flags.set_carry(n_flag);
        self.set_zero_neg(tmp);
        self.store(addr, tmp);
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

    fn asl_addr(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val = self.mmu.ld8(addr, self.cc);
        self.regs.flags.set_carry((val >> 7) != 0);
        let tmp = val << 1;
        self.set_zero_neg(tmp);
        self.store(addr, tmp);
    }

    fn lsr_acc(&mut self) {
        let acc = self.regs.acc;
        self.regs.flags.set_carry((acc & 0b01) != 0);
        let tmp = acc >> 1;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn lsr_addr(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val = self.mmu.ld8(addr, self.cc);
        self.regs.flags.set_carry((val & 0b01) != 0);
        let tmp = val >> 1;
        self.set_zero_neg(tmp);
        self.store(addr, tmp);
    }

    fn cpx(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let tmp = self.regs.x as i16 - val as i16;
        self.regs.flags.set_carry(tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn cpy(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let tmp = self.regs.y as i16 - val as i16;
        self.regs.flags.set_carry(tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn cmp(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let tmp = self.regs.acc as i16 - val as i16;
        self.regs.flags.set_carry(tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn generic_branch(&mut self, flag: bool) {
        let val = self.ld8_pc_up() as i8;
        if flag {
            let addr = self.regs.pc.get_addr();
            self.regs.pc.add_signed(val);
            self.incr_cc();
            let new_addr = self.regs.pc.get_addr();
            self.check_pb(addr, new_addr)
        }
    }

    fn bit(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let acc = self.regs.acc;
        self.regs.flags.set_zero((val & acc) == 0);
        self.regs.flags.set_overflow((val & 0x40) != 0);
        self.regs.flags.set_neg((val & 0x80) != 0);
    }

    fn dec(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val: u8 = self.mmu.ld8(addr, self.cc).wrapping_sub(1);
        self.set_zero_neg(val);
        self.store(addr, val);
    }

    fn inc(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val: u8 = self.mmu.ld8(addr, self.cc).wrapping_add(1);
        self.set_zero_neg(val);
        self.store(addr, val);
    }

    fn sta(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let tmp = self.regs.acc;
        self.store(addr, tmp);
    }

    fn stx(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let tmp = self.regs.x;
        self.store(addr, tmp);
    }

    fn sty(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let tmp = self.regs.y;
        self.store(addr, tmp);
    }

    fn jmp(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        self.regs.pc.set_addr(addr);
    }

    fn aac(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let acc = self.regs.acc;
        self.regs.acc = acc & val;
        self.set_zero_neg(self.regs.acc);
        self.regs.flags.set_carry(self.regs.flags.neg());
    }

    fn aax(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let tmp = self.regs.acc & self.regs.x;
        self.store(addr, tmp);
    }

    fn arr(&mut self, mode: Mode) {
        self.and(mode);
        let (acc, _) = Cpu::<M>::get_ror(self.regs.flags.carry(), self.regs.acc);
        self.regs.acc = acc;
        let b5 = ((acc >> 5) & 1) == 1;
        let b6 = ((acc >> 6) & 1) == 1;
        self.regs.flags.set_carry(b6);
        self.regs.flags.set_overflow(b5 ^ b6);
        self.set_zero_neg(self.regs.acc);
    }

    fn alr(&mut self, mode: Mode) {
        self.and(mode);
        self.lsr_acc();
    }

    fn lax(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        self.regs.acc = val;
        self.regs.x = val;
        self.set_zero_neg(val);
    }

    fn axs(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let tmp = self.regs.x & self.regs.acc;
        let (tmp, carry) = tmp.overflowing_sub(val);
        // No idea why this is !carry
        self.regs.flags.set_carry(!carry);
        self.regs.x = tmp;
        self.set_zero_neg(tmp);
    }

    //TODO this is dec followed by cmp, refactor this to use those functions
    fn dcp(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val: u8 = self.mmu.ld8(addr, self.cc).wrapping_sub(1);
        self.set_zero_neg(val);
        self.store(addr, val);
        let tmp = self.regs.acc as i16 - val as i16;
        self.regs.flags.set_carry(tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    //TODO This one can also probably be refactored
    fn isc(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val: u8 = self.mmu.ld8(addr, self.cc).wrapping_add(1);
        self.set_zero_neg(val);
        self.store(addr, val);
        self.adc_val(val ^ 0xFF);
    }

    //TODO same as this one
    fn slo(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val = self.mmu.ld8(addr, self.cc);
        self.regs.flags.set_carry((val >> 7) != 0);
        let tmp = val << 1;
        self.store(addr, tmp);

        let tmp = self.regs.acc | tmp;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn rla(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let (tmp, n_flag) =
            Cpu::<M>::get_rol(self.regs.flags.carry(), self.mmu.ld8(addr, self.cc));
        self.regs.flags.set_carry(n_flag);
        self.store(addr, tmp);

        let tmp = self.regs.acc & tmp;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn sre(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val = self.mmu.ld8(addr, self.cc);
        self.regs.flags.set_carry((val & 0b01) != 0);
        let tmp = val >> 1;
        self.store(addr, tmp);

        let tmp = self.regs.acc ^ tmp;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn rra(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let (tmp, n_flag) =
            Cpu::<M>::get_ror(self.regs.flags.carry(), self.mmu.ld8(addr, self.cc));
        self.regs.flags.set_carry(n_flag);
        self.set_zero_neg(tmp);
        self.store(addr, tmp);
        self.adc_val(tmp);
    }

    fn tax(&mut self) {
        let acc = self.regs.acc;
        self.regs.x = acc;
        self.set_zero_neg(acc);
    }

    fn atx(&mut self, mode: Mode) {
        self.lda(mode);
        self.tax();
    }

    fn sna(&mut self, mode: Mode, and_reg: u8, addr_mode_reg: u8) {
        let mut addr = self.address_mem(mode);
        if (addr - addr_mode_reg as u16) & 0xFF00 != addr & 0xFF00 {
            addr &= (and_reg as u16) << 8;
        }
        self.store(addr, and_reg & (((addr >> 8) as u8).wrapping_add(1)));
    }

    fn brk(&mut self) {
        self.regs.pc.add_signed(1);
        self.push_pc();
        self.push(self.regs.flags.as_byte() | 0b10000);
        self.regs.flags.set_itr(true);
        self.regs.pc.set_addr(self.mmu.ld16(IRQ_VEC, self.cc));
    }

    fn rts(&mut self) {
        self.pull_pc();
        self.regs.pc.add_unsigned(1);
    }

    fn rti(&mut self) {
        self.pull_status();
        self.pull_pc();
    }

    fn jsr(&mut self) {
        let addr = self.address_mem(Mode::Abs);
        self.regs.pc.add_signed(-1);
        self.push_pc();
        self.regs.pc.set_addr(addr);
    }

    fn pla(&mut self) {
        let acc = self.pop();
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

    fn push(&mut self, val: u8) {
        let addr = self.regs.sp as u16 | 0x100;
        self.store(addr, val);
        self.regs.sp -= 1;
    }

    fn pop(&mut self) -> u8 {
        self.regs.sp += 1;
        self.mmu.ld8(self.regs.sp as u16 | 0x100, self.cc)
    }

    fn pull_pc(&mut self) {
        let low = self.pop();
        let high = self.pop();
        self.regs.pc.set_addr(((high as u16) << 8) | low as u16);
    }

    fn pull_status(&mut self) {
        let tmp = self.pop();
        self.regs.flags.set_byte(tmp);
        self.regs.flags.set_unused(true);
        self.regs.flags.set_brk(false);
    }

    fn push_pc(&mut self) {
        let high = self.regs.pc.get_addr() >> 8;
        let low = self.regs.pc.get_addr();
        self.push(high as u8);
        self.push(low as u8);
    }

    fn set_zero_neg(&mut self, val: u8) {
        self.regs.flags.set_neg(val >> 7 == 1);
        self.regs.flags.set_zero(val == 0);
    }

    pub fn step(&mut self) -> u16 {
        let byte = self.ld8_pc_up();
        self.cycle_count += CYCLES[byte as usize] as u16;
        self.execute_op(byte);
        let tmp = self.cycle_count;
        if log_enabled!(Level::Debug) {
            debug!("INST: {:X} {:?} CYC:{}", byte, self.regs.clone(), self.cc);
        }
        self.cc += tmp as usize;
        self.cycle_count = 0;
        tmp
    }

    fn ld8_pc_up(&mut self) -> u8 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add_unsigned(1);
        self.mmu.ld8(ram_ptr, self.cc)
    }

    fn ld16_pc_up(&mut self) -> u16 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add_unsigned(2);
        self.mmu.ld16(ram_ptr, self.cc)
    }

    pub fn execute_op(&mut self, op: u8) {
        match op {
            INC_ABSX => self.inc(Mode::NoPBAbsX),
            INC_ZPX => self.inc(Mode::ZPX),
            INC_ABS => self.inc(Mode::Abs),
            INC_ZP => self.inc(Mode::ZP),
            SBC_IMM | 0xEB => self.sbc(Mode::Imm),
            SBC_ABSX => self.sbc(Mode::AbsX),
            SBC_ABSY => self.sbc(Mode::AbsY),
            SBC_ABS => self.sbc(Mode::Abs),
            SBC_INDY => self.sbc(Mode::IndY),
            SBC_INDX => self.sbc(Mode::IndX),
            SBC_ZPX => self.sbc(Mode::ZPX),
            SBC_ZP => self.sbc(Mode::ZP),
            CPX_IMM => self.cpx(Mode::Imm),
            CPX_ABS => self.cpx(Mode::Abs),
            CPX_ZP => self.cpx(Mode::ZP),
            LDX_IMM => self.ldx(Mode::Imm),
            LDX_ZPY => self.ldx(Mode::ZPY),
            LDX_ABS => self.ldx(Mode::Abs),
            LDX_ABSY => self.ldx(Mode::AbsY),
            LDX_ZP => self.ldx(Mode::ZP),
            DEC_ZPX => self.dec(Mode::ZPX),
            DEC_ABS => self.dec(Mode::Abs),
            DEC_ABSX => self.dec(Mode::NoPBAbsX),
            DEC_ZP => self.dec(Mode::ZP),
            CMP_IMM => self.cmp(Mode::Imm),
            CMP_ABSX => self.cmp(Mode::AbsX),
            CMP_ABSY => self.cmp(Mode::AbsY),
            CMP_ZPX => self.cmp(Mode::ZPX),
            CMP_INDY => self.cmp(Mode::IndY),
            CMP_ABS => self.cmp(Mode::Abs),
            CMP_ZP => self.cmp(Mode::ZP),
            CMP_INDX => self.cmp(Mode::IndX),
            CPY_IMM => self.cpy(Mode::Imm),
            CPY_ZP => self.cpy(Mode::ZP),
            CPY_ABS => self.cpy(Mode::Abs),
            LDA_IMM => self.lda(Mode::Imm),
            LDA_ABSX => self.lda(Mode::AbsX),
            LDA_ABSY => self.lda(Mode::AbsY),
            LDA_ZPX => self.lda(Mode::ZPX),
            LDA_INDY => self.lda(Mode::IndY),
            LDA_ABS => self.lda(Mode::Abs),
            LDA_ZP => self.lda(Mode::ZP),
            LDA_INDX => self.lda(Mode::IndX),
            LDY_IMM => self.ldy(Mode::Imm),
            LDY_ZPX => self.ldy(Mode::ZPX),
            LDY_ABS => self.ldy(Mode::Abs),
            LDY_ABSX => self.ldy(Mode::AbsX),
            LDY_ZP => self.ldy(Mode::ZP),
            STA_ABSX => self.sta(Mode::NoPBAbsX),
            STA_ABSY => self.sta(Mode::NoPBAbsY),
            STA_ZPX => self.sta(Mode::ZPX),
            STA_INDY => self.sta(Mode::NoPBIndY),
            STA_ABS => self.sta(Mode::Abs),
            STA_ZP => self.sta(Mode::ZP),
            STA_INDX => self.sta(Mode::IndX),
            STX_ABS => self.stx(Mode::Abs),
            STX_ZP => self.stx(Mode::ZP),
            STX_ZPY => self.stx(Mode::ZPY),
            STY_ABS => self.sty(Mode::Abs),
            STY_ZP => self.sty(Mode::ZP),
            STY_ZPX => self.sty(Mode::ZPX),
            ADC_IMM => self.adc(Mode::Imm),
            ADC_ABSX => self.adc(Mode::AbsX),
            ADC_ABSY => self.adc(Mode::AbsY),
            ADC_ZPX => self.adc(Mode::ZPX),
            ADC_INDY => self.adc(Mode::IndY),
            ADC_ABS => self.adc(Mode::Abs),
            ADC_ZP => self.adc(Mode::ZP),
            ADC_INDX => self.adc(Mode::IndX),
            ROR_ABSX => self.ror_addr(Mode::NoPBAbsX),
            ROR_ZPX => self.ror_addr(Mode::ZPX),
            ROR_ZP => self.ror_addr(Mode::ZP),
            ROR_ABS => self.ror_addr(Mode::Abs),
            ROR_ACC => self.ror_acc(),
            EOR_IMM => self.eor(Mode::Imm),
            EOR_ABSX => self.eor(Mode::AbsX),
            EOR_ABSY => self.eor(Mode::AbsY),
            EOR_ZPX => self.eor(Mode::ZPX),
            EOR_INDY => self.eor(Mode::IndY),
            EOR_ABS => self.eor(Mode::Abs),
            EOR_ZP => self.eor(Mode::ZP),
            EOR_INDX => self.eor(Mode::IndX),
            LSR_ABSX => self.lsr_addr(Mode::NoPBAbsX),
            LSR_ZPX => self.lsr_addr(Mode::ZPX),
            LSR_ABS => self.lsr_addr(Mode::Abs),
            LSR_ZP => self.lsr_addr(Mode::ZP),
            LSR_ACC => self.lsr_acc(),
            JMP_ABS => self.jmp(Mode::Abs),
            ROL_ABS => self.rol_addr(Mode::Abs),
            ROL_ABSX => self.rol_addr(Mode::NoPBAbsX),
            ROL_ZPX => self.rol_addr(Mode::ZPX),
            ROL_ZP => self.rol_addr(Mode::ZP),
            ROL_ACC => self.rol_acc(),
            AND_IMM => self.and(Mode::Imm),
            AND_ZP => self.and(Mode::ZP),
            AND_ABSX => self.and(Mode::AbsX),
            AND_ABSY => self.and(Mode::AbsY),
            AND_INDY => self.and(Mode::IndY),
            AND_ABS => self.and(Mode::Abs),
            AND_INDX => self.and(Mode::IndX),
            AND_ZPX => self.and(Mode::ZPX),
            BIT_ABS => self.bit(Mode::Abs),
            BIT_ZP => self.bit(Mode::ZP),
            ORA_IMM => self.ora(Mode::Imm),
            ORA_ABSX => self.ora(Mode::AbsX),
            ORA_ABSY => self.ora(Mode::AbsY),
            ORA_ZPX => self.ora(Mode::ZPX),
            ORA_INDY => self.ora(Mode::IndY),
            ORA_ABS => self.ora(Mode::Abs),
            ORA_ZP => self.ora(Mode::ZP),
            ORA_INDX => self.ora(Mode::IndX),
            ASL_ABSX => self.asl_addr(Mode::NoPBAbsX),
            ASL_ABS => self.asl_addr(Mode::Abs),
            ASL_ZP => self.asl_addr(Mode::ZP),
            ASL_ZPX => self.asl_addr(Mode::ZPX),
            ASL_ACC => self.asl_acc(),
            0x0B | 0x2B => self.aac(Mode::Imm),
            0x87 => self.aax(Mode::ZP),
            0x97 => self.aax(Mode::ZPY),
            0x83 => self.aax(Mode::IndX),
            0x8F => self.aax(Mode::Abs),
            0x6B => self.arr(Mode::Imm),
            0xA7 => self.lax(Mode::ZP),
            0xB7 => self.lax(Mode::ZPY),
            0xAF => self.lax(Mode::Abs),
            0xBF => self.lax(Mode::AbsY),
            0xA3 => self.lax(Mode::IndX),
            0xB3 => self.lax(Mode::IndY),
            0xCB => self.axs(Mode::Imm),
            0xC7 => self.dcp(Mode::ZP),
            0xD7 => self.dcp(Mode::ZPX),
            0xCF => self.dcp(Mode::Abs),
            0xDF => self.dcp(Mode::NoPBAbsX),
            0xDB => self.dcp(Mode::NoPBAbsY),
            0xC3 => self.dcp(Mode::IndX),
            0xD3 => self.dcp(Mode::NoPBIndY),
            0xE7 => self.isc(Mode::ZP),
            0xF7 => self.isc(Mode::ZPX),
            0xEF => self.isc(Mode::Abs),
            0xFF => self.isc(Mode::NoPBAbsX),
            0xFB => self.isc(Mode::NoPBAbsY),
            0xE3 => self.isc(Mode::IndX),
            0xF3 => self.isc(Mode::NoPBIndY),
            0x07 => self.slo(Mode::ZP),
            0x17 => self.slo(Mode::ZPX),
            0x0F => self.slo(Mode::Abs),
            0x1F => self.slo(Mode::NoPBAbsX),
            0x1B => self.slo(Mode::NoPBAbsY),
            0x03 => self.slo(Mode::IndX),
            0x13 => self.slo(Mode::NoPBIndY),
            0x27 => self.rla(Mode::ZP),
            0x37 => self.rla(Mode::ZPX),
            0x2F => self.rla(Mode::Abs),
            0x3F => self.rla(Mode::NoPBAbsX),
            0x3B => self.rla(Mode::NoPBAbsY),
            0x23 => self.rla(Mode::IndX),
            0x33 => self.rla(Mode::NoPBIndY),
            0x47 => self.sre(Mode::ZP),
            0x57 => self.sre(Mode::ZPX),
            0x4F => self.sre(Mode::Abs),
            0x5F => self.sre(Mode::NoPBAbsX),
            0x5B => self.sre(Mode::NoPBAbsY),
            0x53 => self.sre(Mode::NoPBIndY),
            0x43 => self.sre(Mode::IndX),
            0x67 => self.rra(Mode::ZP),
            0x77 => self.rra(Mode::ZPX),
            0x6F => self.rra(Mode::Abs),
            0x7F => self.rra(Mode::NoPBAbsX),
            0x7B => self.rra(Mode::NoPBAbsY),
            0x63 => self.rra(Mode::IndX),
            0x73 => self.rra(Mode::NoPBIndY),
            0x4B => self.alr(Mode::Imm),
            0xAB => self.atx(Mode::Imm),
            0x9C => self.sna(Mode::AbsX, self.regs.y, self.regs.x), //sya
            0x9E => self.sna(Mode::AbsY, self.regs.x, self.regs.y), //sxa
            RTS => self.rts(),
            RTI => self.rti(),
            SED => self.regs.flags.set_dec(true),
            CLC => self.regs.flags.set_carry(false),
            SEC => self.regs.flags.set_carry(true),
            CLI => self.regs.flags.set_itr(false),
            SEI => self.regs.flags.set_itr(true),
            CLV => self.regs.flags.set_overflow(false),
            CLD => self.regs.flags.set_dec(false),
            NOP | 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => (),
            // DOP: Double NOP
            0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 | 0x82 | 0x89
            | 0xC2 | 0xD4 | 0xE2 | 0xF4 | 0x04 => {
                self.regs.pc.add_signed(1);
            }
            // TOP: Triple NOP
            0x0C => {
                self.address_mem(Mode::Abs);
            }
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                self.address_mem(Mode::AbsX);
            }
            BRK => self.brk(),
            TAX => self.tax(),
            TXA => self.txa(),
            TAY => self.tay(),
            TYA => self.tya(),
            DEX => self.dex(),
            INX => self.inx(),
            DEY => self.dey(),
            INY => self.iny(),
            TSX => self.tsx(),
            TXS => self.regs.sp = self.regs.x,
            PHA => self.push(self.regs.acc),
            PLA => self.pla(),
            PHP => self.push(self.regs.flags.as_byte() | 0b10000),
            PLP => self.pull_status(),
            BVS => self.generic_branch(self.regs.flags.overflow()),
            BVC => self.generic_branch(!self.regs.flags.overflow()),
            BMI => self.generic_branch(self.regs.flags.neg()),
            BPL => self.generic_branch(!self.regs.flags.neg()),
            BNE => self.generic_branch(!self.regs.flags.zero()),
            BEQ => self.generic_branch(self.regs.flags.zero()),
            BCS => self.generic_branch(self.regs.flags.carry()),
            BCC => self.generic_branch(!self.regs.flags.carry()),
            JSR => self.jsr(),
            JMP_IND => self.jmp(Mode::JmpIndir),
            _ => panic!("Unsupported op {:X} {:?}", op, self.regs),
        }
    }
}
