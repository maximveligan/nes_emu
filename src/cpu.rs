use cpu_const::*;
use std::fmt;
use mmu::Mmu;
use mapper::Mapper;

#[derive(Clone)]
pub struct Registers {
    pub acc: u8,
    pub x: u8,
    pub y: u8,
    pub pc: ProgramCounter,
    pub sp: u8,
    pub flags: u8,
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} A:{:02X} X:{:02X} Y:{:02X} Flags:{:02X} SP:{:02X}",
            self.pc, self.acc, self.x, self.y, self.flags, self.sp
        )
    }
}

#[derive(Clone)]
pub struct ProgramCounter(u16);

impl ProgramCounter {
    pub fn new(val: u16) -> ProgramCounter {
        ProgramCounter { 0: val }
    }

    fn add_unsigned(&mut self, offset: u16) {
        self.0 += offset;
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

pub struct Cpu {
    pub regs: Registers,
    pub cycle_count: u16,
    pub mmu: Mmu,
}

#[derive(Debug)]
pub enum Op {
    Store(Store),
    Math(Math),
    Bit(Bit),
    Branch(Branch),
    Jump(Jump),
    Reg(Reg),
    Sys(Sys),
}

#[derive(Debug)]
pub enum Store {
    LDA,
    LDX,
    LDY,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    PHA,
    PHP,
    PLA,
    PLP,
}

#[derive(Debug)]
pub enum Math {
    ADC,
    DEC,
    DEX,
    DEY,
    INC,
    INX,
    INY,
    SBC,
}

#[derive(Debug)]
pub enum Bit {
    AND,
    ASL,
    BIT,
    EOR,
    LSR,
    ORA,
    ROL,
    ROR,
}

#[derive(Debug)]
pub enum Branch {
    BCC,
    BCS,
    BEQ,
    BMI,
    BNE,
    BPL,
    BVC,
    BVS,
}

#[derive(Debug)]
pub enum Jump {
    JMP,
    JSR,
    RTI,
    RTS,
}

#[derive(Debug)]
pub enum Reg {
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    SEC,
    SED,
    SEI,
}

#[derive(Debug)]
pub enum Sys {
    BRK,
    NOP,
}

pub enum AddrMode {
    Imm(u8),
    Impl,
    Accum,
    ZP(u8),
    ZPX(u8),
    ZPY(u8),
    Abs(u16),
    AbsX(u16),
    AbsY(u16),
    JmpIndir(u16),
    IndX(u8),
    IndY(u8),
    Rel(i8),
}

impl fmt::Debug for AddrMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddrMode::Imm(c) => write!(f, "Imm {:X}", c),
            AddrMode::Impl => write!(f, "Impl"),
            AddrMode::Accum => write!(f, "Accum"),
            AddrMode::ZP(c) => write!(f, "ZP {:X}", c),
            AddrMode::ZPX(c) => write!(f, "ZPX {:X}", c),
            AddrMode::ZPY(c) => write!(f, "ZPY {:X}", c),
            AddrMode::Abs(c) => write!(f, "Abs {:X}", c),
            AddrMode::AbsX(c) => write!(f, "AbsX {:X}", c),
            AddrMode::AbsY(c) => write!(f, "AbsY {:X}", c),
            AddrMode::JmpIndir(c) => write!(f, "JmpIndir {:X}", c),
            AddrMode::IndX(c) => write!(f, "IndX {:X}", c),
            AddrMode::IndY(c) => write!(f, "IndY {:X}", c),
            AddrMode::Rel(c) => write!(f, "Rel {:X}", c),
        }
    }
}

enum AddrDT {
    Addr(u16),
    Const(u8),
    Signed(i8),
}

impl fmt::Debug for AddrDT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddrDT::Addr(c) => write!(f, "${:X}", c),
            AddrDT::Const(c) => write!(f, "#{:X}", c),
            AddrDT::Signed(i) => write!(f, "Signed {:X}", *i as u8),
        }
    }
}

impl AddrMode {
    fn address_mem(&self, cpu: &mut Cpu) -> Option<(AddrDT, bool)> {
        match *self {
            AddrMode::Imm(v) => Some((AddrDT::Const(v), false)),
            AddrMode::Impl => None,
            AddrMode::Accum => None,
            AddrMode::ZP(v) => Some((AddrDT::Addr(v as u16), false)),
            AddrMode::ZPX(v) => {
                Some((AddrDT::Addr(v.wrapping_add(cpu.regs.x) as u16), false))
            }
            AddrMode::ZPY(v) => {
                Some((AddrDT::Addr(v.wrapping_add(cpu.regs.y) as u16), false))
            }
            AddrMode::Abs(v) => Some((AddrDT::Addr(v), false)),
            AddrMode::AbsX(v) => Some((
                AddrDT::Addr(v + (cpu.regs.x as u16)),
                check_pb(v, v + cpu.regs.x as u16),
            )),
            AddrMode::AbsY(v) => {
                let tmp = v.wrapping_add(cpu.regs.y as u16);
                Some((AddrDT::Addr(tmp), check_pb(v, tmp)))
            }
            AddrMode::JmpIndir(v) => {
                let low = cpu.mmu.ld8(v);
                let high: u8 = if v & 0xFF == 0xFF {
                    cpu.mmu.ld8(v - 0xFF)
                } else {
                    cpu.mmu.ld8(v + 1)
                };
                Some((AddrDT::Addr((high as u16) << 8 | (low as u16)), false))
            }
            AddrMode::IndX(v) => {
                let base_address = v.wrapping_add(cpu.regs.x) as u16;
                let tmp = if base_address == 0xFF {
                    (cpu.mmu.ld8(0) as u16) << 8
                        | (cpu.mmu.ld8(base_address) as u16)
                } else {
                    cpu.mmu.ld16(base_address)
                };

                //println!("Load @ {:X} = {:X} = {:X}", base_address, tmp,
                // cpu.mem.ld8(tmp)); println!("Index Indir X
                // {:X}", tmp);
                Some((AddrDT::Addr(tmp), false))
            }

            AddrMode::IndY(v) => {
                let tmp = if v == 0xFF {
                    (cpu.mmu.ld8(0) as u16) << 8 | (cpu.mmu.ld8(0xFF) as u16)
                } else {
                    cpu.mmu.ld16(v as u16)
                };

                let addr = tmp.wrapping_add(cpu.regs.y as u16);
                //println!("Indir Index Y {}", addr);
                Some((AddrDT::Addr(addr), check_pb(tmp, addr)))
            }
            AddrMode::Rel(v) => Some((AddrDT::Signed(v as i8), false)),
        }
    }
}

