use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_51, "51"),
    (test_31, "31"),
    (test_11, "11"),
    (test_f1, "f1"),
    (test_d1, "d1"),
    (test_b1, "b1"),
    (test_71, "71"),
    (test_b3, "b3")
);
