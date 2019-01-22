use mmu::MemManageUnit;
use std::ops::Add;
use cpu_const::*;

pub struct Registers {
    pub acc: u8,
    pub x: u8,
    pub y: u8,
    pub pc: ProgramCounter,
    pub sp: u8,
    pub flags: u8
}

#[derive(Debug, PartialEq)]
pub struct ProgramCounter(u16);

impl ProgramCounter {
    pub fn new(val: u16) -> ProgramCounter {
        ProgramCounter { 0: val }
    }

    fn add(&mut self, offset: u16) {
        self.0 += offset;
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
    pub cycle_count: usize,
}

pub enum Opcode {
    Storage(Storage),
    Math(Math),
    Bitwise(Bitwise),
    Branch(Branch),
    Jump(Jump),
    RegOps(RegOps),
    System(System)
}

pub enum Storage {
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

pub enum Bitwise {
    AND,
    ASL,
    BIT,
    EOR,
    LSR,
    ORA,
    ROL,
    ROR,
}

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

pub enum Jump {
    JMP,
    JSR,
    RTI,
    RTS,
}

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


pub enum System {
    BRK,
    NOP,
}

pub enum InvalidOpcode {
    DoesntExist(String, u8)
}

pub enum AddressingMode {
    Immediate(u8),
    Implied,
    Accumulator,
    ZeroPage(u8),
    ZeroPageX(u8),
    ZeroPageY(u8),
    Abs(u16),
    AbsX(u16),
    AbsY(u16),
    Indirect(u16),
    IndexedIndirectX(u8),
    IndirectIndexedY(u8),
    Relative(i8)
}

enum AddrDataType {
    Address(u16),
    Constant(u8),
}

impl AddressingMode {
    // Returns the first index of where the data needed to be retrieved lives
    fn address_mem(&self, cpu: &Cpu) -> (Option<AddrDataType>, bool) {
        match *self {
            AddressingMode::Immediate(v) => (Some(AddrDataType::Constant(v)), false),
            AddressingMode::Implied => (None, false),
            AddressingMode::Accumulator => (None, false),
            AddressingMode::ZeroPage(v) => (Some(AddrDataType::Address(cpu.mem.load_u8(v as u16) as u16)), false),
            AddressingMode::ZeroPageX(v) => (Some(AddrDataType::Address((cpu.mem.load_u8(v as u16) + cpu.regs.x) as u16)), false),
            AddressingMode::ZeroPageY(v) => (Some(AddrDataType::Address((cpu.mem.load_u8(v as u16) + cpu.regs.y) as u16)), false),
            AddressingMode::Abs(v) => (Some(AddrDataType::Address(cpu.mem.load_u16(v))), false),
            AddressingMode::AbsX(v) => (Some(AddrDataType::Address(cpu.mem.load_u16(v) + (cpu.regs.x as u16))), true), //TODO: Implement logic for bound check
            AddressingMode::AbsY(v) => (Some(AddrDataType::Address(cpu.mem.load_u16(v) + (cpu.regs.y as u16))), true), //TODO: Implement logic for bound check
            AddressingMode::Indirect(v) => unimplemented!("No page boundary cross"),
            AddressingMode::IndexedIndirectX(v) => unimplemented!("No page boundary cross"),
            AddressingMode::IndirectIndexedY(v) => unimplemented!("Possible cross over"),
            AddressingMode::Relative(v) => unimplemented!("No page boundary cross")
        }
    }
}

impl Cpu {
    fn execute_op(&mut self, op: Opcode, addr_mode: Option<AddrDataType>) -> Result<(), InvalidOpcode> {
        match addr_mode {
            Some(mode) => match mode {
                AddrDataType::Address(addr) => unimplemented!(),
                AddrDataType::Constant(c) => unimplemented!(),
            }

            None => match op {
                Opcode::Storage(Storage::TAX) => unimplemented!(),
                Opcode::Storage(Storage::TAY) => unimplemented!(),
                Opcode::Storage(Storage::TSX) => unimplemented!(),
                Opcode::Storage(Storage::TXA) => unimplemented!(),
                Opcode::Storage(Storage::TXS) => unimplemented!(),
                Opcode::Storage(Storage::TYA) => unimplemented!(),
                Opcode::Storage(Storage::PHA) => unimplemented!(),
                Opcode::Storage(Storage::PHP) => unimplemented!(),
                Opcode::Storage(Storage::PLA) => unimplemented!(),
                Opcode::Storage(Storage::PLP) => unimplemented!(),
                Opcode::Math(Math::DEX) => unimplemented!(),
                Opcode::Math(Math::DEY) => unimplemented!(),
                Opcode::Math(Math::INX) => unimplemented!(),
                Opcode::Math(Math::INY) => unimplemented!(),
                Opcode::Jump(Jump::RTI) => unimplemented!(),
                Opcode::Jump(Jump::RTS) => unimplemented!(),
                Opcode::RegOps(RegOps::CLC) => unimplemented!(),
                Opcode::RegOps(RegOps::CLD) => unimplemented!(),
                Opcode::RegOps(RegOps::CLI) => unimplemented!(),
                Opcode::RegOps(RegOps::CLV) => unimplemented!(),
                Opcode::RegOps(RegOps::SEC) => unimplemented!(),
                Opcode::RegOps(RegOps::SED) => unimplemented!(),
                Opcode::RegOps(RegOps::SEI) => unimplemented!(),
                Opcode::System(System::BRK) => unimplemented!(),
                Opcode::System(System::NOP) => unimplemented!(),
                _ => panic!("Programmer error: All opcodes with implied addressing mode have been taken care of"),
            }
        }
    }

