use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_b6, "b6"),
    (test_96, "96"),
    (test_97, "97"),
    (test_b7, "b7")
);
