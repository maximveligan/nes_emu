extern crate cpu_6502;
use cpu_6502::{
    Memory,
    cpu::{Flags, ProgramCounter, Registers},
};
use serde::{Deserialize, de};
use std::fmt;

#[derive(PartialEq, PartialOrd, Debug)]
enum InstructionType {
    Read,
    Write,
}

impl<'de> Deserialize<'de> for InstructionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V;

        impl<'de> de::Visitor<'de> for V {
            type Value = InstructionType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string with value of either read or write")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "read" => Ok(InstructionType::Read),
                    "write" => Ok(InstructionType::Write),
                    s => Err(de::Error::custom(format!(
                        "Got an unexpected string: {}",
                        s
                    ))),
                }
            }
        }
        deserializer.deserialize_any(V)
    }
}

#[derive(Deserialize, PartialEq, PartialOrd)]
pub struct BusOp {
    addr: u16,
    val: u8,
    inst_type: InstructionType,
}

impl fmt::Debug for BusOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inst_type {
            InstructionType::Read => {
                write!(
                    f,
                    "OP - addr: {:02X} val: {:02X} Read",
                    &self.addr, &self.val
                )
            }
            InstructionType::Write => {
                write!(
                    f,
                    "OP - addr: {:02X} val: {:02X} Write",
                    &self.addr, &self.val
                )
            }
        }
    }
}

impl Default for BusOp {
    fn default() -> Self {
        Self {
            addr: 0,
            val: 0,
            inst_type: InstructionType::Write,
        }
    }
}

impl BusOp {
    fn new(addr: u16, val: u8, inst_type: InstructionType) -> Self {
        BusOp {
            addr,
            val,
            inst_type,
        }
    }
}

#[derive(Deserialize, PartialEq, PartialOrd, Debug)]
pub struct CpuState {
    pub pc: u16,
    pub s: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub ram: Vec<(usize, u8)>,
}

impl CpuState {
    pub fn turn_into(&self) -> Registers {
        Registers {
            acc: self.a,
            x: self.x,
            y: self.y,
            pc: ProgramCounter::new(self.pc),
            sp: self.s,
            flags: Flags(self.p),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct TestData {
    pub name: String,
    pub initial: CpuState,
    #[serde(rename = "final")]
    pub ending: CpuState,
    pub cycles: Vec<BusOp>,
}

pub struct TestMem {
    pub mem: Box<[u8]>,
    pub cycle_logs: Vec<BusOp>,
    pub cycle: usize,
}

impl Memory for TestMem {
    fn ld8(&mut self, addr: u16) -> u8 {
        self.cycle += 1;
        let tmp = self.mem[addr as usize];
        self.cycle_logs
            .push(BusOp::new(addr, tmp, InstructionType::Read));
        tmp
    }

    fn ld16(&mut self, addr: u16) -> u16 {
        let l_byte = self.ld8(addr);
        let r_byte = self.ld8(addr + 1);
        (r_byte as u16) << 8 | (l_byte as u16)
    }

    fn store(&mut self, addr: u16, val: u8) {
        self.cycle += 1;
        self.mem[addr as usize] = val;
        self.cycle_logs
            .push(BusOp::new(addr, val, InstructionType::Write));
    }
}

#[macro_export]
macro_rules! test_op {
    ( $(($test_name:ident, $path:literal)),* ) => {
        $(
            #[test]
            #[ignore]
            fn $test_name() {
                let path = format!("./tests/65x02/nes6502/v1/{}.json", $path);
                println!("{}", path);
                let test_json = ::std::path::Path::new(&path);
                let tests: Vec<TestData> =
                    serde_json::from_reader(::std::fs::File::open(test_json).expect("File should exist"))
                        .expect("Serde failed");
                for test in tests.iter() {
                    let mut memory = TestMem {
                        mem: Box::new([0; 0x10000]),
                        cycle_logs: Vec::with_capacity(test.cycles.len()),
                        cycle: 0,
                    };

                    for entry in test.initial.ram.iter() {
                        let (addr, val) = entry;
                        memory.mem[*addr] = *val;
                    }

                    let mut cpu = Cpu::from_registers(test.initial.turn_into());
                    cpu.step(&mut memory);

                    // Check if the registers are in a healthy state
                    assert_eq!(
                        test.ending.pc,
                        cpu.regs.pc.get_addr(),
                        "{} failed, PC: expected {}, got {}",
                        test.name,
                        test.ending.pc,
                        cpu.regs.pc.get_addr(),
                    );
                    assert_eq!(
                        test.ending.s, cpu.regs.sp,
                        "{} failed, SP: expected {}, got {}",
                        test.name, test.ending.s, cpu.regs.sp
                    );
                    assert_eq!(
                        test.ending.a, cpu.regs.acc,
                        "{} failed, ACC: expected {}, got {}",
                        test.name, test.ending.a, cpu.regs.acc
                    );
                    assert_eq!(
                        test.ending.x, cpu.regs.x,
                        "{} failed, X: expected {}, got {}",
                        test.name, test.ending.x, cpu.regs.x
                    );
                    assert_eq!(
                        test.ending.y, cpu.regs.y,
                        "{} failed, Y: expected {}, got {}",
                        test.name, test.ending.y, cpu.regs.y
                    );
                    assert_eq!(
                        test.ending.p, cpu.regs.flags.0,
                        "{} failed, FLAGS: expected {}, got {}",
                        test.name, test.ending.p, cpu.regs.flags.0
                    );

                    // Check all cycles
                    for (i, cycle) in test.cycles.iter().enumerate() {
                        println!("{:?} {:?}\t{:?}", cpu.regs, *cycle, memory.cycle_logs[i]);
                        assert_eq!(
                            memory.cycle_logs[i], *cycle,
                            "{} failed, cycle {}, expected {:?}, got {:?}",
                            test.name, i, test.cycles, memory.cycle_logs
                        );
                    }
                    println!();

                    // Check all RAM entries
                    for entry in test.ending.ram.iter() {
                        let (addr, val) = entry;
                        assert_eq!(
                            memory.mem[*addr], *val,
                            "{} failed, addr: {} expected {}, got val: {}",
                            test.name, addr, val, memory.mem[*addr]
                        );
                    }
                }
            }
        )*
    };
}
