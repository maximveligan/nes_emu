use mmu::MemManageUnit;
use std::ops::Add;

const INC_ABSX: u8 = 0xFE;
const SBC_ABSX: u8 = 0xFD;
const SBC_ABSY: u8 = 0xF9;
const INC_ZPX: u8 = 0xF6;
const SBC_ZPX: u8 = 0xF5;
const SBC_INDY: u8 = 0xF1;
const INC_ABS: u8 = 0xEE;
const SED: u8 = 0xF8;
const CPX_ABS: u8 = 0xEC;
const NOP: u8 = 0xEA;
const SBC_IMM: u8 = 0xE9;
const INX: u8 = 0xE8;
const INC_ZP: u8 = 0xE6;
const SBC_ZP: u8 = 0xE5;
const CPX_ZP: u8 = 0xE4;
const SBC_INDX: u8 = 0xE1;
const CPX_IMM: u8 = 0xE0;
const CMP_ABSX: u8 = 0xDD;
const CMP_ABSY: u8 = 0xD9;
const CLD: u8 = 0xD8;
const DEC_ZPX: u8 = 0xD6;
const CMP_ZPX: u8 = 0xD5;
const CMP_INDY: u8 = 0xD1;
const BNE: u8 = 0xD0;
const DEC_ABS: u8 = 0xCE;
const CMP_ABS: u8 = 0xCD;
const DEX: u8 = 0xCA;
const CMP_IMM: u8 = 0xC9;
const INY: u8 = 0xC8;
const DEC_ZP: u8 = 0xC6;
const CMP_ZP: u8 = 0xC5;
const CPY_ZP: u8 = 0xC4;
const CMP_INDX: u8 = 0xC1;
const CPY_IMM: u8 = 0xC0;
const BEQ: u8 = 0xF0;
const LDA_ABSX: u8 = 0xBD;
const LDY_ABSX: u8 = 0xBC;
const TSX: u8 = 0xBA;
const LDA_ABSY: u8 = 0xB9;
const CLV: u8 = 0xB8;
const LDX_ZPY: u8 = 0xB6;
const LDA_ZPX: u8 = 0xB5;
const LDY_ZPX: u8 = 0xB4;
const LDA_INDY: u8 = 0xB1;
const BCS: u8 = 0xB0;
const LDX_ABS: u8 = 0xAE;
const LDA_ABS: u8 = 0xAD;
const LDY_ABS: u8 = 0xAC;
const TAX: u8 = 0xAA;
const LDA_IMM: u8 = 0xA9;
const TAY: u8 = 0xA8;
const LDX_ZP: u8 = 0xA6;
const LDA_ZP: u8 = 0xA5;
const LDY_ZP: u8 = 0xA4;
const LDX_IMM: u8 = 0xA2;
const LDA_INDX: u8 = 0xA1;
const LDY_IMM: u8 = 0xA0;
const STA_ABSX: u8 = 0x9D;
const TXS: u8 = 0x9A;
const STA_ABSY: u8 = 0x99;
const TYA: u8 = 0x98;
const STX_ZPY: u8 = 0x96;
const STA_ZPX: u8 = 0x95;
const STY_ZPX: u8 = 0x94;
const STA_INDY: u8 = 0x91;
const BCC: u8 = 0x90;
const STX_ABS: u8 = 0x8E;
const STA_ABS: u8 = 0x8D;
const STY_ABS: u8 = 0x8C;
const TXA: u8 = 0x8A;
const DEY: u8 = 0x88;
const STX_ZP: u8 = 0x86;
const STA_ZP: u8 = 0x85;
const STY_ZP: u8 = 0x84;
const STA_INDX: u8 = 0x81;
const ROR_ABSX: u8 = 0x7E;
const ADC_ABSX: u8 = 0x7D;
const ADC_ABSY: u8 = 0x79;
const SEI: u8 = 0x78;
const ROR_ZPX: u8 = 0x76;
const ADC_ZPX: u8 = 0x75;
const ADC_INDY: u8 = 0x71;
const BVS: u8 = 0x70;
const ROR_ABS: u8 = 0x6E;
const ADC_ABS: u8 = 0x6D;
const JMP_IND: u8 = 0x6C;
const ROR_ACC: u8 = 0x6A;
const ADC_IMM: u8 = 0x69;
const PLA: u8 = 0x68;
const ROR_ZP: u8 = 0x66;
const ADC_ZP: u8 = 0x65;
const ADC_INDX: u8 = 0x61;
const RTS: u8 = 0x60;
const LSR_ABSX: u8 = 0x5E;
const EOR_ABSX: u8 = 0x5D;
const EOR_ABSY: u8 = 0x59;
const CLI: u8 = 0x58;
const LSR_ZPX: u8 = 0x56;
const EOR_ZPX: u8 = 0x55;
const EOR_INDY: u8 = 0x51;
const BVC: u8 = 0x50;
const LSR_ABS: u8 = 0x4E;
const EOR_ABS: u8 = 0x4D;
const JMP_ABS: u8 = 0x4C;
const LSR_ACC: u8 = 0x4A;
const EOR_IMM: u8 = 0x49;
const PHA: u8 = 0x48;
const LSR_ZP: u8 = 0x46;
const EOR_ZP: u8 = 0x45;
const EOR_INDX: u8 = 0x41;
const RTI: u8 = 0x40;
const ROL_ABSX: u8 = 0x3E;
const AND_ABSX: u8 = 0x3D;
const AND_ABSY: u8 = 0x39;
const SEC: u8 = 0x38;
const ROL_ZPX: u8 = 0x36;
const AND_ZP_X: u8 = 0x35;
const AND_INDY: u8 = 0x31;
const BMI: u8 = 0x30;
const ROL_ABS: u8 = 0x2E;
const AND_ABS: u8 = 0x2D;
const BIT_ABS: u8 = 0x2C;
const ROL_ACC: u8 = 0x2A;
const AND_IMM: u8 = 0x29;
const PLP: u8 = 0x28;
const ROL_ZP: u8 = 0x26;
const AND_ZP: u8 = 0x25;
const BIT_ZP: u8 = 0x24;
const AND_INDX: u8 = 0x21;
const JSR: u8 = 0x20;
const ASL_ABSX: u8 = 0x1E;
const ORA_ABSX: u8 = 0x1D;
const ORA_ABSY: u8 = 0x19;
const CLC: u8 = 0x18;
const ASL_ZPX: u8 = 0x16;
const ORA_ZPX: u8 = 0x15;
const ORA_INDY: u8 = 0x11;
const BPL: u8      = 0x10;
const ASL_ABS: u8  = 0x0E;
const ORA_ABS: u8  = 0x0D;
const ASL_ACC: u8  = 0x0A;
const ORA_IMM: u8  = 0x09;
const PHP: u8      = 0x08;
const ASL_ZP: u8   = 0x06;
const ORA_ZP: u8   = 0x05;
const ORA_INDX: u8 = 0x01;
const BRK: u8      = 0x00;

