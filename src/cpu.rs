use cpu_const::*;
use std::fmt;
use mmu::Mmu;

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

enum Flag {
    Carry = 0b0000_0001,
    Zero = 0b0000_0010,
    Itr = 0b0000_0100,
    Dec = 0b0000_1000,
    Brk = 0b0001_0000,
    Unused = 0b0010_0000,
    Overflow = 0b0100_0000,
    Neg = 0b1000_0000,
}

pub struct Cpu {
    pub regs: Registers,
    pub cycle_count: u16,
    pub mmu: Mmu,
}

pub enum Mode {
    Imm,
    ZP,
    ZPX,
    ZPY,
    Abs,
    AbsX,
    AbsY,
    JmpIndir,
    IndX,
    IndY,
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
            Mode::JmpIndir => {
                let tmp = self.ld16_pc_up();
                let low = self.mmu.ld8(tmp);
                let high: u8 = if tmp & 0xFF == 0xFF {
                    self.mmu.ld8(tmp - 0xFF)
                } else {
                    self.mmu.ld8(tmp + 1)
                };
                ((high as u16) << 8 | (low as u16))
            }
            Mode::IndX => {
                let tmp = self.ld8_pc_up();
                let base_address = tmp.wrapping_add(self.regs.x) as u16;
                if base_address == 0xFF {
                    (self.mmu.ld8(0) as u16) << 8
                        | (self.mmu.ld8(base_address) as u16)
                } else {
                    self.mmu.ld16(base_address)
                }
            }
            Mode::IndY => {
                let base = self.ld8_pc_up();
                let tmp = if base == 0xFF {
                    (self.mmu.ld8(0) as u16) << 8 | (self.mmu.ld8(0xFF) as u16)
                } else {
                    self.mmu.ld16(base as u16)
                };
                let addr = tmp.wrapping_add(self.regs.y as u16);
                self.check_pb(tmp, addr);
                addr
            }
        }
    }

    pub fn proc_nmi(&mut self) {
        let flags = self.regs.flags;
        self.push_pc();
        self.push(flags);
        self.regs.pc.set_addr(self.mmu.ld16(NMI_VEC));
    }

    fn read_op(&mut self, mode: Mode) -> u8 {
        let addr = self.address_mem(mode);
        self.mmu.ld8(addr)
    }

    fn write_dma(&mut self, high_nyb: u8) {
        // TODO: NES adds 1 cycle if CPU is on an odd CPU cycle, add logic in
        // CPU to track if current cycle is even or odd

        self.incr_cc();
        let page_num = (high_nyb as u16) << 8;
        for address in page_num..=page_num + 0xFF {
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
        let tmp = acc as u16 + val as u16 + self.get_flag(Flag::Carry) as u16;
        self.set_flag(Flag::Carry, tmp > 0xFF);
        self.set_flag(
            Flag::Overflow,
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
            Cpu::get_ror(self.get_flag(Flag::Carry), self.regs.acc);
        self.set_flag(Flag::Carry, n_flag);
        self.set_zero_neg(tmp);
        self.regs.acc = tmp;
    }

    fn ror_addr(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
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

    fn rol_addr(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
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

    fn asl_addr(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
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

    fn lsr_addr(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val = self.mmu.ld8(addr);
        self.set_flag(Flag::Carry, (val & 0b01) != 0);
        let tmp = val >> 1;
        self.set_zero_neg(tmp);
        self.store(addr, tmp);
    }

    fn cpx(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let tmp = self.regs.x as i16 - val as i16;
        self.set_flag(Flag::Carry, tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn cpy(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let tmp = self.regs.y as i16 - val as i16;
        self.set_flag(Flag::Carry, tmp >= 0);
        self.set_zero_neg(tmp as u8);
    }

    fn cmp(&mut self, mode: Mode) {
        let val = self.read_op(mode);
        let tmp = self.regs.acc as i16 - val as i16;
        self.set_flag(Flag::Carry, tmp >= 0);
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
        self.set_flag(Flag::Zero, (val & acc) == 0);
        self.set_flag(Flag::Overflow, (val & 0x40) != 0);
        self.set_flag(Flag::Neg, (val & 0x80) != 0);
    }

    fn dec(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val: u8 = self.mmu.ld8(addr).wrapping_sub(1);
        self.set_zero_neg(val);
        self.store(addr, val);
    }

    fn inc(&mut self, mode: Mode) {
        let addr = self.address_mem(mode);
        let val: u8 = self.mmu.ld8(addr).wrapping_add(1);
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
        self.cycle_count += CYCLES[byte as usize] as u16;
        self.execute_op(byte);
        if debug {
            println!("{:?}", regs);
        }
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

    pub fn execute_op(&mut self, op: u8) {
        match op {
            INC_ABSX => self.inc(Mode::AbsX),
            INC_ZPX => self.inc(Mode::ZPX),
            INC_ABS => self.inc(Mode::Abs),
            INC_ZP => self.inc(Mode::ZP),
            SBC_IMM => self.sbc(Mode::Imm),
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
            DEC_ABSX => self.dec(Mode::AbsX),
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
            STA_ABSX => self.sta(Mode::AbsX),
            STA_ABSY => self.sta(Mode::AbsY),
            STA_ZPX => self.sta(Mode::ZPX),
            STA_INDY => self.sta(Mode::IndY),
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
            ROR_ABSX => self.ror_addr(Mode::AbsX),
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
            LSR_ABSX => self.lsr_addr(Mode::AbsX),
            LSR_ZPX => self.lsr_addr(Mode::ZPX),
            LSR_ABS => self.lsr_addr(Mode::Abs),
            LSR_ZP => self.lsr_addr(Mode::ZP),
            LSR_ACC => self.lsr_acc(),
            JMP_ABS => self.jmp(Mode::Abs),
            ROL_ABS => self.rol_addr(Mode::Abs),
            ROL_ABSX => self.rol_addr(Mode::AbsX),
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
            ASL_ABSX => self.asl_addr(Mode::AbsX),
            ASL_ABS => self.asl_addr(Mode::Abs),
            ASL_ZP => self.asl_addr(Mode::ZP),
            ASL_ZPX => self.asl_addr(Mode::ZPX),
            ASL_ACC => self.asl_acc(),
            RTS => {
                self.pull_pc();
                self.regs.pc.add_unsigned(1);
            }
            RTI => {
                self.pull_status();
                self.pull_pc();
            }
            SED => self.set_flag(Flag::Dec, true),
            CLC => self.set_flag(Flag::Carry, false),
            SEC => self.set_flag(Flag::Carry, true),
            CLI => self.set_flag(Flag::Itr, false),
            SEI => self.set_flag(Flag::Itr, true),
            CLV => self.set_flag(Flag::Overflow, false),
            CLD => self.set_flag(Flag::Dec, false),

            NOP | 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => (),

            // DOP: Double NOP
            0x14 | 0x34 | 0x44 | 0x54 | 0x64 | 0x74 | 0x80 | 0x82 | 0x89
            | 0xC2 | 0xD4 | 0xE2 | 0xF4 | 0x04 => {
                self.regs.pc.add_signed(1);
            }

            // TOP: Triple NOP
            0x0C | 0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                self.regs.pc.add_signed(2);
            }

            BRK => {
                self.push_pc();
                let flags = self.regs.flags;
                self.push(flags);
                self.regs.pc.set_addr(IRQ_VEC);
                self.set_flag(Flag::Brk, true);
            }
            TAX => {
                let acc = self.regs.acc;
                self.regs.x = acc;
                self.set_zero_neg(acc);
            }
            TXA => {
                let x = self.regs.x;
                self.regs.acc = x;
                self.set_zero_neg(x);
            }
            TAY => {
                let acc = self.regs.acc;
                self.regs.y = acc;
                self.set_zero_neg(acc);
            }
            TYA => {
                let y = self.regs.y;
                self.regs.acc = y;
                self.set_zero_neg(y);
            }
            DEX => {
                let x = self.regs.x.wrapping_sub(1);
                self.regs.x = x;
                self.set_zero_neg(x);
            }
            INX => {
                let x = self.regs.x.wrapping_add(1);
                self.regs.x = x;
                self.set_zero_neg(x);
            }
            DEY => {
                let y = self.regs.y.wrapping_sub(1);
                self.regs.y = y;
                self.set_zero_neg(y);
            }
            INY => {
                let y = self.regs.y.wrapping_add(1);
                self.regs.y = y;
                self.set_zero_neg(y);
            }
            TSX => {
                let sp = self.regs.sp;
                self.regs.x = sp;
                self.set_zero_neg(sp);
            }
            TXS => {
                let x = self.regs.x;
                self.regs.sp = x;
            }
            PHA => {
                let acc = self.regs.acc;
                self.push(acc);
            }
            PLA => {
                let acc = self.pop();
                self.regs.acc = acc;
                self.set_zero_neg(acc);
            }
            PHP => {
                let flags = self.regs.flags;
                self.push(flags | Flag::Brk as u8);
            }
            PLP => {
                self.pull_status();
            }
            BVS => {
                let flag = self.get_flag(Flag::Overflow);
                self.generic_branch(flag);
            }
            BVC => {
                let flag = !self.get_flag(Flag::Overflow);
                self.generic_branch(flag);
            }
            BMI => {
                let flag = self.get_flag(Flag::Neg);
                self.generic_branch(flag);
            }
            BPL => {
                let flag = !self.get_flag(Flag::Neg);
                self.generic_branch(flag);
            }
            BNE => {
                let flag = !self.get_flag(Flag::Zero);
                self.generic_branch(flag);
            }
            BEQ => {
                let flag = self.get_flag(Flag::Zero);
                self.generic_branch(flag);
            }
            BCS => {
                let flag = self.get_flag(Flag::Carry);
                self.generic_branch(flag);
            }
            BCC => {
                let flag = !self.get_flag(Flag::Carry);
                self.generic_branch(flag);
            }
            JSR => {
                let addr = self.address_mem(Mode::Abs);
                self.regs.pc.add_signed(-1);
                self.push_pc();
                self.regs.pc.set_addr(addr);
            }
            JMP_IND => self.jmp(Mode::JmpIndir),
            _ => panic!("Unsupported op {:X}", op),
        }
    }
}
