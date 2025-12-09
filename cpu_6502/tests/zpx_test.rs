use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_f6, "f6"),
    (test_f5, "f5"),
    (test_d6, "d6"),
    (test_d5, "d5"),
    (test_b5, "b5"),
    (test_b4, "b4"),
    (test_95, "95"),
    (test_94, "94"),
    (test_75, "75"),
    (test_76, "76"),
    (test_55, "55"),
    (test_56, "56"),
    (test_36, "36"),
    (test_35, "35"),
    (test_15, "15"),
    (test_16, "16") // (test_d7, "d7"),
                    // (test_f7, "f7"),
                    // (test_17, "17"),
                    // (test_37, "37"),
                    // (test_57, "57"),
                    // (test_77, "77")
);