static CYCLES: [u8; 256] = [7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
                            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                            6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
                            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                            6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
                            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                            6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
                            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                            2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
                            2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
                            2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
                            2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
                            2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
                            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                            2, 6, 3, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
                            2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7];

pub struct Registers {
    acc: u8,
    x: u8,
    y: u8,
    pc: ProgramCounter,
    sp: u8,
    flags: u8
}

#[derive(Debug, PartialEq)]
pub struct ProgramCounter(u16);

impl ProgramCounter {
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
    regs: Registers,
    mem: MemManageUnit,
}

pub enum Opcode {
    Storage(Storage),
    Math(Math),
    Bitwise(Bitwise),
    Branch(Branch),
    Jump(Jump),
    RegOps(RegOps),
    Stack(Stack),
    System(System)
}

pub enum AddressingMode {
    Immediate,
    ZeroPage,
    Absolute,
    Implied,
    Accumulator,
    Indexed,
    ZeroPageIndexed,
    Indirect,
    PreIndexedIndirect,
    PostIndexedIndirect,
    Relative
}

impl AddressingMode {
    // Returns the first index of where the data needed to be retrieved lives
    fn address_mem(&self, pc: ProgramCounter) -> (u16, bool) {
        let page_crossed = false;
        let mem_pointer = 0x0000;
        match &self {
            Immediate => (3, true),
            ZeroPage =>  (3, true),
            Absolute =>  (3, true),
            Implied =>  (3, true),
            Accumulator =>  (3, true),
            Indexed =>  (3, true),
            ZeroPageIndexed =>  (3, true),
            Indirect =>  (3, true),
            PreIndexedIndirect =>  (3, true),
            PostIndexedIndirect =>  (3, true),
            Relativ =>  (3, true),
        }
    }
}

