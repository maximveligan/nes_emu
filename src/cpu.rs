use cpu_const::*;
use std::fmt;
use mmu::MemManageUnit;
use mapper::Mapper;

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
            "Registers\n{:?}\nAcc: 0x{:X}, Dec {}\nX: 0x{:X}, Dec {}\n\
             Y: 0x{:X}, Dec {}\nSP: 0x{:X}, Dec {}\nStatus: 0b{:b}",
            self.pc,
            self.acc,
            self.acc,
            self.x,
            self.x,
            self.y,
            self.y,
            self.sp,
            self.sp,
            self.flags
        )
    }
}

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

impl fmt::Debug for ProgramCounter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PC: 0x{:X}, Dec {}", self.0, self.0)
    }
}

pub struct Cpu {
    pub regs: Registers,
    pub mem: MemManageUnit,
    pub cycle_count: u8,
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

#[derive(Debug)]
pub enum InvalidOp {
    DoesntExist(String, u8),
}

#[derive(Debug)]
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
    IndexIndirX(u8),
    IndirIndexY(u8),
    Rel(i8),
}

#[derive(Debug)]
enum AddrDT {
    Addr(u16),
    Const(u8),
    Signed(i8),
}

impl AddrMode {
    fn address_mem(&self, cpu: &Cpu) -> Option<(AddrDT, bool)> {
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
            AddrMode::AbsY(v) => Some((
                AddrDT::Addr(v + (cpu.regs.y as u16)),
                check_pb(v, v + cpu.regs.y as u16),
            )),
            AddrMode::JmpIndir(v) => {
                let low = cpu.mem.load_u8(v);
                let high: u8 = if v & 0xFF == 0xFF {
                    cpu.mem.load_u8(v - 0xFF)
                } else {
                    cpu.mem.load_u8(v + 1)
                };
                Some((AddrDT::Addr((high as u16) << 8 | (low as u16)), false))
            }
            AddrMode::IndexIndirX(v) => Some((
                AddrDT::Addr(
                    cpu.mem.load_u16(v.wrapping_add(cpu.regs.x) as u16),
                ),
                false,
            )),
            AddrMode::IndirIndexY(v) => {
                let tmp = cpu.mem.load_u16(v as u16);
                Some((
                    AddrDT::Addr(tmp.wrapping_add(cpu.regs.y as u16)),
                    check_pb(tmp, tmp + cpu.regs.y as u16),
                ))
            }
            AddrMode::Rel(v) => Some((AddrDT::Signed(v as i8), false)),
        }
    }
}

fn check_pb(base: u16, base_offset: u16) -> bool {
    (base & 0xFF00) != (base_offset & 0xFF00)
}

impl Cpu {
    pub fn new(mapper: Mapper) -> Cpu {
        let mut cpu = Cpu {
            cycle_count: 0,
            regs: Registers {
                acc: 0,
                x: 0,
                y: 0,
                pc: ProgramCounter::new(0),
                sp: 0xFD,
                flags: 0,
            },
            mem: MemManageUnit::new(mapper),
        };
        cpu.set_flag(0b00100000, true);
        cpu.regs.pc.set_addr(cpu.mem.load_u16(RESET_VEC));
        cpu
    }

    fn incr_cc(&mut self) {
        self.cycle_count += 1;
    }

    fn write_dma(&mut self, high_nyb: u8) {
        // TODO: NES adds 1 cycle if CPU is on an odd CPU cycle, add logic in
        // CPU to track if currently cycle is even or odd
        self.incr_cc();
        let page_num = (high_nyb as u16) << 8;
        for address in page_num..page_num + 0xFF {
            let tmp = self.mem.load_u8(address);
            self.store_u8(OAM_DATA, tmp);
            self.cycle_count += 2;
        }
    }

