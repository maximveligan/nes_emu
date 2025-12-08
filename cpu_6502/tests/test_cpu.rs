extern crate cpu_6502;
use cpu_6502::{
    Memory,
    cpu::{Cpu, Flags, ProgramCounter, Registers},
};
use serde::{Deserialize, de};
use serde_json;
use std::fmt;
use std::fs::File;
use std::path::Path;

const OPS: [u8; 151] = [
    0xFE, 0xFD, 0xED, 0xF9, 0xF6, 0xF5, 0xF1, 0xEE, 0xF8, 0xEC, 0xEA, 0xE9, 0xE8, 0xE6, 0xE5, 0xE4,
    0xE1, 0xE0, 0xDD, 0xD9, 0xD8, 0xD6, 0xD5, 0xD1, 0xD0, 0xCE, 0xDE, 0xCD, 0xCC, 0xCA, 0xC9, 0xC8,
    0xC6, 0xC5, 0xC4, 0xC1, 0xC0, 0xF0, 0xBD, 0xBC, 0xBA, 0xB9, 0xB8, 0xB6, 0xB5, 0xB4, 0xB1, 0xB0,
    0xAE, 0xBE, 0xAD, 0xAC, 0xAA, 0xA9, 0xA8, 0xA6, 0xA5, 0xA4, 0xA2, 0xA1, 0xA0, 0x9D, 0x9A, 0x99,
    0x98, 0x96, 0x95, 0x94, 0x91, 0x90, 0x8E, 0x8D, 0x8C, 0x8A, 0x88, 0x86, 0x85, 0x84, 0x81, 0x7E,
    0x7D, 0x79, 0x78, 0x76, 0x75, 0x71, 0x70, 0x6E, 0x6D, 0x6C, 0x6A, 0x69, 0x68, 0x66, 0x65, 0x61,
    0x60, 0x5E, 0x5D, 0x59, 0x58, 0x56, 0x55, 0x51, 0x50, 0x4E, 0x4D, 0x4C, 0x4A, 0x49, 0x48, 0x46,
    0x45, 0x41, 0x40, 0x3E, 0x3D, 0x39, 0x38, 0x36, 0x35, 0x31, 0x30, 0x2E, 0x2D, 0x2C, 0x2A, 0x29,
    0x28, 0x26, 0x25, 0x24, 0x21, 0x20, 0x1E, 0x1D, 0x19, 0x18, 0x16, 0x15, 0x11, 0x10, 0x0E, 0x0D,
    0x0A, 0x09, 0x08, 0x06, 0x05, 0x01, 0x00,
];

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

#[derive(Deserialize, PartialEq, PartialOrd, Debug)]
struct BusOp {
    addr: u16,
    val: u8,
    inst_type: InstructionType,
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
struct CpuState {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<(usize, u8)>,
}

impl CpuState {
    fn turn_into(&self) -> Registers {
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
struct TestData {
    name: String,
    initial: CpuState,
    #[serde(rename = "final")]
    ending: CpuState,
    cycles: Vec<BusOp>,
}

struct TestMem {
    mem: Box<[u8]>,
    cycle_logs: Vec<BusOp>,
    cycle: usize,
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

#[test]
fn cpu_stress() {
    for op in OPS.iter() {
        let path = format!("./tests/65x02/nes6502/v1/{:02x}.json", op);
        println!("{}", path);
        let test_json = Path::new(&path);
        let tests: Vec<TestData> =
            serde_json::from_reader(File::open(test_json).expect("File should exist"))
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
            let cc = cpu.step(&mut memory);

            // Check if the registers are in a healthy state
            assert_eq!(
                test.ending.pc,
                cpu.regs.pc.get_addr(),
                "{} failed, expected {}, got {}",
                test.name,
                test.ending.pc,
                cpu.regs.pc.get_addr(),
            );
            assert_eq!(
                test.ending.s, cpu.regs.sp,
                "{} failed, expected {}, got {}",
                test.name, test.ending.s, cpu.regs.sp
            );
            assert_eq!(
                test.ending.a, cpu.regs.acc,
                "{} failed, expected {}, got {}",
                test.name, test.ending.a, cpu.regs.acc
            );
            assert_eq!(
                test.ending.x, cpu.regs.x,
                "{} failed, expected {}, got {}",
                test.name, test.ending.x, cpu.regs.x
            );
            assert_eq!(
                test.ending.y, cpu.regs.y,
                "{} failed, expected {}, got {}",
                test.name, test.ending.y, cpu.regs.y
            );
            assert_eq!(
                test.ending.p, cpu.regs.flags.0,
                "{} failed, expected {}, got {}",
                test.name, test.ending.p, cpu.regs.flags.0
            );

            assert_eq!(
                test.cycles.len(),
                cc,
                "{} failed, incorrect cycle count. Expected {}, got {}",
                test.name,
                test.cycles.len(),
                cc
            );

            // Check all RAM entries
            for entry in test.ending.ram.iter() {
                let (addr, val) = entry;
                assert_eq!(
                    memory.mem[*addr], *val,
                    "{} failed, addr: {} expected {}, got val: {}",
                    test.name, addr, val, memory.mem[*addr]
                );
            }

            // Check all cycles
            for (i, cycle) in test.cycles.iter().enumerate() {
                // println!("{:?}\t{:?}", *cycle, memory.cycle_logs[i]);
                // assert_eq!(
                //     memory.cycle_logs[i], *cycle,
                //     "{} failed, cycle {}, expected {:?}, got {:?}",
                //     test.name, i, test.cycles, memory.cycle_logs
                // );
            }
            // println!();

            assert_eq!(
                memory.cycle_logs.len(),
                cc,
                "{} failed, returned cycle count {} and calculated count {} were not the same",
                test.name,
                cc,
                memory.cycle_logs.len()
            );
        }
    }
}