pub enum Storage {
    LDA(u16),
    LDX(u16),
    LDY(u16),
    STA(u16),
    STX(u16),
    STY(u16),
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

pub enum Math {
    ADC(u16),
    DEC(u16),
    DEX,
    DEY,
    INC(u16),
    INX,
    INY,
    SBC(u16),
}

pub enum Bitwise {
    AND(u16),
    ASL(u16),
    BIT(u16),
    EOR(u16),
    LSR(u16),
    ORA(u16),
    ROL(u16),
    ROR(u16),
}

pub enum Branch {
    BCC(u16),
    BCS(u16),
    BEQ(u16),
    BMI(u16),
    BNE(u16),
    BPL(u16),
    BVC(u16),
    BVS(u16),
}

pub enum Jump {
    JMP(u16),
    JSR(u16),
    RTI,
    RTS,
}

pub enum RegOps {
    CLC,
    CLD,
    CLI,
    CLV,
    CMP(u16),
    CPX(u16),
    CPY(u16),
    SEC,
    SED,
    SEI,
}

pub enum Stack {
    PHA,
    PHP,
    PLA,
    PLP,
}

pub enum System {
    BRK,
    NOP,
}

pub enum InvalidOpcode {
    DoesntExist(String, u8)
}

impl Cpu {
    fn execute_op(&mut self, op: Opcode) -> Result<(), InvalidOpcode> {
        Ok(())
    }

