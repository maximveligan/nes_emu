use cpu_const::*;
use mmu::MemManageUnit;
use std::ops::Add;

pub struct Registers {
    pub acc: u8,
    pub x: u8,
    pub y: u8,
    pub pc: ProgramCounter,
    pub sp: u8,
    pub flags: u8,
}

#[derive(Debug, PartialEq)]
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

    fn set_addr(&mut self, addr: u16) {
        self.0 = addr;
    }
    fn get_addr(&self) -> u16 {
        self.0
    }
}

pub struct Cpu {
    pub regs: Registers,
    pub mem: MemManageUnit,
    pub cycle_count: u8,
}

#[derive(Debug)]
pub enum Opcode {
    Store(Store),
    Math(Math),
    BitOp(BitOp),
    Branch(Branch),
    Jump(Jump),
    RegOps(RegOps),
    System(System),
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
pub enum BitOp {
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
pub enum RegOps {
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
pub enum System {
    BRK,
    NOP,
}

#[derive(Debug)]
pub enum InvalidOpcode {
    DoesntExist(String, u8),
}

#[derive(Debug)]
pub enum AddrMode {
    Immediate(u8),
    Implied,
    Accum,
    ZeroPg(u8),
    ZeroPgX(u8),
    ZeroPgY(u8),
    Abs(u16),
    AbsX(u16),
    AbsY(u16),
    JmpIndir(u16),
    IndexIndirX(u8),
    IndirIndexY(u8),
    Relative(i8),
}

#[derive(Debug)]
enum AddrDT {
    Address(u16),
    Constant(u8),
    Signed(i8),
}

impl AddrMode {
    fn address_mem(&self, cpu: &Cpu) -> (Option<AddrDT>, bool) {
        match *self {
            AddrMode::Immediate(v) => (Some(AddrDT::Constant(v)), false),
            AddrMode::Implied => (None, false),
            AddrMode::Accum => (None, false),
            AddrMode::ZeroPg(v) => (Some(AddrDT::Address(v as u16)), false),
            AddrMode::ZeroPgX(v) => (
                Some(AddrDT::Address(v.wrapping_add(cpu.regs.x) as u16)),
                false,
            ),
            AddrMode::ZeroPgY(v) => (
                Some(AddrDT::Address(v.wrapping_add(cpu.regs.y) as u16)),
                false,
            ),
            AddrMode::Abs(v) => (Some(AddrDT::Address(v)), false),
            AddrMode::AbsX(v) => {
                (Some(AddrDT::Address(v + (cpu.regs.x as u16))),
                (v & 0xFF) + cpu.regs.x as u16 > 0xFF)
            }
            AddrMode::AbsY(v) => {
                (Some(AddrDT::Address(v + (cpu.regs.y as u16))),
                (v & 0xFF) + cpu.regs.y as u16 > 0xFF)
            }
            AddrMode::JmpIndir(v) => {
                let low = cpu.mem.load_u8(v);
                let high: u8 = if v & 0xFF  == 0xFF {
                    cpu.mem.load_u8(v - 0xFF)
                } else {
                    cpu.mem.load_u8(v + 1)
                };
                (Some(AddrDT::Address((high as u16) << 8 | (low as u16))),
                 false)
            }
            AddrMode::IndexIndirX(v) => (
                Some(AddrDT::Address(
                    cpu.mem.load_u16(v.wrapping_add(cpu.regs.x) as u16),
                )),
                false,
            ),
            AddrMode::IndirIndexY(v) => {
                let tmp = cpu.mem.load_u16(v as u16);
                (Some(AddrDT::Address(tmp.wrapping_add(cpu.regs.y as u16),
                )),
                (tmp & 0xFF) + cpu.regs.y as u16 > 0xFF)
            }
            AddrMode::Relative(v) => {
                (Some(AddrDT::Signed(v as i8)), false)
            }
        }
    }
}

impl Cpu {
    fn new() -> Cpu {
        Cpu {
            cycle_count: 0,
            regs: Registers {
                acc: 0,
                x: 1,
                y: 2,
                pc: ProgramCounter::new(0),
                sp: 0,
                flags: 0,
            },
            mem: MemManageUnit::new(),
        }
    }

    fn incr_cc(&mut self) {
        self.cycle_count += 1;
    }