    fn store_u8(&mut self, addr: u16, val: u8) {
        if addr == DMA_ADDR {
            self.write_dma(val);
        } else {
            self.mem.store_u8(addr, val);
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
                        let tmp = self.mem.load_u8(addr);
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
                            }
                            Op::Store(Store::STA) => {
                                let tmp = self.regs.acc;
                                self.store_u8(addr, tmp);
                            }
                            Op::Store(Store::STX) => {
                                let tmp = self.regs.x;
                                self.store_u8(addr, tmp);
                            }
                            Op::Store(Store::STY) => {
                                let tmp = self.regs.y;
                                self.store_u8(addr, tmp);
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
                            Op::Branch(Branch::BCC) => !self.get_flag(CARRY),
                            Op::Branch(Branch::BCS) => self.get_flag(CARRY),
                            Op::Branch(Branch::BNE) => !self.get_flag(ZERO),
                            Op::Branch(Branch::BEQ) => self.get_flag(ZERO),
                            Op::Branch(Branch::BPL) => !self.get_flag(NEG),
                            Op::Branch(Branch::BMI) => self.get_flag(NEG),
                            Op::Branch(Branch::BVC) => !self.get_flag(O_F),
                            Op::Branch(Branch::BVS) => self.get_flag(O_F),
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
                        self.set_zero_neg(x);
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
                        self.set_flag(BRK_F, true);
                        let flags = self.regs.flags;
                        self.push(flags);
                    }
                    Op::Store(Store::PLA) => {
                        let acc = self.pop();
                        self.regs.acc = acc;
                        self.set_zero_neg(acc);
                    }
                    Op::Store(Store::PLP) => {
                        self.regs.flags = self.pop();
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
                        self.regs.flags = self.pop();
                        self.pull_pc();
                    }
                    Op::Jump(Jump::RTS) => {
                        self.pull_pc();
                        self.regs.pc.add_unsigned(1);
                    }
                    Op::Reg(Reg::CLC) => self.set_flag(CARRY, false),
                    Op::Reg(Reg::CLD) => self.set_flag(DEC, false),
                    Op::Reg(Reg::CLI) => self.set_flag(ITR, false),
                    Op::Reg(Reg::CLV) => self.set_flag(O_F, false),
                    Op::Reg(Reg::SEC) => self.set_flag(CARRY, true),
                    Op::Reg(Reg::SED) => self.set_flag(DEC, true),
                    Op::Reg(Reg::SEI) => self.set_flag(ITR, true),
                    Op::Sys(Sys::BRK) => {
                        self.push_pc();
                        let flags = self.regs.flags;
                        self.push(flags);
                        self.regs.pc.set_addr(BRK_VEC);
                        self.set_flag(BRK_F, true);
                    }

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
        let tmp = acc as i16 - val as i16 - (1 - self.get_flag(CARRY) as i16);
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
        let (tmp, n_flag) = Cpu::get_ror(self.get_flag(CARRY), self.regs.acc);
        self.set_flag(CARRY, n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn ror_addr(&mut self, addr: u16) {
        let (tmp, n_flag) =
            Cpu::get_ror(self.get_flag(CARRY), self.mem.load_u8(addr));
        self.set_flag(CARRY, n_flag);
        self.set_zero_neg(tmp);
        self.store_u8(addr, tmp);
    }

    fn get_ror(carry_flag: bool, val: u8) -> (u8, bool) {
        ((val >> 1) | ((carry_flag as u8) << 7), (val & 0b01) != 0)
    }

    fn rol_acc(&mut self) {
        let (tmp, n_flag) = Cpu::get_rol(self.get_flag(CARRY), self.regs.acc);
        self.set_flag(CARRY, n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn rol_addr(&mut self, addr: u16) {
        let (tmp, n_flag) =
            Cpu::get_rol(self.get_flag(CARRY), self.mem.load_u8(addr));
        self.set_flag(CARRY, n_flag);
        self.set_zero_neg(tmp);
        self.store_u8(addr, tmp);
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
        self.store_u8(addr, val);
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
        self.store_u8(addr, val);
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
        self.set_flag(ZERO, (val & acc) == 0);
        self.set_flag(O_F, (val & 0x40) != 0);
        self.set_flag(NEG, (val & 0x80) != 0);
    }

    fn dec(&mut self, addr: u16) {
        let val: u8 = self.mem.load_u8(addr).wrapping_sub(1);
        self.set_zero_neg(val);
        self.store_u8(addr, val);
    }

    fn inc(&mut self, addr: u16) {
        let val: u8 = self.mem.load_u8(addr).wrapping_add(1);
        self.set_zero_neg(val);
        self.store_u8(addr, val);
    }

    fn push(&mut self, val: u8) {
        let addr = self.regs.sp as u16 | 0x100;
        self.store_u8(addr, val);
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
        let high = self.regs.pc.get_addr() >> 8;
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

    pub fn step(&mut self) -> Result<(), InvalidOp> {
        let byte = self.loadu8_pc_up();
        self.cycle_count += CYCLES[byte as usize];
        let (op, addr_mode) = self.decode_op(byte)?;
        println!("{:?}, {:?}", op, addr_mode);
        println!("{:?}", self.regs);
        let addr_data = addr_mode.address_mem(&self);
        self.execute_op(op, addr_data);
        Ok(())
    }

    fn loadu8_pc_up(&mut self) -> u8 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add_unsigned(1);
        self.mem.load_u8(ram_ptr)
    }

    fn loadu16_pc_up(&mut self) -> u16 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add_unsigned(2);
        self.mem.load_u16(ram_ptr)
    }

    pub fn decode_op(&mut self, op: u8) -> Result<(Op, AddrMode), InvalidOp> {
        match op {
            INC_ABSX => {
                Ok((Op::Math(Math::INC), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            SBC_ABSX => {
                Ok((Op::Math(Math::SBC), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            SBC_ABSY => {
                Ok((Op::Math(Math::SBC), AddrMode::AbsY(self.loadu16_pc_up())))
            }
            SBC_INDY => Ok((
                Op::Math(Math::SBC),
                AddrMode::IndirIndexY(self.loadu8_pc_up()),
            )),
            SBC_INDX => Ok((
                Op::Math(Math::SBC),
                AddrMode::IndexIndirX(self.loadu8_pc_up()),
            )),
            SBC_ZPX => {
                Ok((Op::Math(Math::SBC), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            SBC_ZP => {
                Ok((Op::Math(Math::SBC), AddrMode::ZP(self.loadu8_pc_up())))
            }
            INC_ZPX => {
                Ok((Op::Math(Math::INC), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            INC_ABS => {
                Ok((Op::Math(Math::INC), AddrMode::Abs(self.loadu16_pc_up())))
            }
            INC_ZP => {
                Ok((Op::Math(Math::INC), AddrMode::ZP(self.loadu8_pc_up())))
            }
            CPX_ABS => {
                Ok((Op::Reg(Reg::CPX), AddrMode::Abs(self.loadu16_pc_up())))
            }
            CPX_ZP => {
                Ok((Op::Reg(Reg::CPX), AddrMode::ZP(self.loadu8_pc_up())))
            }
            CPX_IMM => {
                Ok((Op::Reg(Reg::CPX), AddrMode::Imm(self.loadu8_pc_up())))
            }
            SBC_IMM => {
                Ok((Op::Math(Math::SBC), AddrMode::Imm(self.loadu8_pc_up())))
            }
            CMP_IMM => {
                Ok((Op::Reg(Reg::CMP), AddrMode::Imm(self.loadu8_pc_up())))
            }
            CPY_IMM => {
                Ok((Op::Reg(Reg::CPY), AddrMode::Imm(self.loadu8_pc_up())))
            }
            LDA_IMM => {
                Ok((Op::Store(Store::LDA), AddrMode::Imm(self.loadu8_pc_up())))
            }
            LDX_IMM => {
                Ok((Op::Store(Store::LDX), AddrMode::Imm(self.loadu8_pc_up())))
            }
            LDY_IMM => {
                Ok((Op::Store(Store::LDY), AddrMode::Imm(self.loadu8_pc_up())))
            }
            ADC_IMM => {
                Ok((Op::Math(Math::ADC), AddrMode::Imm(self.loadu8_pc_up())))
            }
            EOR_IMM => {
                Ok((Op::Bit(Bit::EOR), AddrMode::Imm(self.loadu8_pc_up())))
            }
            AND_IMM => {
                Ok((Op::Bit(Bit::AND), AddrMode::Imm(self.loadu8_pc_up())))
            }
            ORA_IMM => {
                Ok((Op::Bit(Bit::ORA), AddrMode::Imm(self.loadu8_pc_up())))
            }
            CMP_ABSX => {
                Ok((Op::Reg(Reg::CMP), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            CMP_ABSY => {
                Ok((Op::Reg(Reg::CMP), AddrMode::AbsY(self.loadu16_pc_up())))
            }
            DEC_ZPX => {
                Ok((Op::Math(Math::DEC), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            DEC_ABS => {
                Ok((Op::Math(Math::DEC), AddrMode::Abs(self.loadu16_pc_up())))
            }
            DEC_ZP => {
                Ok((Op::Math(Math::DEC), AddrMode::ZP(self.loadu8_pc_up())))
            }
            CMP_ZPX => {
                Ok((Op::Reg(Reg::CMP), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            CMP_INDY => Ok((
                Op::Reg(Reg::CMP),
                AddrMode::IndirIndexY(self.loadu8_pc_up()),
            )),
            CMP_ABS => {
                Ok((Op::Reg(Reg::CMP), AddrMode::Abs(self.loadu16_pc_up())))
            }
            CMP_ZP => {
                Ok((Op::Reg(Reg::CMP), AddrMode::ZP(self.loadu8_pc_up())))
            }
            CPY_ZP => {
                Ok((Op::Reg(Reg::CPY), AddrMode::ZP(self.loadu8_pc_up())))
            }
            CMP_INDX => Ok((
                Op::Reg(Reg::CMP),
                AddrMode::IndexIndirX(self.loadu8_pc_up()),
            )),
            LDA_ABSX => Ok((
                Op::Store(Store::LDA),
                AddrMode::AbsX(self.loadu16_pc_up()),
            )),
            LDA_ABSY => Ok((
                Op::Store(Store::LDA),
                AddrMode::AbsY(self.loadu16_pc_up()),
            )),
            LDA_ZPX => {
                Ok((Op::Store(Store::LDA), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            LDA_INDY => Ok((
                Op::Store(Store::LDA),
                AddrMode::IndirIndexY(self.loadu8_pc_up()),
            )),
            LDA_ABS => {
                Ok((Op::Store(Store::LDA), AddrMode::Abs(self.loadu16_pc_up())))
            }
            LDA_ZP => {
                Ok((Op::Store(Store::LDA), AddrMode::ZP(self.loadu8_pc_up())))
            }
            LDA_INDX => Ok((
                Op::Store(Store::LDA),
                AddrMode::IndexIndirX(self.loadu8_pc_up()),
            )),
            LDY_ABSX => Ok((
                Op::Store(Store::LDY),
                AddrMode::AbsX(self.loadu16_pc_up()),
            )),
            LDY_ZPX => {
                Ok((Op::Store(Store::LDY), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            LDX_ABS => {
                Ok((Op::Store(Store::LDX), AddrMode::Abs(self.loadu16_pc_up())))
            }
            LDY_ABS => {
                Ok((Op::Store(Store::LDY), AddrMode::Abs(self.loadu16_pc_up())))
            }
            LDX_ZP => {
                Ok((Op::Store(Store::LDX), AddrMode::ZP(self.loadu8_pc_up())))
            }
            LDY_ZP => {
                Ok((Op::Store(Store::LDY), AddrMode::ZP(self.loadu8_pc_up())))
            }
            STA_ABSX => Ok((
                Op::Store(Store::STA),
                AddrMode::AbsX(self.loadu16_pc_up()),
            )),
            STA_ABSY => Ok((
                Op::Store(Store::STA),
                AddrMode::AbsY(self.loadu16_pc_up()),
            )),
            STA_ZPX => {
                Ok((Op::Store(Store::STA), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            STA_INDY => Ok((
                Op::Store(Store::STA),
                AddrMode::IndirIndexY(self.loadu8_pc_up()),
            )),
            STA_ABS => {
                Ok((Op::Store(Store::STA), AddrMode::Abs(self.loadu16_pc_up())))
            }
            STA_ZP => {
                Ok((Op::Store(Store::STA), AddrMode::ZP(self.loadu8_pc_up())))
            }
            STA_INDX => Ok((
                Op::Store(Store::STA),
                AddrMode::IndexIndirX(self.loadu8_pc_up()),
            )),
            STX_ABS => {
                Ok((Op::Store(Store::STX), AddrMode::Abs(self.loadu16_pc_up())))
            }
            STX_ZP => {
                Ok((Op::Store(Store::STX), AddrMode::ZP(self.loadu8_pc_up())))
            }
            STY_ABS => {
                Ok((Op::Store(Store::STY), AddrMode::Abs(self.loadu16_pc_up())))
            }
            STY_ZPX => {
                Ok((Op::Store(Store::STY), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            STY_ZP => {
                Ok((Op::Store(Store::STY), AddrMode::ZP(self.loadu8_pc_up())))
            }
            ROR_ABSX => {
                Ok((Op::Bit(Bit::ROR), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            ROR_ZPX => {
                Ok((Op::Bit(Bit::ROR), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            ADC_ABSX => {
                Ok((Op::Math(Math::ADC), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            ADC_ABSY => {
                Ok((Op::Math(Math::ADC), AddrMode::AbsY(self.loadu16_pc_up())))
            }
            ADC_ZPX => {
                Ok((Op::Math(Math::ADC), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            ADC_INDY => Ok((
                Op::Math(Math::ADC),
                AddrMode::IndirIndexY(self.loadu8_pc_up()),
            )),
            ADC_ABS => {
                Ok((Op::Math(Math::ADC), AddrMode::Abs(self.loadu16_pc_up())))
            }
            ADC_ZP => {
                Ok((Op::Math(Math::ADC), AddrMode::ZP(self.loadu8_pc_up())))
            }
            ADC_INDX => Ok((
                Op::Math(Math::ADC),
                AddrMode::IndexIndirX(self.loadu8_pc_up()),
            )),
            ROR_ABS => {
                Ok((Op::Bit(Bit::ROR), AddrMode::Abs(self.loadu16_pc_up())))
            }
            ROR_ZP => {
                Ok((Op::Bit(Bit::ROR), AddrMode::ZP(self.loadu8_pc_up())))
            }
            LSR_ABSX => {
                Ok((Op::Bit(Bit::LSR), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            EOR_ABSX => {
                Ok((Op::Bit(Bit::EOR), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            EOR_ABSY => {
                Ok((Op::Bit(Bit::EOR), AddrMode::AbsY(self.loadu16_pc_up())))
            }
            EOR_ZPX => {
                Ok((Op::Bit(Bit::EOR), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            EOR_INDY => Ok((
                Op::Bit(Bit::EOR),
                AddrMode::IndirIndexY(self.loadu8_pc_up()),
            )),
            EOR_ABS => {
                Ok((Op::Bit(Bit::EOR), AddrMode::Abs(self.loadu16_pc_up())))
            }
            EOR_ZP => {
                Ok((Op::Bit(Bit::EOR), AddrMode::ZP(self.loadu8_pc_up())))
            }
            EOR_INDX => Ok((
                Op::Bit(Bit::EOR),
                AddrMode::IndexIndirX(self.loadu8_pc_up()),
            )),
            LSR_ZPX => {
                Ok((Op::Bit(Bit::LSR), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            LSR_ABS => {
                Ok((Op::Bit(Bit::LSR), AddrMode::Abs(self.loadu16_pc_up())))
            }
            LSR_ZP => {
                Ok((Op::Bit(Bit::LSR), AddrMode::ZP(self.loadu8_pc_up())))
            }
            JMP_ABS => {
                Ok((Op::Jump(Jump::JMP), AddrMode::Abs(self.loadu16_pc_up())))
            }
            ROL_ABSX => {
                Ok((Op::Bit(Bit::ROL), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            AND_ABSX => {
                Ok((Op::Bit(Bit::AND), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            AND_ABSY => {
                Ok((Op::Bit(Bit::AND), AddrMode::AbsY(self.loadu16_pc_up())))
            }
            ROL_ZPX => {
                Ok((Op::Bit(Bit::ROL), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            AND_INDY => Ok((
                Op::Bit(Bit::AND),
                AddrMode::IndirIndexY(self.loadu8_pc_up()),
            )),
            ROL_ABS => {
                Ok((Op::Bit(Bit::ROL), AddrMode::Abs(self.loadu16_pc_up())))
            }
            AND_ABS => {
                Ok((Op::Bit(Bit::AND), AddrMode::Abs(self.loadu16_pc_up())))
            }
            BIT_ABS => {
                Ok((Op::Bit(Bit::BIT), AddrMode::Abs(self.loadu16_pc_up())))
            }
            BIT_ZP => {
                Ok((Op::Bit(Bit::BIT), AddrMode::ZP(self.loadu8_pc_up())))
            }
            ROL_ZP => {
                Ok((Op::Bit(Bit::ROL), AddrMode::ZP(self.loadu8_pc_up())))
            }
            AND_ZP => {
                Ok((Op::Bit(Bit::AND), AddrMode::ZP(self.loadu8_pc_up())))
            }
            AND_INDX => Ok((
                Op::Bit(Bit::AND),
                AddrMode::IndexIndirX(self.loadu8_pc_up()),
            )),
            ASL_ABSX => {
                Ok((Op::Bit(Bit::ASL), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            ORA_ABSX => {
                Ok((Op::Bit(Bit::ORA), AddrMode::AbsX(self.loadu16_pc_up())))
            }
            ORA_ABSY => {
                Ok((Op::Bit(Bit::ORA), AddrMode::AbsY(self.loadu16_pc_up())))
            }
            ORA_ZPX => {
                Ok((Op::Bit(Bit::ORA), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            ORA_INDY => Ok((
                Op::Bit(Bit::ORA),
                AddrMode::IndirIndexY(self.loadu8_pc_up()),
            )),
            ORA_ABS => {
                Ok((Op::Bit(Bit::ORA), AddrMode::Abs(self.loadu16_pc_up())))
            }
            ORA_ZP => {
                Ok((Op::Bit(Bit::ORA), AddrMode::ZP(self.loadu8_pc_up())))
            }
            ORA_INDX => Ok((
                Op::Bit(Bit::ORA),
                AddrMode::IndexIndirX(self.loadu8_pc_up()),
            )),
            ASL_ZPX => {
                Ok((Op::Bit(Bit::ASL), AddrMode::ZPX(self.loadu8_pc_up())))
            }
            ASL_ABS => {
                Ok((Op::Bit(Bit::ASL), AddrMode::Abs(self.loadu16_pc_up())))
            }
            ASL_ZP => {
                Ok((Op::Bit(Bit::ASL), AddrMode::ZP(self.loadu8_pc_up())))
            }
            LDX_ZPY => {
                Ok((Op::Store(Store::LDX), AddrMode::ZPY(self.loadu8_pc_up())))
            }
            STX_ZPY => {
                Ok((Op::Store(Store::STX), AddrMode::ZPY(self.loadu8_pc_up())))
            }
            AND_ZPX => {
                Ok((Op::Bit(Bit::AND), AddrMode::ZPX(self.loadu8_pc_up())))
            }
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
            NOP => Ok((Op::Sys(Sys::NOP), AddrMode::Impl)),
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
                AddrMode::Rel(self.loadu8_pc_up() as i8),
            )),
            BVC => Ok((
                Op::Branch(Branch::BVC),
                AddrMode::Rel(self.loadu8_pc_up() as i8),
            )),
            BMI => Ok((
                Op::Branch(Branch::BMI),
                AddrMode::Rel(self.loadu8_pc_up() as i8),
            )),
            BPL => Ok((
                Op::Branch(Branch::BPL),
                AddrMode::Rel(self.loadu8_pc_up() as i8),
            )),
            BNE => Ok((
                Op::Branch(Branch::BNE),
                AddrMode::Rel(self.loadu8_pc_up() as i8),
            )),
            BEQ => Ok((
                Op::Branch(Branch::BEQ),
                AddrMode::Rel(self.loadu8_pc_up() as i8),
            )),
            BCS => Ok((
                Op::Branch(Branch::BCS),
                AddrMode::Rel(self.loadu8_pc_up() as i8),
            )),
            BCC => Ok((
                Op::Branch(Branch::BCC),
                AddrMode::Rel(self.loadu8_pc_up() as i8),
            )),
            JSR => {
                Ok((Op::Jump(Jump::JSR), AddrMode::Abs(self.loadu16_pc_up())))
            }
            JMP_IND => Ok((
                Op::Jump(Jump::JMP),
                AddrMode::JmpIndir(self.loadu16_pc_up()),
            )),
            _ => Err(InvalidOp::DoesntExist("Unsupported op".to_string(), op)),
        }
    }
}