    fn step(&mut self) -> Result<(), InvalidOpcode> {
        let byte = self.loadu8_pc_incr();
        let op = self.decode_op(byte)?;
        self.execute_op(op)?;
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

    fn decode_op(&self, op: u8) -> Result<Opcode, InvalidOpcode> {
        match op {
            INC_ABSX => {self.mem.load_u8(self.regs.pc.get_addr()); panic!()}
            SBC_ABSX => unimplemented!(),
            SBC_ABSY => unimplemented!(),
            INC_ZPX => unimplemented!(),
            SBC_ZPX => unimplemented!(),
            SBC_INDY => unimplemented!(),
            INC_ABS => unimplemented!(),
            SED => unimplemented!(),
            CPX_ABS => unimplemented!(),
            NOP => unimplemented!(),
            SBC_IMM => unimplemented!(),
            INX => unimplemented!(),
            INC_ZP => unimplemented!(),
            SBC_ZP => unimplemented!(),
            CPX_ZP => unimplemented!(),
            SBC_INDX => unimplemented!(),
            CPX_IMM  => unimplemented!(),
            CMP_ABSX => unimplemented!(),
            CMP_ABSY => unimplemented!(),
            CLD => unimplemented!(),
            DEC_ZPX => unimplemented!(),
            CMP_ZPX => unimplemented!(),
            CMP_INDY => unimplemented!(),
            BNE => unimplemented!(),
            DEC_ABS => unimplemented!(),
            CMP_ABS => unimplemented!(),
            BCC => unimplemented!(),
            DEX => unimplemented!(),
            CMP_IMM => unimplemented!(),
            INY => unimplemented!(),
            DEC_ZP => unimplemented!(),
            CMP_ZP => unimplemented!(),
            CPY_ZP => unimplemented!(),
            CMP_INDX => unimplemented!(),
            CPY_IMM => unimplemented!(),
            BEQ => unimplemented!(),
            LDA_ABSX => unimplemented!(),
            LDY_ABSX => unimplemented!(),
            TSX => unimplemented!(),
            LDA_ABSY => unimplemented!(),
            CLV => unimplemented!(),
            LDX_ZPY => unimplemented!(),
            LDA_ZPX => unimplemented!(),
            LDY_ZPX => unimplemented!(),
            LDA_INDY => unimplemented!(),
            BCS => unimplemented!(),
            LDX_ABS => unimplemented!(),
            LDA_ABS => unimplemented!(),
            LDY_ABS => unimplemented!(),
            TAX => unimplemented!(),
            LDA_IMM => unimplemented!(),
            TAY => unimplemented!(),
            LDX_ZP => unimplemented!(),
            LDA_ZP => unimplemented!(),
            LDY_ZP => unimplemented!(),
            LDX_IMM => unimplemented!(),
            LDA_INDX => unimplemented!(),
            LDY_IMM => unimplemented!(),
            STA_ABSX => unimplemented!(),
            TXS => unimplemented!(),
            STA_ABSY => unimplemented!(),
            TYA => unimplemented!(),
            STX_ZPY => unimplemented!(),
            STA_ZPX => unimplemented!(),
            STY_ZPX => unimplemented!(),
            STA_INDY => unimplemented!(),
            BCC => unimplemented!(),
            STX_ABS => unimplemented!(),
            STA_ABS => unimplemented!(),
            STY_ABS => unimplemented!(),
            TXA => unimplemented!(),
            DEY => unimplemented!(),
            STX_ZP => unimplemented!(),
            STA_ZP => unimplemented!(),
            STY_ZP => unimplemented!(),
            STA_INDX => unimplemented!(),
            ROR_ABSX => unimplemented!(),
            ADC_ABSX => unimplemented!(),
            ADC_ABSY => unimplemented!(),
            SEI => unimplemented!(),
            ROR_ZPX => unimplemented!(),
            ADC_ZPX => unimplemented!(),
            ADC_INDY => unimplemented!(),
            BVS => unimplemented!(),
            ROR_ABS => unimplemented!(),
            ADC_ABS => unimplemented!(),
            JMP_IND => unimplemented!(),
            ROR_ACC => unimplemented!(),
            ADC_IMM => unimplemented!(),
            PLA => unimplemented!(),
            ROR_ZP => unimplemented!(),
            ADC_ZP => unimplemented!(),
            ADC_INDX => unimplemented!(),
            RTS => unimplemented!(),
            LSR_ABSX => unimplemented!(),
            EOR_ABSX => unimplemented!(),
            EOR_ABSY => unimplemented!(),
            CLI => unimplemented!(),
            LSR_ZPX => unimplemented!(),
            EOR_ZPX => unimplemented!(),
            EOR_INDY => unimplemented!(),
            BVC => unimplemented!(),
            LSR_ABS => unimplemented!(),
            EOR_ABS => unimplemented!(),
            JMP_ABS => unimplemented!(),
            LSR_ACC => unimplemented!(),
            EOR_IMM => unimplemented!(),
            PHA => unimplemented!(),
            LSR_ZP => unimplemented!(),
            EOR_ZP => unimplemented!(),
            EOR_INDX => unimplemented!(),
            RTI => unimplemented!(),
            ROL_ABSX => unimplemented!(),
            AND_ABSX => unimplemented!(),
            AND_ABSY => unimplemented!(),
            SEC => unimplemented!(),
            ROL_ZPX => unimplemented!(),
            AND_ZP_X => unimplemented!(),
            AND_INDY => unimplemented!(),
            BMI => unimplemented!(),
            ROL_ABS => unimplemented!(),
            AND_ABS => unimplemented!(),
            BIT_ABS => unimplemented!(),
            ROL_ACC => unimplemented!(),
            AND_IMM => unimplemented!(),
            PLP => unimplemented!(),
            ROL_ZP => unimplemented!(),
            AND_ZP => unimplemented!(),
            BIT_ZP => unimplemented!(),
            AND_INDX => unimplemented!(),
            JSR => unimplemented!(),
            ASL_ABSX => unimplemented!(),
            ORA_ABSX => unimplemented!(),
            ORA_ABSY => unimplemented!(),
            CLC => unimplemented!(),
            ASL_ZPX => unimplemented!(),
            ORA_ZPX => unimplemented!(),
            ORA_INDY => unimplemented!(),
            BPL => unimplemented!(),
            ASL_ABS => unimplemented!(),
            ORA_ABS => unimplemented!(),
            ASL_ACC => unimplemented!(),
            ORA_IMM  => unimplemented!(),
            PHP => unimplemented!(),
            ASL_ZP => unimplemented!(),
            ORA_ZP => unimplemented!(),
            ORA_INDX => unimplemented!(),
            BRK => unimplemented!(),
            _ => Err(InvalidOpcode::DoesntExist("Unsupported op".to_string(), op)),
        }
    }

}