    fn execute_op(&mut self, op: Opcode, addr_mode: Option<AddrDT>) {
        match addr_mode {
            Some(mode) => match mode {
                AddrDT::Address(addr) => match op {
                    // Operandless mirrors (those using the acc addr mode)
                    Opcode::BitOp(BitOp::ROR) => self.ror_addr(addr),
                    Opcode::BitOp(BitOp::ASL) => self.asl_addr(addr),
                    Opcode::BitOp(BitOp::ROL) => self.rol_addr(addr),
                    Opcode::BitOp(BitOp::LSR) => self.lsr_addr(addr),

                    // Immediate mirrors
                    Opcode::RegOps(RegOps::CPX) => {
                        let tmp = self.mem.load_u8(addr);
                        self.cpx(tmp);
                    }
                    Opcode::RegOps(RegOps::CMP) => {
                        let tmp = self.mem.load_u8(addr);
                        self.cmp(tmp);
                    }
                    Opcode::RegOps(RegOps::CPY) => {
                        let tmp = self.mem.load_u8(addr);
                        self.cpy(tmp);
                    }
                    Opcode::Math(Math::SBC) => {
                        let tmp = self.mem.load_u8(addr);
                        self.sbc(tmp);
                    }
                    Opcode::Math(Math::ADC) => {
                        let tmp = self.mem.load_u8(addr);
                        self.adc(tmp);
                    }
                    Opcode::Store(Store::LDA) => {
                        let tmp = self.mem.load_u8(addr);
                        self.lda(tmp);
                    }
                    Opcode::Store(Store::LDX) => {
                        let tmp = self.mem.load_u8(addr);
                        self.ldx(tmp);
                    }
                    Opcode::Store(Store::LDY) => {
                        let tmp = self.mem.load_u8(addr);
                        self.ldy(tmp);
                    }
                    Opcode::BitOp(BitOp::EOR) => {
                        let tmp = self.mem.load_u8(addr);
                        self.eor(tmp);
                    }
                    Opcode::BitOp(BitOp::AND) => {
                        let tmp = self.mem.load_u8(addr);
                        self.and(tmp);
                    },
                    Opcode::BitOp(BitOp::ORA) => {
                        let tmp = self.mem.load_u8(addr);
                        self.ora(tmp);
                    },

                    // Opcodes without IMP and ACC support
                    Opcode::BitOp(BitOp::BIT) => {
                        let tmp = self.mem.load_u8(addr);
                        self.bit(tmp);
                    }
                    Opcode::Math(Math::DEC) => self.dec(addr),
                    Opcode::Math(Math::INC) => self.inc(addr),
                    Opcode::Jump(Jump::JMP) => self.regs.pc.set_addr(addr),
                    Opcode::Jump(Jump::JSR) => {
                        self.regs.pc.add_signed(-1);
                        self.push_pc();
                    }
                    Opcode::Store(Store::STA) => {
                        let tmp = self.regs.acc;
                        self.mem.store_u8(addr, tmp);
                    }
                    Opcode::Store(Store::STX) => {
                        let tmp = self.regs.x;
                        self.mem.store_u8(addr, tmp);
                    }
                    Opcode::Store(Store::STY) => {
                        let tmp = self.regs.y;
                        self.mem.store_u8(addr, tmp);
                    }
                    err => panic!(
                        "Got {:?} as an opcode that needs an address", err)
                }

                // These are all the immediate opcodes
                AddrDT::Constant(c) => match op {
                    Opcode::RegOps(RegOps::CPX) => self.cpx(c),
                    Opcode::RegOps(RegOps::CMP) => self.cmp(c),
                    Opcode::RegOps(RegOps::CPY) => self.cpy(c),
                    Opcode::Math(Math::SBC) => self.sbc(c),
                    Opcode::Math(Math::ADC) => self.adc(c),
                    Opcode::Store(Store::LDA) => self.lda(c),
                    Opcode::Store(Store::LDX) => self.ldx(c),
                    Opcode::Store(Store::LDY) => self.ldy(c),
                    Opcode::BitOp(BitOp::EOR) => self.eor(c),
                    Opcode::BitOp(BitOp::AND) => self.and(c),
                    Opcode::BitOp(BitOp::ORA) => self.ora(c),
                    err => panic!(
                        "No other instructions support immediate addressing
                         mode. Found {:?}", err)
                },
                AddrDT::Signed(i) => {
                    let flag = match op {
                        Opcode::Branch(Branch::BCC) => !self.get_flag(CARRY),
                        Opcode::Branch(Branch::BCS) => self.get_flag(CARRY),
                        Opcode::Branch(Branch::BNE) => !self.get_flag(ZERO),
                        Opcode::Branch(Branch::BEQ) => self.get_flag(ZERO),
                        Opcode::Branch(Branch::BPL) => !self.get_flag(NEG),
                        Opcode::Branch(Branch::BMI) => self.get_flag(NEG),
                        Opcode::Branch(Branch::BVC) => !self.get_flag(O_F),
                        Opcode::Branch(Branch::BVS) => self.get_flag(O_F),
                        err => panic!("Nothing else uses signed {:?}", err),
                    };
                    self.generic_branch(i, flag);
                }
            }

            // These are all opcodes without any operands (implied and accum)
            None => match op {
                // Implied mode
                Opcode::Store(Store::TAX) => {
                    let acc = self.regs.acc;
                    self.regs.x = acc;
                    self.set_zero_neg(acc);
                }
                Opcode::Store(Store::TAY) => {
                    let acc = self.regs.acc;
                    self.regs.y = acc;
                    self.set_zero_neg(acc);
                }
                Opcode::Store(Store::TSX) => {
                    let sp = self.regs.sp;
                    self.regs.x = sp;
                    self.set_zero_neg(sp);
                }
                Opcode::Store(Store::TXA) => {
                    let x = self.regs.x;
                    self.regs.acc = x;
                    self.set_zero_neg(x);
                }
                Opcode::Store(Store::TXS) => {
                    let x = self.regs.x;
                    self.regs.sp = x;
                    self.set_zero_neg(x);
                }
                Opcode::Store(Store::TYA) => {
                    let y = self.regs.y;
                    self.regs.acc = y;
                    self.set_zero_neg(y);
                }
                Opcode::Store(Store::PHA) => {
                    let acc = self.regs.acc;
                    self.push(acc);
                }
                Opcode::Store(Store::PHP) => {
                    self.set_flag(BRK_F, true);
                    let flags = self.regs.flags;
                    self.push(flags);
                }
                Opcode::Store(Store::PLA) => {
                    let acc = self.pop();
                    self.regs.acc = acc;
                    self.set_zero_neg(acc);
                }
                Opcode::Store(Store::PLP) => {
                    self.regs.flags = self.pop();
                }
                Opcode::Math(Math::DEX) => {
                    let x = self.regs.x.wrapping_sub(1);
                    self.regs.x = x;
                    self.set_zero_neg(x);
                }
                Opcode::Math(Math::DEY) => {
                    let y = self.regs.y.wrapping_sub(1);
                    self.regs.y = y;
                    self.set_zero_neg(y);
                }
                Opcode::Math(Math::INX) => {
                    let x = self.regs.x.wrapping_add(1);
                    self.regs.x = x;
                    self.set_zero_neg(x);
                }
                Opcode::Math(Math::INY) => {
                    let y = self.regs.y.wrapping_add(1);
                    self.regs.y = y;
                    self.set_zero_neg(y);
                }
                Opcode::Jump(Jump::RTI) => {
                    self.regs.flags = self.pop();
                    self.pull_pc();
                }
                Opcode::Jump(Jump::RTS) => {
                    self.pull_pc();
                    self.regs.pc.add_unsigned(1);
                }
                Opcode::RegOps(RegOps::CLC) => self.set_flag(CARRY, false),
                Opcode::RegOps(RegOps::CLD) => self.set_flag(DEC, false),
                Opcode::RegOps(RegOps::CLI) => self.set_flag(ITR, false),
                Opcode::RegOps(RegOps::CLV) => self.set_flag(O_F, false),
                Opcode::RegOps(RegOps::SEC) => self.set_flag(CARRY, true),
                Opcode::RegOps(RegOps::SED) => self.set_flag(DEC, true),
                Opcode::RegOps(RegOps::SEI) => self.set_flag(ITR, true),
                Opcode::System(System::BRK) => unimplemented!(),

                // ACC mode
                Opcode::BitOp(BitOp::ROR) => self.ror_acc(),
                Opcode::BitOp(BitOp::ASL) => self.asl_acc(),
                Opcode::BitOp(BitOp::ROL) => self.rol_acc(),
                Opcode::BitOp(BitOp::LSR) => self.lsr_acc(),
                err => panic!(
                    "Programmer error: All opcodes with implied or accumulator
                     addressing mode have been taken care of: got {:?}", err),
            }
        }
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
        let tmp = acc as u16 + val as u16 + self.get_flag(CARRY) as u16;
        self.set_flag(CARRY, tmp > 0xFF);
        self.set_flag(
            O_F,
            ((acc as u16 ^ tmp) & (val as u16 ^ tmp) & 0x80) != 0,
        );
        let tmp = tmp as u8;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn sbc(&mut self, val: u8) {
        let acc = self.regs.acc;
        let tmp =
            acc as i16 - val as i16 - (1 - self.get_flag(CARRY) as i16);
        self.set_flag(CARRY, tmp >= 0);
        self.set_flag(
            O_F,
            ((acc as i16 ^ tmp) & (val as i16 ^ tmp) & 0x80) != 0,
        );
        let tmp = tmp as u8;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp as u8;
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
            Cpu::get_ror(self.get_flag(CARRY), self.regs.acc);
        self.set_flag(CARRY, n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn ror_addr(&mut self, addr: u16) {
        let (tmp, n_flag) =
            Cpu::get_ror(self.get_flag(CARRY), self.mem.load_u8(addr));
        self.set_flag(CARRY, n_flag);
        self.set_zero_neg(tmp);
        self.mem.store_u8(addr, tmp);
    }

    fn get_ror(carry_flag: bool, val: u8) -> (u8, bool) {
        ((val >> 1) | ((carry_flag as u8) << 7), (val & 0b01) != 0)
    }

    fn rol_acc(&mut self) {
        let (tmp, n_flag) =
            Cpu::get_rol(self.get_flag(CARRY), self.regs.acc);
        self.set_flag(CARRY, n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn rol_addr(&mut self, addr: u16) {
        let (tmp, n_flag) =
            Cpu::get_rol(self.get_flag(CARRY), self.mem.load_u8(addr));
        self.set_flag(CARRY, n_flag);
        self.set_zero_neg(tmp);
        self.mem.store_u8(addr, tmp);
    }

    fn get_rol(carry_flag: bool, val: u8) -> (u8, bool) {
        ((val << 1) | (carry_flag as u8), (val & 0x80) != 0)
    }

    fn asl_acc(&mut self) {
        let acc = self.regs.acc;
        self.set_flag(CARRY, (acc >> 7) != 0);
        let tmp = acc << 1;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn asl_addr(&mut self, addr: u16) {
        let val = self.mem.load_u8(addr);
        self.set_flag(CARRY, (val >> 7) != 0);
        let tmp = val << 1;
        self.set_zero_neg(tmp);
        self.mem.store_u8(addr, val);
    }

    fn lsr_acc(&mut self) {
        let acc = self.regs.acc;
        self.set_flag(CARRY, (acc & 0b01) != 0);
        let tmp = acc >> 1;
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn lsr_addr(&mut self, addr: u16) {
        let val = self.mem.load_u8(addr);
        self.set_flag(CARRY, (val & 0b01) != 0);
        let tmp = val >> 1;
        self.set_zero_neg(tmp);
        self.mem.store_u8(addr, val);
    }

    fn cpx(&mut self, val: u8) {
        let tmp = self.regs.x as i16 - val as i16;
        self.set_flag(CARRY, tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn cpy(&mut self, val: u8) {
        let tmp = self.regs.y as i16 - val as i16;
        self.set_flag(CARRY, tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn cmp(&mut self, val: u8) {
        let tmp = self.regs.acc as i16 - val as i16;
        self.set_flag(CARRY, tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn generic_branch(&mut self, val: i8, flag_val: bool) {
        if flag_val {
            self.regs.pc.add_signed(val);
        }
    }
    fn bit(&mut self, val: u8) {
        let acc = self.regs.acc;
        self.set_flag(ZERO, (val & acc) == 0);
        self.set_flag(O_F, (val & 0x40) != 0);
        self.set_flag(NEG, (val & 0x80) != 0);
    }

    fn dec(&mut self, addr: u16) {
        let val: u8 = self.mem.load_u8(addr).wrapping_sub(1);
        self.set_zero_neg(val);
        self.mem.store_u8(addr, val);
    }

    fn inc(&mut self, addr: u16) {
        let val: u8 = self.mem.load_u8(addr).wrapping_add(1);
        self.set_zero_neg(val);
        self.mem.store_u8(addr, val);
    }

    fn push(&mut self, val: u8) {
        self.mem.store_u8(self.regs.sp as u16 | 0x100, val);
        self.regs.sp -= 1;
    }

    fn pop(&mut self) -> u8 {
        self.regs.sp += 1;
        self.mem.load_u8(self.regs.sp as u16 | 0x100)
    }

    fn pull_pc(&mut self) {
        let low = self.pop();
        let high = self.pop();
        self.regs.pc.set_addr(((high as u16) << 8) | low as u16);
    }

    fn push_pc(&mut self) {
        let high = (self.regs.pc.get_addr() >> 8);
        let low = self.regs.pc.get_addr();
        self.push(high as u8);
        self.push(low as u8);
    }

    fn set_zero_neg(&mut self, val: u8) {
        self.set_flag(NEG, val >> 7 == 1);
        self.set_flag(ZERO, val == 0);
    }

    fn set_flag(&mut self, flag: u8, val: bool) {
        if val {
            self.regs.flags |= flag;
        } else {
            self.regs.flags &= !flag;
        }
    }

    fn get_flag(&mut self, flag: u8) -> bool {
        (self.regs.flags & flag) != 0
    }

    pub fn step(&mut self) -> Result<(), InvalidOpcode> {
        let byte = self.loadu8_pc_incr();
        let (op, addr_mode) = self.decode_op(byte)?;
        let (address, page_bounary_crossed) = addr_mode.address_mem(&self);
        self.execute_op(op, address);
        Ok(())
    }

    fn loadu8_pc_incr(&mut self) -> u8 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add_unsigned(1);
        self.mem.load_u8(ram_ptr)
    }

    fn loadu16_pc_incr(&mut self) -> u16 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add_unsigned(2);
        self.mem.load_u16(ram_ptr)
    }

    pub fn decode_op(
        &mut self,
        op: u8,
    ) -> Result<(Opcode, AddrMode), InvalidOpcode> {
        self.cycle_count += CYCLES[op as usize];
        match op {
            INC_ABSX => Ok((
                Opcode::Math(Math::INC),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            SBC_ABSX => Ok((
                Opcode::Math(Math::SBC),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            SBC_ABSY => Ok((
                Opcode::Math(Math::SBC),
                AddrMode::AbsY(self.loadu16_pc_incr()),
            )),
            SBC_INDY => Ok((
                Opcode::Math(Math::SBC),
                AddrMode::IndirIndexY(self.loadu8_pc_incr()),
            )),
            SBC_INDX => Ok((
                Opcode::Math(Math::SBC),
                AddrMode::IndexIndirX(self.loadu8_pc_incr()),
            )),
            SBC_ZPX => Ok((
                Opcode::Math(Math::SBC),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            SBC_ZP => Ok((
                Opcode::Math(Math::SBC),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            INC_ZPX => Ok((
                Opcode::Math(Math::INC),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            INC_ABS => Ok((
                Opcode::Math(Math::INC),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            INC_ZP => Ok((
                Opcode::Math(Math::INC),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            CPX_ABS => Ok((
                Opcode::RegOps(RegOps::CPX),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            CPX_ZP => Ok((
                Opcode::RegOps(RegOps::CPX),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            CPX_IMM => Ok((
                Opcode::RegOps(RegOps::CPX),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            SBC_IMM => Ok((
                Opcode::Math(Math::SBC),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            CMP_IMM => Ok((
                Opcode::RegOps(RegOps::CMP),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            CPY_IMM => Ok((
                Opcode::RegOps(RegOps::CPY),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            LDA_IMM => Ok((
                Opcode::Store(Store::LDA),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            LDX_IMM => Ok((
                Opcode::Store(Store::LDX),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            LDY_IMM => Ok((
                Opcode::Store(Store::LDY),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            ADC_IMM => Ok((
                Opcode::Math(Math::ADC),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            EOR_IMM => Ok((
                Opcode::BitOp(BitOp::EOR),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            AND_IMM => Ok((
                Opcode::BitOp(BitOp::AND),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            ORA_IMM => Ok((
                Opcode::BitOp(BitOp::ORA),
                AddrMode::Immediate(self.loadu8_pc_incr()),
            )),
            CMP_ABSX => Ok((
                Opcode::RegOps(RegOps::CMP),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            CMP_ABSY => Ok((
                Opcode::RegOps(RegOps::CMP),
                AddrMode::AbsY(self.loadu16_pc_incr()),
            )),
            DEC_ZPX => Ok((
                Opcode::Math(Math::DEC),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            DEC_ABS => Ok((
                Opcode::Math(Math::DEC),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            DEC_ZP => Ok((
                Opcode::Math(Math::DEC),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            CMP_ZPX => Ok((
                Opcode::RegOps(RegOps::CMP),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            CMP_INDY => Ok((
                Opcode::RegOps(RegOps::CMP),
                AddrMode::IndirIndexY(self.loadu8_pc_incr()),
            )),
            CMP_ABS => Ok((
                Opcode::RegOps(RegOps::CMP),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            CMP_ZP => Ok((
                Opcode::RegOps(RegOps::CMP),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            CPY_ZP => Ok((
                Opcode::RegOps(RegOps::CPY),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            CMP_INDX => Ok((
                Opcode::RegOps(RegOps::CMP),
                AddrMode::IndexIndirX(self.loadu8_pc_incr()),
            )),
            LDA_ABSX => Ok((
                Opcode::Store(Store::LDA),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            LDA_ABSY => Ok((
                Opcode::Store(Store::LDA),
                AddrMode::AbsY(self.loadu16_pc_incr()),
            )),
            LDA_ZPX => Ok((
                Opcode::Store(Store::LDA),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            LDA_INDY => Ok((
                Opcode::Store(Store::LDA),
                AddrMode::IndirIndexY(self.loadu8_pc_incr()),
            )),
            LDA_ABS => Ok((
                Opcode::Store(Store::LDA),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            LDA_ZP => Ok((
                Opcode::Store(Store::LDA),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            LDA_INDX => Ok((
                Opcode::Store(Store::LDA),
                AddrMode::IndexIndirX(self.loadu8_pc_incr()),
            )),
            LDY_ABSX => Ok((
                Opcode::Store(Store::LDY),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            LDY_ZPX => Ok((
                Opcode::Store(Store::LDY),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            LDX_ABS => Ok((
                Opcode::Store(Store::LDX),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            LDY_ABS => Ok((
                Opcode::Store(Store::LDY),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            LDX_ZP => Ok((
                Opcode::Store(Store::LDX),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            LDY_ZP => Ok((
                Opcode::Store(Store::LDY),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            STA_ABSX => Ok((
                Opcode::Store(Store::STA),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            STA_ABSY => Ok((
                Opcode::Store(Store::STA),
                AddrMode::AbsY(self.loadu16_pc_incr()),
            )),
            STA_ZPX => Ok((
                Opcode::Store(Store::STA),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            STA_INDY => Ok((
                Opcode::Store(Store::STA),
                AddrMode::IndirIndexY(self.loadu8_pc_incr()),
            )),
            STA_ABS => Ok((
                Opcode::Store(Store::STA),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            STA_ZP => Ok((
                Opcode::Store(Store::STA),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            STA_INDX => Ok((
                Opcode::Store(Store::STA),
                AddrMode::IndexIndirX(self.loadu8_pc_incr()),
            )),
            STX_ABS => Ok((
                Opcode::Store(Store::STX),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            STX_ZP => Ok((
                Opcode::Store(Store::STX),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            STY_ABS => Ok((
                Opcode::Store(Store::STY),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            STY_ZPX => Ok((
                Opcode::Store(Store::STY),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            STY_ZP => Ok((
                Opcode::Store(Store::STY),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            ROR_ABSX => Ok((
                Opcode::BitOp(BitOp::ROR),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            ROR_ZPX => Ok((
                Opcode::BitOp(BitOp::ROR),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            ADC_ABSX => Ok((
                Opcode::Math(Math::ADC),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            ADC_ABSY => Ok((
                Opcode::Math(Math::ADC),
                AddrMode::AbsY(self.loadu16_pc_incr()),
            )),
            ADC_ZPX => Ok((
                Opcode::Math(Math::ADC),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            ADC_INDY => Ok((
                Opcode::Math(Math::ADC),
                AddrMode::IndirIndexY(self.loadu8_pc_incr()),
            )),
            ADC_ABS => Ok((
                Opcode::Math(Math::ADC),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            ADC_ZP => Ok((
                Opcode::Math(Math::ADC),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            ADC_INDX => Ok((
                Opcode::Math(Math::ADC),
                AddrMode::IndexIndirX(self.loadu8_pc_incr()),
            )),
            ROR_ABS => Ok((
                Opcode::BitOp(BitOp::ROR),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            ROR_ZP => Ok((
                Opcode::BitOp(BitOp::ROR),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            LSR_ABSX => Ok((
                Opcode::BitOp(BitOp::LSR),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            EOR_ABSX => Ok((
                Opcode::BitOp(BitOp::EOR),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            EOR_ABSY => Ok((
                Opcode::BitOp(BitOp::EOR),
                AddrMode::AbsY(self.loadu16_pc_incr()),
            )),
            EOR_ZPX => Ok((
                Opcode::BitOp(BitOp::EOR),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            EOR_INDY => Ok((
                Opcode::BitOp(BitOp::EOR),
                AddrMode::IndirIndexY(self.loadu8_pc_incr()),
            )),
            EOR_ABS => Ok((
                Opcode::BitOp(BitOp::EOR),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            EOR_ZP => Ok((
                Opcode::BitOp(BitOp::EOR),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            EOR_INDX => Ok((
                Opcode::BitOp(BitOp::EOR),
                AddrMode::IndexIndirX(self.loadu8_pc_incr()),
            )),
            LSR_ZPX => Ok((
                Opcode::BitOp(BitOp::LSR),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            LSR_ABS => Ok((
                Opcode::BitOp(BitOp::LSR),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            LSR_ZP => Ok((
                Opcode::BitOp(BitOp::LSR),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            JMP_ABS => Ok((
                Opcode::Jump(Jump::JMP),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            ROL_ABSX => Ok((
                Opcode::BitOp(BitOp::ROL),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            AND_ABSX => Ok((
                Opcode::BitOp(BitOp::AND),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            AND_ABSY => Ok((
                Opcode::BitOp(BitOp::AND),
                AddrMode::AbsY(self.loadu16_pc_incr()),
            )),
            ROL_ZPX => Ok((
                Opcode::BitOp(BitOp::ROL),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            AND_INDY => Ok((
                Opcode::BitOp(BitOp::AND),
                AddrMode::IndirIndexY(self.loadu8_pc_incr()),
            )),
            ROL_ABS => Ok((
                Opcode::BitOp(BitOp::ROL),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            AND_ABS => Ok((
                Opcode::BitOp(BitOp::AND),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            BIT_ABS => Ok((
                Opcode::BitOp(BitOp::BIT),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            BIT_ZP => Ok((
                Opcode::BitOp(BitOp::BIT),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            ROL_ZP => Ok((
                Opcode::BitOp(BitOp::ROL),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            AND_ZP => Ok((
                Opcode::BitOp(BitOp::AND),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            AND_INDX => Ok((
                Opcode::BitOp(BitOp::AND),
                AddrMode::IndexIndirX(self.loadu8_pc_incr()),
            )),
            ASL_ABSX => Ok((
                Opcode::BitOp(BitOp::ASL),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            ORA_ABSX => Ok((
                Opcode::BitOp(BitOp::ORA),
                AddrMode::AbsX(self.loadu16_pc_incr()),
            )),
            ORA_ABSY => Ok((
                Opcode::BitOp(BitOp::ORA),
                AddrMode::AbsY(self.loadu16_pc_incr()),
            )),
            ORA_ZPX => Ok((
                Opcode::BitOp(BitOp::ORA),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            ORA_INDY => Ok((
                Opcode::BitOp(BitOp::ORA),
                AddrMode::IndirIndexY(self.loadu8_pc_incr()),
            )),
            ORA_ABS => Ok((
                Opcode::BitOp(BitOp::ORA),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            ORA_ZP => Ok((
                Opcode::BitOp(BitOp::ORA),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            ORA_INDX => Ok((
                Opcode::BitOp(BitOp::ORA),
                AddrMode::IndexIndirX(self.loadu8_pc_incr()),
            )),
            ASL_ZPX => Ok((
                Opcode::BitOp(BitOp::ASL),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            ASL_ABS => Ok((
                Opcode::BitOp(BitOp::ASL),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            ASL_ZP => Ok((
                Opcode::BitOp(BitOp::ASL),
                AddrMode::ZeroPg(self.loadu8_pc_incr()),
            )),
            LDX_ZPY => Ok((
                Opcode::Store(Store::LDX),
                AddrMode::ZeroPgY(self.loadu8_pc_incr()),
            )),
            STX_ZPY => Ok((
                Opcode::Store(Store::STX),
                AddrMode::ZeroPgY(self.loadu8_pc_incr()),
            )),
            AND_ZPX => Ok((
                Opcode::BitOp(BitOp::AND),
                AddrMode::ZeroPgX(self.loadu8_pc_incr()),
            )),
            ROR_ACC => Ok((Opcode::BitOp(BitOp::ROR), AddrMode::Accum)),
            ASL_ACC => Ok((Opcode::BitOp(BitOp::ASL), AddrMode::Accum)),
            ROL_ACC => Ok((Opcode::BitOp(BitOp::ROL), AddrMode::Accum)),
            LSR_ACC => Ok((Opcode::BitOp(BitOp::LSR), AddrMode::Accum)),
            RTS => Ok((Opcode::Jump(Jump::RTS), AddrMode::Implied)),
            RTI => Ok((Opcode::Jump(Jump::RTI), AddrMode::Implied)),
            SED => Ok((Opcode::RegOps(RegOps::SED), AddrMode::Implied)),
            CLC => Ok((Opcode::RegOps(RegOps::CLC), AddrMode::Implied)),
            SEC => Ok((Opcode::RegOps(RegOps::SEC), AddrMode::Implied)),
            CLI => Ok((Opcode::RegOps(RegOps::CLI), AddrMode::Implied)),
            SEI => Ok((Opcode::RegOps(RegOps::SEI), AddrMode::Implied)),
            CLV => Ok((Opcode::RegOps(RegOps::CLV), AddrMode::Implied)),
            CLD => Ok((Opcode::RegOps(RegOps::CLD), AddrMode::Implied)),
            NOP => Ok((Opcode::System(System::NOP), AddrMode::Implied)),
            BRK => Ok((Opcode::System(System::BRK), AddrMode::Implied)),
            TAX => Ok((Opcode::Store(Store::TAX), AddrMode::Implied)),
            TXA => Ok((Opcode::Store(Store::TXA), AddrMode::Implied)),
            TAY => Ok((Opcode::Store(Store::TAY), AddrMode::Implied)),
            TYA => Ok((Opcode::Store(Store::TYA), AddrMode::Implied)),
            DEX => Ok((Opcode::Math(Math::DEX), AddrMode::Implied)),
            INX => Ok((Opcode::Math(Math::INX), AddrMode::Implied)),
            DEY => Ok((Opcode::Math(Math::DEY), AddrMode::Implied)),
            INY => Ok((Opcode::Math(Math::INY), AddrMode::Implied)),
            TSX => Ok((Opcode::Store(Store::TSX), AddrMode::Implied)),
            TXS => Ok((Opcode::Store(Store::TXS), AddrMode::Implied)),
            PHA => Ok((Opcode::Store(Store::PHA), AddrMode::Implied)),
            PLA => Ok((Opcode::Store(Store::PLA), AddrMode::Implied)),
            PHP => Ok((Opcode::Store(Store::PHP), AddrMode::Implied)),
            PLP => Ok((Opcode::Store(Store::PLP), AddrMode::Implied)),
            BVS => Ok((
                Opcode::Branch(Branch::BVS),
                AddrMode::Relative(self.loadu8_pc_incr() as i8),
            )),
            BVC => Ok((
                Opcode::Branch(Branch::BVC),
                AddrMode::Relative(self.loadu8_pc_incr() as i8),
            )),
            BMI => Ok((
                Opcode::Branch(Branch::BMI),
                AddrMode::Relative(self.loadu8_pc_incr() as i8),
            )),
            BPL => Ok((
                Opcode::Branch(Branch::BPL),
                AddrMode::Relative(self.loadu8_pc_incr() as i8),
            )),
            BNE => Ok((
                Opcode::Branch(Branch::BNE),
                AddrMode::Relative(self.loadu8_pc_incr() as i8),
            )),
            BEQ => Ok((
                Opcode::Branch(Branch::BEQ),
                AddrMode::Relative(self.loadu8_pc_incr() as i8),
            )),
            BCS => Ok((
                Opcode::Branch(Branch::BCS),
                AddrMode::Relative(self.loadu8_pc_incr() as i8),
            )),
            BCC => Ok((
                Opcode::Branch(Branch::BCC),
                AddrMode::Relative(self.loadu8_pc_incr() as i8),
            )),
            JSR => Ok((
                Opcode::Jump(Jump::JSR),
                AddrMode::Abs(self.loadu16_pc_incr()),
            )),
            JMP_IND => Ok((
                Opcode::Jump(Jump::JMP),
                AddrMode::JmpIndir(self.loadu16_pc_incr()),
            )),
            _ => Err(InvalidOpcode::DoesntExist(
                "Unsupported op".to_string(),
                op,
            )),
        }
    }
}
