use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_f9, "f9"),
    (test_be, "be"),
    (test_d9, "d9"),
    (test_b9, "b9"),
    (test_79, "79"),
    (test_59, "59"),
    (test_39, "39"),
    (test_19, "19"),
    (test_bf, "bf") // (test_9e, "9e")
);
