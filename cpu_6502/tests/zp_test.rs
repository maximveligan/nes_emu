use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_55, "55"),
    (test_e5, "e5"),
    (test_76, "76"),
    (test_56, "56"),
    (test_36, "36"),
    (test_35, "35"),
    (test_24, "24"),
    (test_15, "15"),
    (test_06, "06"),
    (test_87, "87"),
    (test_a7, "a7"),
    // (test_c7, "c7"),
    // (test_e7, "e7"),
    // (test_07, "07"),
    // (test_27, "27"),
    // (test_47, "47"),
    // (test_67, "67"),
    (test_f6, "f6"),
    (test_e4, "e4"),
    (test_b6, "b6"),
    (test_d6, "d6"),
    (test_d5, "d5"),
    (test_c4, "c4"),
    (test_b5, "b5"),
    (test_b4, "b4"),
    (test_95, "95"),
    (test_86, "86"),
    (test_84, "84"),
    (test_75, "75")
);
