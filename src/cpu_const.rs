pub const INC_ABSX: u8 = 0xFE;
pub const SBC_ABSX: u8 = 0xFD;
pub const SBC_ABSY: u8 = 0xF9;
pub const INC_ZPX: u8 = 0xF6;
pub const SBC_ZPX: u8 = 0xF5;
pub const SBC_INDY: u8 = 0xF1;
pub const INC_ABS: u8 = 0xEE;
pub const SED: u8 = 0xF8;
pub const CPX_ABS: u8 = 0xEC;
pub const NOP: u8 = 0xEA;
pub const SBC_IMM: u8 = 0xE9;
pub const INX: u8 = 0xE8;
pub const INC_ZP: u8 = 0xE6;
pub const SBC_ZP: u8 = 0xE5;
pub const CPX_ZP: u8 = 0xE4;
pub const SBC_INDX: u8 = 0xE1;
pub const CPX_IMM: u8 = 0xE0;
pub const CMP_ABSX: u8 = 0xDD;
pub const CMP_ABSY: u8 = 0xD9;
pub const CLD: u8 = 0xD8;
pub const DEC_ZPX: u8 = 0xD6;
pub const CMP_ZPX: u8 = 0xD5;
pub const CMP_INDY: u8 = 0xD1;
pub const BNE: u8 = 0xD0;
pub const DEC_ABS: u8 = 0xCE;
pub const CMP_ABS: u8 = 0xCD;
pub const DEX: u8 = 0xCA;
pub const CMP_IMM: u8 = 0xC9;
pub const INY: u8 = 0xC8;
pub const DEC_ZP: u8 = 0xC6;
pub const CMP_ZP: u8 = 0xC5;
pub const CPY_ZP: u8 = 0xC4;
pub const CMP_INDX: u8 = 0xC1;
pub const CPY_IMM: u8 = 0xC0;
pub const BEQ: u8 = 0xF0;
pub const LDA_ABSX: u8 = 0xBD;
pub const LDY_ABSX: u8 = 0xBC;
pub const TSX: u8 = 0xBA;
pub const LDA_ABSY: u8 = 0xB9;
pub const CLV: u8 = 0xB8;
pub const LDX_ZPY: u8 = 0xB6;
pub const LDA_ZPX: u8 = 0xB5;
pub const LDY_ZPX: u8 = 0xB4;
pub const LDA_INDY: u8 = 0xB1;
pub const BCS: u8 = 0xB0;
pub const LDX_ABS: u8 = 0xAE;
pub const LDA_ABS: u8 = 0xAD;
pub const LDY_ABS: u8 = 0xAC;
pub const TAX: u8 = 0xAA;
pub const LDA_IMM: u8 = 0xA9;
pub const TAY: u8 = 0xA8;
pub const LDX_ZP: u8 = 0xA6;
pub const LDA_ZP: u8 = 0xA5;
pub const LDY_ZP: u8 = 0xA4;
pub const LDX_IMM: u8 = 0xA2;
pub const LDA_INDX: u8 = 0xA1;
pub const LDY_IMM: u8 = 0xA0;
pub const STA_ABSX: u8 = 0x9D;
pub const TXS: u8 = 0x9A;
pub const STA_ABSY: u8 = 0x99;
pub const TYA: u8 = 0x98;
pub const STX_ZPY: u8 = 0x96;
pub const STA_ZPX: u8 = 0x95;
pub const STY_ZPX: u8 = 0x94;
pub const STA_INDY: u8 = 0x91;
pub const BCC: u8 = 0x90;
pub const STX_ABS: u8 = 0x8E;
pub const STA_ABS: u8 = 0x8D;
pub const STY_ABS: u8 = 0x8C;
pub const TXA: u8 = 0x8A;
pub const DEY: u8 = 0x88;
pub const STX_ZP: u8 = 0x86;
pub const STA_ZP: u8 = 0x85;
pub const STY_ZP: u8 = 0x84;
pub const STA_INDX: u8 = 0x81;
pub const ROR_ABSX: u8 = 0x7E;
pub const ADC_ABSX: u8 = 0x7D;
pub const ADC_ABSY: u8 = 0x79;
pub const SEI: u8 = 0x78;
pub const ROR_ZPX: u8 = 0x76;
pub const ADC_ZPX: u8 = 0x75;
pub const ADC_INDY: u8 = 0x71;
pub const BVS: u8 = 0x70;
pub const ROR_ABS: u8 = 0x6E;
pub const ADC_ABS: u8 = 0x6D;
pub const JMP_IND: u8 = 0x6C;
pub const ROR_ACC: u8 = 0x6A;
pub const ADC_IMM: u8 = 0x69;
pub const PLA: u8 = 0x68;
pub const ROR_ZP: u8 = 0x66;
pub const ADC_ZP: u8 = 0x65;
pub const ADC_INDX: u8 = 0x61;
pub const RTS: u8 = 0x60;
pub const LSR_ABSX: u8 = 0x5E;
pub const EOR_ABSX: u8 = 0x5D;
pub const EOR_ABSY: u8 = 0x59;
pub const CLI: u8 = 0x58;
pub const LSR_ZPX: u8 = 0x56;
pub const EOR_ZPX: u8 = 0x55;
pub const EOR_INDY: u8 = 0x51;
pub const BVC: u8 = 0x50;
pub const LSR_ABS: u8 = 0x4E;
pub const EOR_ABS: u8 = 0x4D;
pub const JMP_ABS: u8 = 0x4C;
pub const LSR_ACC: u8 = 0x4A;
pub const EOR_IMM: u8 = 0x49;
pub const PHA: u8 = 0x48;
pub const LSR_ZP: u8 = 0x46;
pub const EOR_ZP: u8 = 0x45;
pub const EOR_INDX: u8 = 0x41;
pub const RTI: u8 = 0x40;
pub const ROL_ABSX: u8 = 0x3E;
pub const AND_ABSX: u8 = 0x3D;
pub const AND_ABSY: u8 = 0x39;
pub const SEC: u8 = 0x38;
pub const ROL_ZPX: u8 = 0x36;
pub const AND_ZPX: u8 = 0x35;
pub const AND_INDY: u8 = 0x31;
pub const BMI: u8 = 0x30;
pub const ROL_ABS: u8 = 0x2E;
pub const AND_ABS: u8 = 0x2D;
pub const BIT_ABS: u8 = 0x2C;
pub const ROL_ACC: u8 = 0x2A;
pub const AND_IMM: u8 = 0x29;
pub const PLP: u8 = 0x28;
pub const ROL_ZP: u8 = 0x26;
pub const AND_ZP: u8 = 0x25;
pub const BIT_ZP: u8 = 0x24;
pub const AND_INDX: u8 = 0x21;
pub const JSR: u8 = 0x20;
pub const ASL_ABSX: u8 = 0x1E;
pub const ORA_ABSX: u8 = 0x1D;
pub const ORA_ABSY: u8 = 0x19;
pub const CLC: u8 = 0x18;
pub const ASL_ZPX: u8 = 0x16;
pub const ORA_ZPX: u8 = 0x15;
pub const ORA_INDY: u8 = 0x11;
pub const BPL: u8 = 0x10;
pub const ASL_ABS: u8 = 0x0E;
pub const ORA_ABS: u8 = 0x0D;
pub const ASL_ACC: u8 = 0x0A;
pub const ORA_IMM: u8 = 0x09;
pub const PHP: u8 = 0x08;
pub const ASL_ZP: u8 = 0x06;
pub const ORA_ZP: u8 = 0x05;
pub const ORA_INDX: u8 = 0x01;
pub const BRK: u8 = 0x00;



pub static CYCLES: [u8; 256] = [
    7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2,
    4, 2, 7, 4, 4, 7, 7, 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6, 2, 5,
    2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, 6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2,
    2, 3, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, 6, 6, 2, 8,
    3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4,
    4, 7, 7, 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 2, 6, 2, 6, 4, 4,
    4, 4, 2, 5, 2, 5, 5, 5, 5, 5, 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4,
    4, 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4, 2, 6, 2, 8, 3, 3, 5, 5,
    2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, 2,
    6, 3, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4,
    2, 7, 4, 4, 7, 7,
];

pub const CARRY_FLG: u8 = 0b00000001;
pub const ZERO_FLG: u8 = 0b00000010;
pub const ITR_FLG: u8 = 0b00000100;
pub const DEC_FLG: u8 = 0b00001000;
pub const BRK_FLG: u8 = 0b00010000;
pub const O_F_FLG: u8 = 0b01000000;
pub const NEG_FLG: u8 = 0b10000000;

pub const NMI_VEC: u16 = 0xFFFA;
pub const RESET_VEC: u16 = 0xFFFC;
pub const BRK_VEC: u16 = 0xFFFE;
