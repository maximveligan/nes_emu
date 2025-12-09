use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_91, "91") // (test_d3, "d3"),
                    // (test_f3, "f3"),
                    // (test_13, "13"),
                    // (test_33, "33"),
                    // (test_53, "53"),
                    // (test_73, "73")
);
