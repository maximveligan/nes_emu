use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_e1, "e1"),
    (test_c1, "c1"),
    (test_a1, "a1"),
    (test_81, "81"),
    (test_61, "61"),
    (test_41, "41"),
    (test_21, "21"),
    (test_01, "01"),
    (test_83, "83"),
    (test_a3, "a3") // (test_c3, "c3"),
                    // (test_e3, "e3"),
                    // (test_03, "03"),
                    // (test_23, "23"),
                    // (test_43, "43"),
                    // (test_63, "63")
);