fn check_pb(base: u16, base_offset: u16) -> bool {
    (base & 0xFF00) != (base_offset & 0xFF00)
}

impl Cpu {
    pub fn new(mmu: Mmu) -> Cpu {
        let mut cpu = Cpu {
            cycle_count: 0,
            regs: Registers {
                acc: 0,
                x: 0,
                y: 0,
                pc: ProgramCounter::new(0),
                sp: 0xFD,
                flags: 0b00100100,
            },
            mmu: mmu,
        };
        cpu.regs.pc.set_addr(cpu.mmu.ld16(RESET_VEC));
        cpu
    }

    fn incr_cc(&mut self) {
        self.cycle_count += 1;
    }

    pub fn proc_nmi(&mut self) {
        //println!("NMI");
        let flags = self.regs.flags;
        self.push_pc();
        self.push(flags);
        self.regs.pc.set_addr(self.mmu.ld16(NMI_VEC));
    }

    fn write_dma(&mut self, high_nyb: u8) {
        // TODO: NES adds 1 cycle if CPU is on an odd CPU cycle, add logic in
        // CPU to track if current cycle is even or odd

        self.incr_cc();
        let page_num = (high_nyb as u16) << 8;
        for address in page_num..page_num + 0xFF {
            let tmp = self.mmu.ld8(address);
            self.mmu.store(OAM_DATA, tmp);
            self.cycle_count += 2;
        }
    }

    fn store(&mut self, addr: u16, val: u8) {
        //println!("Address {:X}, Old val {:X}, New val {:X}", addr,
        // self.mem.ld8(addr), val);
        if addr == DMA_ADDR {
            self.write_dma(val);
        } else {
            self.mmu.store(addr, val);
        }
    }