    fn execute_const_op(&mut self, op: Opcode, val: u8) -> Result<(), InvalidOpcode> {
        unimplemented!();
    }

    fn execute_addr_op(&mut self, op: Opcode, val: u16) -> Result<(), InvalidOpcode> {
        unimplemented!();
    }

    pub fn step(&mut self) -> Result<(), InvalidOpcode> {
        let byte = self.loadu8_pc_incr();
        let (op, addr_mode) = self.decode_op(byte)?;
        let (address, page_bounary_crossed) = addr_mode.address_mem(&self);
        self.execute_op(op, address)?;
        Ok(())
    }

    fn loadu8_pc_incr(&mut self) -> u8 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add(1);
        self.mem.load_u8(ram_ptr)
    }

    fn loadu16_pc_incr(&mut self) -> u16 {
        let ram_ptr = self.regs.pc.get_addr();
        self.regs.pc.add(2);
        self.mem.load_u16(ram_ptr)
    }

    pub fn decode_op(&mut self, op: u8) -> Result<(Opcode, AddressingMode), InvalidOpcode> {
        match op {
            INC_ABSX => Ok((Opcode::Math(Math::INC), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            SBC_ABSX => Ok((Opcode::Math(Math::SBC), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            SBC_ABSY => Ok((Opcode::Math(Math::SBC), AddressingMode::AbsY(self.loadu16_pc_incr()))),
            SBC_INDY => Ok((Opcode::Math(Math::SBC), AddressingMode::IndirectIndexedY(self.loadu8_pc_incr()))),
            SBC_INDX => Ok((Opcode::Math(Math::SBC), AddressingMode::IndexedIndirectX(self.loadu8_pc_incr()))),
            SBC_IMM => Ok((Opcode::Math(Math::SBC), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            SBC_ZPX => Ok((Opcode::Math(Math::SBC), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            SBC_ZP => Ok((Opcode::Math(Math::SBC), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            INC_ZPX => Ok((Opcode::Math(Math::INC), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            INC_ABS => Ok((Opcode::Math(Math::INC), AddressingMode::Abs(self.loadu16_pc_incr()))),
            INC_ZP => Ok((Opcode::Math(Math::INC), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            CPX_ABS => Ok((Opcode::RegOps(RegOps::CPX), AddressingMode::Abs(self.loadu16_pc_incr()))),
            CPX_ZP => Ok((Opcode::RegOps(RegOps::CPX), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            CPX_IMM  => Ok((Opcode::RegOps(RegOps::CPX), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            CMP_ABSX => Ok((Opcode::RegOps(RegOps::CMP), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            CMP_ABSY => Ok((Opcode::RegOps(RegOps::CMP), AddressingMode::AbsY(self.loadu16_pc_incr()))),
            DEC_ZPX => Ok((Opcode::Math(Math::DEC), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            DEC_ABS => Ok((Opcode::Math(Math::DEC), AddressingMode::Abs(self.loadu16_pc_incr()))),
            DEC_ZP => Ok((Opcode::Math(Math::DEC), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            CMP_ZPX => Ok((Opcode::RegOps(RegOps::CMP), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            CMP_INDY => Ok((Opcode::RegOps(RegOps::CMP), AddressingMode::IndirectIndexedY(self.loadu8_pc_incr()))),
            CMP_ABS => Ok((Opcode::RegOps(RegOps::CMP), AddressingMode::Abs(self.loadu16_pc_incr()))),
            CMP_IMM => Ok((Opcode::RegOps(RegOps::CMP), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            CMP_ZP => Ok((Opcode::RegOps(RegOps::CMP), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            CPY_ZP => Ok((Opcode::RegOps(RegOps::CPY), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            CPY_IMM => Ok((Opcode::RegOps(RegOps::CPY), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            CMP_INDX => Ok((Opcode::RegOps(RegOps::CMP), AddressingMode::IndexedIndirectX(self.loadu8_pc_incr()))),
            LDA_ABSX => Ok((Opcode::Storage(Storage::LDA), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            LDA_ABSY => Ok((Opcode::Storage(Storage::LDA), AddressingMode::AbsY(self.loadu16_pc_incr()))),
            LDA_ZPX => Ok((Opcode::Storage(Storage::LDA), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            LDA_INDY => Ok((Opcode::Storage(Storage::LDA), AddressingMode::IndirectIndexedY(self.loadu8_pc_incr()))),
            LDA_ABS => Ok((Opcode::Storage(Storage::LDA), AddressingMode::Abs(self.loadu16_pc_incr()))),
            LDA_IMM => Ok((Opcode::Storage(Storage::LDA), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            LDA_ZP => Ok((Opcode::Storage(Storage::LDA), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            LDA_INDX => Ok((Opcode::Storage(Storage::LDA), AddressingMode::IndexedIndirectX(self.loadu8_pc_incr()))),
            LDY_ABSX => Ok((Opcode::Storage(Storage::LDY), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            LDY_ZPX => Ok((Opcode::Storage(Storage::LDY), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            LDX_ABS => Ok((Opcode::Storage(Storage::LDX), AddressingMode::Abs(self.loadu16_pc_incr()))),
            LDY_ABS => Ok((Opcode::Storage(Storage::LDY), AddressingMode::Abs(self.loadu16_pc_incr()))),
            LDX_ZP => Ok((Opcode::Storage(Storage::LDX), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            LDY_ZP => Ok((Opcode::Storage(Storage::LDY), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            LDX_IMM => Ok((Opcode::Storage(Storage::LDX), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            LDY_IMM => Ok((Opcode::Storage(Storage::LDY), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            STA_ABSX => Ok((Opcode::Storage(Storage::STA), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            STA_ABSY => Ok((Opcode::Storage(Storage::STA), AddressingMode::AbsY(self.loadu16_pc_incr()))),
            STA_ZPX => Ok((Opcode::Storage(Storage::STA), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            STA_INDY => Ok((Opcode::Storage(Storage::STA), AddressingMode::IndirectIndexedY(self.loadu8_pc_incr()))),
            STA_ABS => Ok((Opcode::Storage(Storage::STA), AddressingMode::Abs(self.loadu16_pc_incr()))),
            STA_ZP => Ok((Opcode::Storage(Storage::STA), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            STA_INDX => Ok((Opcode::Storage(Storage::STA), AddressingMode::IndexedIndirectX(self.loadu8_pc_incr()))),
            STX_ABS => Ok((Opcode::Storage(Storage::STX), AddressingMode::Abs(self.loadu16_pc_incr()))),
            STX_ZP => Ok((Opcode::Storage(Storage::STX), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            STY_ABS => Ok((Opcode::Storage(Storage::STY), AddressingMode::Abs(self.loadu16_pc_incr()))),
            STY_ZPX => Ok((Opcode::Storage(Storage::STY), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            STY_ZP => Ok((Opcode::Storage(Storage::STY), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            ROR_ABSX => Ok((Opcode::Bitwise(Bitwise::ROR), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            ROR_ZPX => Ok((Opcode::Bitwise(Bitwise::ROR), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            ADC_ABSX => Ok((Opcode::Math(Math::ADC), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            ADC_ABSY => Ok((Opcode::Math(Math::ADC), AddressingMode::AbsY(self.loadu16_pc_incr()))),
            ADC_ZPX => Ok((Opcode::Math(Math::ADC), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            ADC_INDY => Ok((Opcode::Math(Math::ADC), AddressingMode::IndirectIndexedY(self.loadu8_pc_incr()))),
            ADC_ABS => Ok((Opcode::Math(Math::ADC), AddressingMode::Abs(self.loadu16_pc_incr()))),
            ADC_IMM => Ok((Opcode::Math(Math::ADC), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            ADC_ZP => Ok((Opcode::Math(Math::ADC), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            ADC_INDX => Ok((Opcode::Math(Math::ADC), AddressingMode::IndexedIndirectX(self.loadu8_pc_incr()))),
            ROR_ABS => Ok((Opcode::Bitwise(Bitwise::ROR), AddressingMode::Abs(self.loadu16_pc_incr()))),
            ROR_ZP => Ok((Opcode::Bitwise(Bitwise::ROR), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            LSR_ABSX => Ok((Opcode::Bitwise(Bitwise::LSR), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            EOR_ABSX => Ok((Opcode::Bitwise(Bitwise::EOR), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            EOR_ABSY => Ok((Opcode::Bitwise(Bitwise::EOR), AddressingMode::AbsY(self.loadu16_pc_incr()))),
            EOR_ZPX => Ok((Opcode::Bitwise(Bitwise::EOR), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            EOR_INDY => Ok((Opcode::Bitwise(Bitwise::EOR), AddressingMode::IndirectIndexedY(self.loadu8_pc_incr()))),
            EOR_ABS => Ok((Opcode::Bitwise(Bitwise::EOR), AddressingMode::Abs(self.loadu16_pc_incr()))),
            EOR_IMM => Ok((Opcode::Bitwise(Bitwise::EOR), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            EOR_ZP => Ok((Opcode::Bitwise(Bitwise::EOR), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            EOR_INDX => Ok((Opcode::Bitwise(Bitwise::EOR), AddressingMode::IndexedIndirectX(self.loadu8_pc_incr()))),
            LSR_ZPX => Ok((Opcode::Bitwise(Bitwise::LSR), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            LSR_ABS => Ok((Opcode::Bitwise(Bitwise::LSR), AddressingMode::Abs(self.loadu16_pc_incr()))),
            LSR_ZP => Ok((Opcode::Bitwise(Bitwise::LSR), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            JMP_ABS => Ok((Opcode::Jump(Jump::JMP), AddressingMode::Abs(self.loadu16_pc_incr()))),
            ROL_ABSX => Ok((Opcode::Bitwise(Bitwise::ROL), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            AND_ABSX => Ok((Opcode::Bitwise(Bitwise::AND), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            AND_ABSY => Ok((Opcode::Bitwise(Bitwise::AND), AddressingMode::AbsY(self.loadu16_pc_incr()))),
            ROL_ZPX => Ok((Opcode::Bitwise(Bitwise::ROL), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            AND_INDY => Ok((Opcode::Bitwise(Bitwise::AND), AddressingMode::IndirectIndexedY(self.loadu8_pc_incr()))),
            ROL_ABS => Ok((Opcode::Bitwise(Bitwise::ROL), AddressingMode::Abs(self.loadu16_pc_incr()))),
            AND_ABS => Ok((Opcode::Bitwise(Bitwise::AND), AddressingMode::Abs(self.loadu16_pc_incr()))),
            BIT_ABS => Ok((Opcode::Bitwise(Bitwise::BIT), AddressingMode::Abs(self.loadu16_pc_incr()))),
            BIT_ZP => Ok((Opcode::Bitwise(Bitwise::BIT), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            AND_IMM => Ok((Opcode::Bitwise(Bitwise::AND), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            ROL_ZP => Ok((Opcode::Bitwise(Bitwise::ROL), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            AND_ZP => Ok((Opcode::Bitwise(Bitwise::AND), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            AND_INDX => Ok((Opcode::Bitwise(Bitwise::AND), AddressingMode::IndexedIndirectX(self.loadu8_pc_incr()))),
            ASL_ABSX => Ok((Opcode::Bitwise(Bitwise::ASL), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            ORA_ABSX => Ok((Opcode::Bitwise(Bitwise::ORA), AddressingMode::AbsX(self.loadu16_pc_incr()))),
            ORA_ABSY => Ok((Opcode::Bitwise(Bitwise::ORA), AddressingMode::AbsY(self.loadu16_pc_incr()))),
            ORA_ZPX => Ok((Opcode::Bitwise(Bitwise::ORA), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            ORA_INDY => Ok((Opcode::Bitwise(Bitwise::ORA), AddressingMode::IndirectIndexedY(self.loadu8_pc_incr()))),
            ORA_ABS => Ok((Opcode::Bitwise(Bitwise::ORA), AddressingMode::Abs(self.loadu16_pc_incr()))),
            ORA_IMM  => Ok((Opcode::Bitwise(Bitwise::ORA), AddressingMode::Immediate(self.loadu8_pc_incr()))),
            ORA_ZP => Ok((Opcode::Bitwise(Bitwise::ORA), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            ORA_INDX => Ok((Opcode::Bitwise(Bitwise::ORA), AddressingMode::IndexedIndirectX(self.loadu8_pc_incr()))),
            ASL_ZPX => Ok((Opcode::Bitwise(Bitwise::ASL), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            ASL_ABS => Ok((Opcode::Bitwise(Bitwise::ASL), AddressingMode::Abs(self.loadu16_pc_incr()))),
            ASL_ZP => Ok((Opcode::Bitwise(Bitwise::ASL), AddressingMode::ZeroPage(self.loadu8_pc_incr()))),
            LDX_ZPY => Ok((Opcode::Storage(Storage::LDX), AddressingMode::ZeroPageY(self.loadu8_pc_incr()))),
            STX_ZPY => Ok((Opcode::Storage(Storage::STX), AddressingMode::ZeroPageY(self.loadu8_pc_incr()))),
            AND_ZPX => Ok((Opcode::Bitwise(Bitwise::AND), AddressingMode::ZeroPageX(self.loadu8_pc_incr()))),
            ROR_ACC => Ok((Opcode::Bitwise(Bitwise::ROR), AddressingMode::Accumulator)),
            ASL_ACC => Ok((Opcode::Bitwise(Bitwise::ASL), AddressingMode::Accumulator)),
            ROL_ACC => Ok((Opcode::Bitwise(Bitwise::ROL), AddressingMode::Accumulator)),
            LSR_ACC => Ok((Opcode::Bitwise(Bitwise::LSR), AddressingMode::Accumulator)),
            SED => Ok((Opcode::RegOps(RegOps::SED), AddressingMode::Implied)),
            CLC => Ok((Opcode::RegOps(RegOps::CLC), AddressingMode::Implied)),
            SEC => Ok((Opcode::RegOps(RegOps::SEC), AddressingMode::Implied)),
            CLI => Ok((Opcode::RegOps(RegOps::CLI), AddressingMode::Implied)),
            SEI => Ok((Opcode::RegOps(RegOps::SEI), AddressingMode::Implied)),
            CLV => Ok((Opcode::RegOps(RegOps::CLV), AddressingMode::Implied)),
            CLD => Ok((Opcode::RegOps(RegOps::CLD), AddressingMode::Implied)),
            NOP => Ok((Opcode::System(System::NOP), AddressingMode::Implied)),
            BRK => Ok((Opcode::System(System::BRK), AddressingMode::Implied)),
            TAX => Ok((Opcode::Storage(Storage::TAX), AddressingMode::Implied)),
            TXA => Ok((Opcode::Storage(Storage::TXA), AddressingMode::Implied)),
            TAY => Ok((Opcode::Storage(Storage::TAY), AddressingMode::Implied)),
            TYA => Ok((Opcode::Storage(Storage::TYA), AddressingMode::Implied)),
            DEX => Ok((Opcode::Math(Math::DEX), AddressingMode::Implied)),
            INX => Ok((Opcode::Math(Math::INX), AddressingMode::Implied)),
            DEY => Ok((Opcode::Math(Math::DEY), AddressingMode::Implied)),
            INY => Ok((Opcode::Math(Math::INY), AddressingMode::Implied)),
            TSX => Ok((Opcode::Storage(Storage::TSX), AddressingMode::Implied)),
            TXS => Ok((Opcode::Storage(Storage::TXS), AddressingMode::Implied)),
            PHA => Ok((Opcode::Storage(Storage::PHA), AddressingMode::Implied)),
            PLA => Ok((Opcode::Storage(Storage::PLA), AddressingMode::Implied)),
            PHP => Ok((Opcode::Storage(Storage::PHP), AddressingMode::Implied)),
            PLP => Ok((Opcode::Storage(Storage::PLP), AddressingMode::Implied)),

            BVS => unimplemented!(),
            BVC => unimplemented!(),
            BMI => unimplemented!(),
            BPL => unimplemented!(),
            BNE => unimplemented!(),
            BCC => unimplemented!(),
            BEQ => unimplemented!(),
            BCS => unimplemented!(),
            BCC => unimplemented!(),
            RTS => unimplemented!(),
            RTI => unimplemented!(),
            JSR => unimplemented!(),

            JMP_IND => unimplemented!(),
            _ => Err(InvalidOpcode::DoesntExist("Unsupported op".to_string(), op)),
        }
    }
}

