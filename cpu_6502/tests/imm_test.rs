use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_a0, "a0"),
    (test_e9, "e9"),
    (test_eb, "eb"),
    (test_6b, "6b"),
    (test_cb, "cb"),
    (test_69, "69"),
    (test_4b, "4b"),
    (test_49, "49"),
    // (test_ab, "ab"),
    (test_29, "29"),
    (test_09, "09"),
    (test_e0, "e0"),
    (test_a2, "a2"),
    (test_0b, "0b"),
    (test_2b, "2b"),
    (test_c9, "c9"),
    (test_c0, "c0"),
    (test_a9, "a9")
);