    fn execute_op(&mut self, op: Op, addr_mode: Option<(AddrDT, bool)>) {
        match addr_mode {
            Some((mode, pb_crossed)) => {
                if pb_crossed {
                    self.incr_cc();
                }
                match mode {
                    AddrDT::Addr(addr) => {
                        let tmp = self.mmu.ld8(addr);
                        match op {
                            // Operandless mirrors (those using the acc addr)
                            Op::Bit(Bit::ROR) => self.ror_addr(addr),
                            Op::Bit(Bit::ASL) => self.asl_addr(addr),
                            Op::Bit(Bit::ROL) => self.rol_addr(addr),
                            Op::Bit(Bit::LSR) => self.lsr_addr(addr),

                            // Imm mirrors
                            Op::Reg(Reg::CPX) => self.cpx(tmp),
                            Op::Reg(Reg::CMP) => self.cmp(tmp),
                            Op::Reg(Reg::CPY) => self.cpy(tmp),
                            Op::Math(Math::SBC) => self.sbc(tmp),
                            Op::Math(Math::ADC) => self.adc(tmp),
                            Op::Store(Store::LDA) => self.lda(tmp),
                            Op::Store(Store::LDX) => self.ldx(tmp),
                            Op::Store(Store::LDY) => self.ldy(tmp),
                            Op::Bit(Bit::EOR) => self.eor(tmp),
                            Op::Bit(Bit::AND) => self.and(tmp),
                            Op::Bit(Bit::ORA) => self.ora(tmp),

                            // Ops without IMP and ACC support
                            Op::Bit(Bit::BIT) => self.bit(tmp),
                            Op::Math(Math::DEC) => self.dec(addr),
                            Op::Math(Math::INC) => self.inc(addr),
                            Op::Jump(Jump::JMP) => self.regs.pc.set_addr(addr),
                            Op::Jump(Jump::JSR) => {
                                self.regs.pc.add_signed(-1);
                                self.push_pc();
                                self.regs.pc.set_addr(addr);
                            }
                            Op::Store(Store::STA) => {
                                let tmp = self.regs.acc;
                                self.store(addr, tmp);
                            }
                            Op::Store(Store::STX) => {
                                let tmp = self.regs.x;
                                self.store(addr, tmp);
                            }
                            Op::Store(Store::STY) => {
                                let tmp = self.regs.y;
                                self.store(addr, tmp);
                            }
                            err => panic!(
                                "Got {:?} as an opcode that needs an address",
                                err
                            ),
                        }
                    }

                    // These are all the immediate opcodes
                    AddrDT::Const(c) => {
                        match op {
                            Op::Reg(Reg::CPX) => self.cpx(c),
                            Op::Reg(Reg::CMP) => self.cmp(c),
                            Op::Reg(Reg::CPY) => self.cpy(c),
                            Op::Math(Math::SBC) => self.sbc(c),
                            Op::Math(Math::ADC) => self.adc(c),
                            Op::Store(Store::LDA) => self.lda(c),
                            Op::Store(Store::LDX) => self.ldx(c),
                            Op::Store(Store::LDY) => self.ldy(c),
                            Op::Bit(Bit::EOR) => self.eor(c),
                            Op::Bit(Bit::AND) => self.and(c),
                            Op::Bit(Bit::ORA) => self.ora(c),
                            err => panic!(
                            "No other instructions support immediate addressing
                             mode. Found {:?}", err),
                        }
                    }
                    AddrDT::Signed(i) => {
                        let flag = match op {
                            Op::Branch(Branch::BCC) => {
                                !self.get_flag(Flag::Carry)
                            }
                            Op::Branch(Branch::BCS) => {
                                self.get_flag(Flag::Carry)
                            }
                            Op::Branch(Branch::BNE) => {
                                !self.get_flag(Flag::Zero)
                            }
                            Op::Branch(Branch::BEQ) => {
                                self.get_flag(Flag::Zero)
                            }
                            Op::Branch(Branch::BPL) => {
                                !self.get_flag(Flag::Neg)
                            }
                            Op::Branch(Branch::BMI) => self.get_flag(Flag::Neg),
                            Op::Branch(Branch::BVC) => {
                                !self.get_flag(Flag::O_f)
                            }
                            Op::Branch(Branch::BVS) => self.get_flag(Flag::O_f),
                            e => panic!("Nothing else uses signed {:?}", e),
                        };
                        self.generic_branch(i, flag);
                    }
                }
            }
            // These are all opcodes without any operands (implied and accum)
            None => {
                match op {
                    // Impl mode
                    Op::Store(Store::TAX) => {
                        let acc = self.regs.acc;
                        self.regs.x = acc;
                        self.set_zero_neg(acc);
                    }
                    Op::Store(Store::TAY) => {
                        let acc = self.regs.acc;
                        self.regs.y = acc;
                        self.set_zero_neg(acc);
                    }
                    Op::Store(Store::TSX) => {
                        let sp = self.regs.sp;
                        self.regs.x = sp;
                        self.set_zero_neg(sp);
                    }
                    Op::Store(Store::TXA) => {
                        let x = self.regs.x;
                        self.regs.acc = x;
                        self.set_zero_neg(x);
                    }
                    Op::Store(Store::TXS) => {
                        let x = self.regs.x;
                        self.regs.sp = x;
                    }
                    Op::Store(Store::TYA) => {
                        let y = self.regs.y;
                        self.regs.acc = y;
                        self.set_zero_neg(y);
                    }
                    Op::Store(Store::PHA) => {
                        let acc = self.regs.acc;
                        self.push(acc);
                    }
                    Op::Store(Store::PHP) => {
                        let flags = self.regs.flags;
                        self.push(flags | Flag::Brk as u8);
                    }
                    Op::Store(Store::PLA) => {
                        let acc = self.pop();
                        self.regs.acc = acc;
                        self.set_zero_neg(acc);
                    }
                    Op::Store(Store::PLP) => {
                        self.pull_status();
                    }
                    Op::Math(Math::DEX) => {
                        let x = self.regs.x.wrapping_sub(1);
                        self.regs.x = x;
                        self.set_zero_neg(x);
                    }
                    Op::Math(Math::DEY) => {
                        let y = self.regs.y.wrapping_sub(1);
                        self.regs.y = y;
                        self.set_zero_neg(y);
                    }
                    Op::Math(Math::INX) => {
                        let x = self.regs.x.wrapping_add(1);
                        self.regs.x = x;
                        self.set_zero_neg(x);
                    }
                    Op::Math(Math::INY) => {
                        let y = self.regs.y.wrapping_add(1);
                        self.regs.y = y;
                        self.set_zero_neg(y);
                    }
                    Op::Jump(Jump::RTI) => {
                        self.pull_status();
                        self.pull_pc();
                    }
                    Op::Jump(Jump::RTS) => {
                        self.pull_pc();
                        self.regs.pc.add_unsigned(1);
                    }
                    Op::Reg(Reg::CLC) => self.set_flag(Flag::Carry, false),
                    Op::Reg(Reg::CLD) => self.set_flag(Flag::Dec, false),
                    Op::Reg(Reg::CLI) => self.set_flag(Flag::Itr, false),
                    Op::Reg(Reg::CLV) => self.set_flag(Flag::O_f, false),
                    Op::Reg(Reg::SEC) => self.set_flag(Flag::Carry, true),
                    Op::Reg(Reg::SED) => self.set_flag(Flag::Dec, true),
                    Op::Reg(Reg::SEI) => self.set_flag(Flag::Itr, true),
                    Op::Sys(Sys::BRK) => self.brk(),
                    Op::Sys(Sys::NOP) => (),

                    // ACC mode
                    Op::Bit(Bit::ROR) => self.ror_acc(),
                    Op::Bit(Bit::ASL) => self.asl_acc(),
                    Op::Bit(Bit::ROL) => self.rol_acc(),
                    Op::Bit(Bit::LSR) => self.lsr_acc(),
                    err => panic!(
                    "Programmer error: All opcodes with implied or accumulator
                     addressing mode have been taken care of: got {:?}", err),
                }
            }
        }
    }

    fn brk(&mut self) {
        self.push_pc();
        let flags = self.regs.flags;
        self.push(flags);
        self.regs.pc.set_addr(IRQ_VEC);
        self.set_flag(Flag::Brk, true);
    }

    fn and(&mut self, val: u8) {
        let tmp = self.regs.acc & val;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn ora(&mut self, val: u8) {
        let tmp = self.regs.acc | val;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn eor(&mut self, val: u8) {
        let tmp = self.regs.acc ^ val;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn adc(&mut self, val: u8) {
        let acc = self.regs.acc;
        let tmp = acc as u16 + val as u16 + self.get_flag(Flag::Carry) as u16;
        self.set_flag(Flag::Carry, tmp > 0xFF);
        self.set_flag(
            Flag::O_f,
            ((acc as u16 ^ tmp) & (val as u16 ^ tmp) & 0x80) != 0,
        );
        let tmp = tmp as u8;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn sbc(&mut self, val: u8) {
        self.adc(val ^ 0xFF);
    }

    fn lda(&mut self, val: u8) {
        self.regs.acc = val;
        self.set_zero_neg(val);
    }

    fn ldx(&mut self, val: u8) {
        self.regs.x = val;
        self.set_zero_neg(val);
    }

    fn ldy(&mut self, val: u8) {
        self.regs.y = val;
        self.set_zero_neg(val);
    }

    fn ror_acc(&mut self) {
        let (tmp, n_flag) =
            Cpu::get_ror(self.get_flag(Flag::Carry), self.regs.acc);
        self.set_flag(Flag::Carry, n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn ror_addr(&mut self, addr: u16) {
        let (tmp, n_flag) =
            Cpu::get_ror(self.get_flag(Flag::Carry), self.mmu.ld8(addr));
        self.set_flag(Flag::Carry, n_flag);
        self.set_zero_neg(tmp);
        self.store(addr, tmp);
    }

    fn get_ror(carry_flag: bool, val: u8) -> (u8, bool) {
        ((val >> 1) | ((carry_flag as u8) << 7), (val & 0b01) != 0)
    }

    fn rol_acc(&mut self) {
        let (tmp, n_flag) =
            Cpu::get_rol(self.get_flag(Flag::Carry), self.regs.acc);
        self.set_flag(Flag::Carry, n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn rol_addr(&mut self, addr: u16) {
        let (tmp, n_flag) =
            Cpu::get_rol(self.get_flag(Flag::Carry), self.mmu.ld8(addr));
        self.set_flag(Flag::Carry, n_flag);
        self.set_zero_neg(tmp);
        self.store(addr, tmp);
    }

    fn get_rol(carry_flag: bool, val: u8) -> (u8, bool) {
        ((val << 1) | (carry_flag as u8), (val & 0x80) != 0)
    }

    fn asl_acc(&mut self) {
        let acc = self.regs.acc;
        self.set_flag(Flag::Carry, (acc >> 7) != 0);
        let tmp = acc << 1;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn asl_addr(&mut self, addr: u16) {
        let val = self.mmu.ld8(addr);
        self.set_flag(Flag::Carry, (val >> 7) != 0);
        let tmp = val << 1;
        self.set_zero_neg(tmp);
        self.store(addr, tmp);
    }

    fn lsr_acc(&mut self) {
        let acc = self.regs.acc;
        self.set_flag(Flag::Carry, (acc & 0b01) != 0);
        let tmp = acc >> 1;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn lsr_addr(&mut self, addr: u16) {
        let val = self.mmu.ld8(addr);
        self.set_flag(Flag::Carry, (val & 0b01) != 0);
        let tmp = val >> 1;
        self.set_zero_neg(tmp);
        self.store(addr, tmp);
    }

    fn cpx(&mut self, val: u8) {
        let tmp = self.regs.x as i16 - val as i16;
        self.set_flag(Flag::Carry, tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn cpy(&mut self, val: u8) {
        let tmp = self.regs.y as i16 - val as i16;
        self.set_flag(Flag::Carry, tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn cmp(&mut self, val: u8) {
        let tmp = self.regs.acc as i16 - val as i16;
        self.set_flag(Flag::Carry, tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn generic_branch(&mut self, val: i8, flag_val: bool) {
        if flag_val {
            let addr = self.regs.pc.get_addr();
            self.regs.pc.add_signed(val);
            self.incr_cc();
            if check_pb(addr, self.regs.pc.get_addr()) {
                self.incr_cc();
            }
        }
    }
    fn bit(&mut self, val: u8) {
        let acc = self.regs.acc;
        self.set_flag(Flag::Zero, (val & acc) == 0);
        self.set_flag(Flag::O_f, (val & 0x40) != 0);
        self.set_flag(Flag::Neg, (val & 0x80) != 0);
    }

    fn dec(&mut self, addr: u16) {
        let val: u8 = self.mmu.ld8(addr).wrapping_sub(1);
        self.set_zero_neg(val);
        self.store(addr, val);
    }

    fn inc(&mut self, addr: u16) {
        let val: u8 = self.mmu.ld8(addr).wrapping_add(1);
        self.set_zero_neg(val);
        self.store(addr, val);
    }

    fn push(&mut self, val: u8) {
        let addr = self.regs.sp as u16 | 0x100;
        self.store(addr, val);
        self.regs.sp -= 1;
    }

    fn pop(&mut self) -> u8 {
        self.regs.sp += 1;
        self.mmu.ld8(self.regs.sp as u16 | 0x100)
    }

    fn pull_pc(&mut self) {
        let low = self.pop();
        let high = self.pop();
        self.regs.pc.set_addr(((high as u16) << 8) | low as u16);
    }

    fn pull_status(&mut self) {
        self.regs.flags = self.pop();
        self.set_flag(Flag::Unused, true);
        self.set_flag(Flag::Brk, false);
    }

    fn push_pc(&mut self) {
        let high = self.regs.pc.get_addr() >> 8;
        let low = self.regs.pc.get_addr();
        self.push(high as u8);
        self.push(low as u8);
    }

    fn set_zero_neg(&mut self, val: u8) {
        self.set_flag(Flag::Neg, val >> 7 == 1);
        self.set_flag(Flag::Zero, val == 0);
    }

    fn set_flag(&mut self, flag: Flag, val: bool) {
        if val {
            self.regs.flags |= flag as u8;
        } else {
            self.regs.flags &= !(flag as u8);
        }
    }

    fn get_flag(&mut self, flag: Flag) -> bool {
        (self.regs.flags & flag as u8) != 0
    }

    pub fn step(&mut self, debug: bool) -> Result<u16, u8> {
        let regs = self.regs.clone();
        let byte = self.ld8_pc_up();
        let cycle = self.cycle_count;
        self.cycle_count += CYCLES[byte as usize] as u16;
        let (op, addr_mode) = self.decode_op(byte)?;
        let addr_data = addr_mode.address_mem(self);
        if debug {
            //println!("{:?} {:?}", op, regs);
        }
        self.execute_op(op, addr_data);
        let tmp = self.cycle_count;
        self.cycle_count = 0;
        Ok(tmp)
    }

    fn ld8_pc_up(&mut self) -> u8 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add_unsigned(1);
        self.mmu.ld8(ram_ptr)
    }

    fn ld16_pc_up(&mut self) -> u16 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add_unsigned(2);
        self.mmu.ld16(ram_ptr)
    }

    pub fn decode_op(&mut self, op: u8) -> Result<(Op, AddrMode), u8> {
        match op {
            INC_ABSX => {
                Ok((Op::Math(Math::INC), AddrMode::AbsX(self.ld16_pc_up())))
            }
            SBC_ABSX => {
                Ok((Op::Math(Math::SBC), AddrMode::AbsX(self.ld16_pc_up())))
            }
            SBC_ABSY => {
                Ok((Op::Math(Math::SBC), AddrMode::AbsY(self.ld16_pc_up())))
            }
            SBC_ABS => {
                Ok((Op::Math(Math::SBC), AddrMode::Abs(self.ld16_pc_up())))
            }
            SBC_INDY => {
                Ok((Op::Math(Math::SBC), AddrMode::IndY(self.ld8_pc_up())))
            }
            SBC_INDX => {
                Ok((Op::Math(Math::SBC), AddrMode::IndX(self.ld8_pc_up())))
            }
            SBC_ZPX => {
                Ok((Op::Math(Math::SBC), AddrMode::ZPX(self.ld8_pc_up())))
            }
            SBC_ZP => Ok((Op::Math(Math::SBC), AddrMode::ZP(self.ld8_pc_up()))),
            INC_ZPX => {
                Ok((Op::Math(Math::INC), AddrMode::ZPX(self.ld8_pc_up())))
            }
            INC_ABS => {
                Ok((Op::Math(Math::INC), AddrMode::Abs(self.ld16_pc_up())))
            }
            INC_ZP => Ok((Op::Math(Math::INC), AddrMode::ZP(self.ld8_pc_up()))),
            CPX_ABS => {
                Ok((Op::Reg(Reg::CPX), AddrMode::Abs(self.ld16_pc_up())))
            }
            CPX_IMM => Ok((Op::Reg(Reg::CPX), AddrMode::Imm(self.ld8_pc_up()))),
            CPX_ZP => Ok((Op::Reg(Reg::CPX), AddrMode::ZP(self.ld8_pc_up()))),
            SBC_IMM => {
                Ok((Op::Math(Math::SBC), AddrMode::Imm(self.ld8_pc_up())))
            }
            CMP_IMM => Ok((Op::Reg(Reg::CMP), AddrMode::Imm(self.ld8_pc_up()))),
            CPY_IMM => Ok((Op::Reg(Reg::CPY), AddrMode::Imm(self.ld8_pc_up()))),
            LDA_IMM => {
                Ok((Op::Store(Store::LDA), AddrMode::Imm(self.ld8_pc_up())))
            }
            LDX_IMM => {
                Ok((Op::Store(Store::LDX), AddrMode::Imm(self.ld8_pc_up())))
            }
            LDY_IMM => {
                Ok((Op::Store(Store::LDY), AddrMode::Imm(self.ld8_pc_up())))
            }
            ADC_IMM => {
                Ok((Op::Math(Math::ADC), AddrMode::Imm(self.ld8_pc_up())))
            }
            EOR_IMM => Ok((Op::Bit(Bit::EOR), AddrMode::Imm(self.ld8_pc_up()))),
            AND_IMM => Ok((Op::Bit(Bit::AND), AddrMode::Imm(self.ld8_pc_up()))),
            ORA_IMM => Ok((Op::Bit(Bit::ORA), AddrMode::Imm(self.ld8_pc_up()))),
            CMP_ABSX => {
                Ok((Op::Reg(Reg::CMP), AddrMode::AbsX(self.ld16_pc_up())))
            }
            CMP_ABSY => {
                Ok((Op::Reg(Reg::CMP), AddrMode::AbsY(self.ld16_pc_up())))
            }
            DEC_ZPX => {
                Ok((Op::Math(Math::DEC), AddrMode::ZPX(self.ld8_pc_up())))
            }
            DEC_ABS => {
                Ok((Op::Math(Math::DEC), AddrMode::Abs(self.ld16_pc_up())))
            }
            DEC_ABSX => {
                Ok((Op::Math(Math::DEC), AddrMode::AbsX(self.ld16_pc_up())))
            }
            DEC_ZP => Ok((Op::Math(Math::DEC), AddrMode::ZP(self.ld8_pc_up()))),
            CMP_ZPX => Ok((Op::Reg(Reg::CMP), AddrMode::ZPX(self.ld8_pc_up()))),
            CMP_INDY => {
                Ok((Op::Reg(Reg::CMP), AddrMode::IndY(self.ld8_pc_up())))
            }
            CMP_ABS => {
                Ok((Op::Reg(Reg::CMP), AddrMode::Abs(self.ld16_pc_up())))
            }
            CMP_ZP => Ok((Op::Reg(Reg::CMP), AddrMode::ZP(self.ld8_pc_up()))),
            CPY_ZP => Ok((Op::Reg(Reg::CPY), AddrMode::ZP(self.ld8_pc_up()))),
            CMP_INDX => {
                Ok((Op::Reg(Reg::CMP), AddrMode::IndX(self.ld8_pc_up())))
            }
            CPY_ABS => {
                Ok((Op::Reg(Reg::CPY), AddrMode::Abs(self.ld16_pc_up())))
            }
            LDA_ABSX => {
                Ok((Op::Store(Store::LDA), AddrMode::AbsX(self.ld16_pc_up())))
            }
            LDA_ABSY => {
                Ok((Op::Store(Store::LDA), AddrMode::AbsY(self.ld16_pc_up())))
            }
            LDA_ZPX => {
                Ok((Op::Store(Store::LDA), AddrMode::ZPX(self.ld8_pc_up())))
            }
            LDA_INDY => {
                Ok((Op::Store(Store::LDA), AddrMode::IndY(self.ld8_pc_up())))
            }
            LDA_ABS => {
                Ok((Op::Store(Store::LDA), AddrMode::Abs(self.ld16_pc_up())))
            }
            LDA_ZP => {
                Ok((Op::Store(Store::LDA), AddrMode::ZP(self.ld8_pc_up())))
            }
            LDA_INDX => {
                Ok((Op::Store(Store::LDA), AddrMode::IndX(self.ld8_pc_up())))
            }
            LDY_ABSX => {
                Ok((Op::Store(Store::LDY), AddrMode::AbsX(self.ld16_pc_up())))
            }
            LDY_ZPX => {
                Ok((Op::Store(Store::LDY), AddrMode::ZPX(self.ld8_pc_up())))
            }
            LDX_ABS => {
                Ok((Op::Store(Store::LDX), AddrMode::Abs(self.ld16_pc_up())))
            }
            LDX_ABSY => {
                Ok((Op::Store(Store::LDX), AddrMode::AbsY(self.ld16_pc_up())))
            }
            LDY_ABS => {
                Ok((Op::Store(Store::LDY), AddrMode::Abs(self.ld16_pc_up())))
            }
            LDX_ZP => {
                Ok((Op::Store(Store::LDX), AddrMode::ZP(self.ld8_pc_up())))
            }
            LDY_ZP => {
                Ok((Op::Store(Store::LDY), AddrMode::ZP(self.ld8_pc_up())))
            }
            STA_ABSX => {
                Ok((Op::Store(Store::STA), AddrMode::AbsX(self.ld16_pc_up())))
            }
            STA_ABSY => {
                Ok((Op::Store(Store::STA), AddrMode::AbsY(self.ld16_pc_up())))
            }
            STA_ZPX => {
                Ok((Op::Store(Store::STA), AddrMode::ZPX(self.ld8_pc_up())))
            }
            STA_INDY => {
                Ok((Op::Store(Store::STA), AddrMode::IndY(self.ld8_pc_up())))
            }
            STA_ABS => {
                Ok((Op::Store(Store::STA), AddrMode::Abs(self.ld16_pc_up())))
            }
            STA_ZP => {
                Ok((Op::Store(Store::STA), AddrMode::ZP(self.ld8_pc_up())))
            }
            STA_INDX => {
                Ok((Op::Store(Store::STA), AddrMode::IndX(self.ld8_pc_up())))
            }
            STX_ABS => {
                Ok((Op::Store(Store::STX), AddrMode::Abs(self.ld16_pc_up())))
            }
            STX_ZP => {
                Ok((Op::Store(Store::STX), AddrMode::ZP(self.ld8_pc_up())))
            }
            STY_ABS => {
                Ok((Op::Store(Store::STY), AddrMode::Abs(self.ld16_pc_up())))
            }
            STY_ZPX => {
                Ok((Op::Store(Store::STY), AddrMode::ZPX(self.ld8_pc_up())))
            }
            STY_ZP => {
                Ok((Op::Store(Store::STY), AddrMode::ZP(self.ld8_pc_up())))
            }
            ROR_ABSX => {
                Ok((Op::Bit(Bit::ROR), AddrMode::AbsX(self.ld16_pc_up())))
            }
            ROR_ZPX => Ok((Op::Bit(Bit::ROR), AddrMode::ZPX(self.ld8_pc_up()))),
            ADC_ABSX => {
                Ok((Op::Math(Math::ADC), AddrMode::AbsX(self.ld16_pc_up())))
            }
            ADC_ABSY => {
                Ok((Op::Math(Math::ADC), AddrMode::AbsY(self.ld16_pc_up())))
            }
            ADC_ZPX => {
                Ok((Op::Math(Math::ADC), AddrMode::ZPX(self.ld8_pc_up())))
            }
            ADC_INDY => {
                Ok((Op::Math(Math::ADC), AddrMode::IndY(self.ld8_pc_up())))
            }
            ADC_ABS => {
                Ok((Op::Math(Math::ADC), AddrMode::Abs(self.ld16_pc_up())))
            }
            ADC_ZP => Ok((Op::Math(Math::ADC), AddrMode::ZP(self.ld8_pc_up()))),
            ADC_INDX => {
                Ok((Op::Math(Math::ADC), AddrMode::IndX(self.ld8_pc_up())))
            }
            ROR_ABS => {
                Ok((Op::Bit(Bit::ROR), AddrMode::Abs(self.ld16_pc_up())))
            }
            ROR_ZP => Ok((Op::Bit(Bit::ROR), AddrMode::ZP(self.ld8_pc_up()))),
            LSR_ABSX => {
                Ok((Op::Bit(Bit::LSR), AddrMode::AbsX(self.ld16_pc_up())))
            }
            EOR_ABSX => {
                Ok((Op::Bit(Bit::EOR), AddrMode::AbsX(self.ld16_pc_up())))
            }
            EOR_ABSY => {
                Ok((Op::Bit(Bit::EOR), AddrMode::AbsY(self.ld16_pc_up())))
            }
            EOR_ZPX => Ok((Op::Bit(Bit::EOR), AddrMode::ZPX(self.ld8_pc_up()))),
            EOR_INDY => {
                Ok((Op::Bit(Bit::EOR), AddrMode::IndY(self.ld8_pc_up())))
            }
            EOR_ABS => {
                Ok((Op::Bit(Bit::EOR), AddrMode::Abs(self.ld16_pc_up())))
            }
            EOR_ZP => Ok((Op::Bit(Bit::EOR), AddrMode::ZP(self.ld8_pc_up()))),
            EOR_INDX => {
                Ok((Op::Bit(Bit::EOR), AddrMode::IndX(self.ld8_pc_up())))
            }
            LSR_ZPX => Ok((Op::Bit(Bit::LSR), AddrMode::ZPX(self.ld8_pc_up()))),
            LSR_ABS => {
                Ok((Op::Bit(Bit::LSR), AddrMode::Abs(self.ld16_pc_up())))
            }
            LSR_ZP => Ok((Op::Bit(Bit::LSR), AddrMode::ZP(self.ld8_pc_up()))),
            JMP_ABS => {
                Ok((Op::Jump(Jump::JMP), AddrMode::Abs(self.ld16_pc_up())))
            }
            ROL_ABSX => {
                Ok((Op::Bit(Bit::ROL), AddrMode::AbsX(self.ld16_pc_up())))
            }
            AND_ABSX => {
                Ok((Op::Bit(Bit::AND), AddrMode::AbsX(self.ld16_pc_up())))
            }
            AND_ABSY => {
                Ok((Op::Bit(Bit::AND), AddrMode::AbsY(self.ld16_pc_up())))
            }
            ROL_ZPX => Ok((Op::Bit(Bit::ROL), AddrMode::ZPX(self.ld8_pc_up()))),
            AND_INDY => {
                Ok((Op::Bit(Bit::AND), AddrMode::IndY(self.ld8_pc_up())))
            }
            ROL_ABS => {
                Ok((Op::Bit(Bit::ROL), AddrMode::Abs(self.ld16_pc_up())))
            }
            AND_ABS => {
                Ok((Op::Bit(Bit::AND), AddrMode::Abs(self.ld16_pc_up())))
            }
            BIT_ABS => {
                Ok((Op::Bit(Bit::BIT), AddrMode::Abs(self.ld16_pc_up())))
            }
            BIT_ZP => Ok((Op::Bit(Bit::BIT), AddrMode::ZP(self.ld8_pc_up()))),
            ROL_ZP => Ok((Op::Bit(Bit::ROL), AddrMode::ZP(self.ld8_pc_up()))),
            AND_ZP => Ok((Op::Bit(Bit::AND), AddrMode::ZP(self.ld8_pc_up()))),
            AND_INDX => {
                Ok((Op::Bit(Bit::AND), AddrMode::IndX(self.ld8_pc_up())))
            }
            ASL_ABSX => {
                Ok((Op::Bit(Bit::ASL), AddrMode::AbsX(self.ld16_pc_up())))
            }
            ORA_ABSX => {
                Ok((Op::Bit(Bit::ORA), AddrMode::AbsX(self.ld16_pc_up())))
            }
            ORA_ABSY => {
                Ok((Op::Bit(Bit::ORA), AddrMode::AbsY(self.ld16_pc_up())))
            }
            ORA_ZPX => Ok((Op::Bit(Bit::ORA), AddrMode::ZPX(self.ld8_pc_up()))),
            ORA_INDY => {
                Ok((Op::Bit(Bit::ORA), AddrMode::IndY(self.ld8_pc_up())))
            }
            ORA_ABS => {
                Ok((Op::Bit(Bit::ORA), AddrMode::Abs(self.ld16_pc_up())))
            }
            ORA_ZP => Ok((Op::Bit(Bit::ORA), AddrMode::ZP(self.ld8_pc_up()))),
            ORA_INDX => {
                Ok((Op::Bit(Bit::ORA), AddrMode::IndX(self.ld8_pc_up())))
            }
            ASL_ZPX => Ok((Op::Bit(Bit::ASL), AddrMode::ZPX(self.ld8_pc_up()))),
            ASL_ABS => {
                Ok((Op::Bit(Bit::ASL), AddrMode::Abs(self.ld16_pc_up())))
            }
            ASL_ZP => Ok((Op::Bit(Bit::ASL), AddrMode::ZP(self.ld8_pc_up()))),
            LDX_ZPY => {
                Ok((Op::Store(Store::LDX), AddrMode::ZPY(self.ld8_pc_up())))
            }
            STX_ZPY => {
                Ok((Op::Store(Store::STX), AddrMode::ZPY(self.ld8_pc_up())))
            }
            AND_ZPX => Ok((Op::Bit(Bit::AND), AddrMode::ZPX(self.ld8_pc_up()))),
            ROR_ACC => Ok((Op::Bit(Bit::ROR), AddrMode::Accum)),
            ASL_ACC => Ok((Op::Bit(Bit::ASL), AddrMode::Accum)),
            ROL_ACC => Ok((Op::Bit(Bit::ROL), AddrMode::Accum)),
            LSR_ACC => Ok((Op::Bit(Bit::LSR), AddrMode::Accum)),
            RTS => Ok((Op::Jump(Jump::RTS), AddrMode::Impl)),
            RTI => Ok((Op::Jump(Jump::RTI), AddrMode::Impl)),
            SED => Ok((Op::Reg(Reg::SED), AddrMode::Impl)),
            CLC => Ok((Op::Reg(Reg::CLC), AddrMode::Impl)),
            SEC => Ok((Op::Reg(Reg::SEC), AddrMode::Impl)),
            CLI => Ok((Op::Reg(Reg::CLI), AddrMode::Impl)),
            SEI => Ok((Op::Reg(Reg::SEI), AddrMode::Impl)),
            CLV => Ok((Op::Reg(Reg::CLV), AddrMode::Impl)),
            CLD => Ok((Op::Reg(Reg::CLD), AddrMode::Impl)),
            NOP | 0x3A | 0x5A | 0x1a | 0x7A | 0xDA | 0xFA => {
                Ok((Op::Sys(Sys::NOP), AddrMode::Impl))
            }

            // DOP: Double NOP
            0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 | 0x82 | 0x89
            | 0xC2 | 0xD4 | 0xE2 | 0xF4 | 0x04 => {
                self.regs.pc.add_signed(1);
                Ok((Op::Sys(Sys::NOP), AddrMode::Impl))
            }

            // TOP: Triple NOP
            0x0C | 0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                self.regs.pc.add_signed(2);
                Ok((Op::Sys(Sys::NOP), AddrMode::Impl))
            }

            BRK => Ok((Op::Sys(Sys::BRK), AddrMode::Impl)),
            TAX => Ok((Op::Store(Store::TAX), AddrMode::Impl)),
            TXA => Ok((Op::Store(Store::TXA), AddrMode::Impl)),
            TAY => Ok((Op::Store(Store::TAY), AddrMode::Impl)),
            TYA => Ok((Op::Store(Store::TYA), AddrMode::Impl)),
            DEX => Ok((Op::Math(Math::DEX), AddrMode::Impl)),
            INX => Ok((Op::Math(Math::INX), AddrMode::Impl)),
            DEY => Ok((Op::Math(Math::DEY), AddrMode::Impl)),
            INY => Ok((Op::Math(Math::INY), AddrMode::Impl)),
            TSX => Ok((Op::Store(Store::TSX), AddrMode::Impl)),
            TXS => Ok((Op::Store(Store::TXS), AddrMode::Impl)),
            PHA => Ok((Op::Store(Store::PHA), AddrMode::Impl)),
            PLA => Ok((Op::Store(Store::PLA), AddrMode::Impl)),
            PHP => Ok((Op::Store(Store::PHP), AddrMode::Impl)),
            PLP => Ok((Op::Store(Store::PLP), AddrMode::Impl)),
            BVS => Ok((
                Op::Branch(Branch::BVS),
                AddrMode::Rel(self.ld8_pc_up() as i8),
            )),
            BVC => Ok((
                Op::Branch(Branch::BVC),
                AddrMode::Rel(self.ld8_pc_up() as i8),
            )),
            BMI => Ok((
                Op::Branch(Branch::BMI),
                AddrMode::Rel(self.ld8_pc_up() as i8),
            )),
            BPL => Ok((
                Op::Branch(Branch::BPL),
                AddrMode::Rel(self.ld8_pc_up() as i8),
            )),
            BNE => Ok((
                Op::Branch(Branch::BNE),
                AddrMode::Rel(self.ld8_pc_up() as i8),
            )),
            BEQ => Ok((
                Op::Branch(Branch::BEQ),
                AddrMode::Rel(self.ld8_pc_up() as i8),
            )),
            BCS => Ok((
                Op::Branch(Branch::BCS),
                AddrMode::Rel(self.ld8_pc_up() as i8),
            )),
            BCC => Ok((
                Op::Branch(Branch::BCC),
                AddrMode::Rel(self.ld8_pc_up() as i8),
            )),
            JSR => Ok((Op::Jump(Jump::JSR), AddrMode::Abs(self.ld16_pc_up()))),
            JMP_IND => {
                Ok((Op::Jump(Jump::JMP), AddrMode::JmpIndir(self.ld16_pc_up())))
            }
            _ => Err(op),
        }
    }
}
