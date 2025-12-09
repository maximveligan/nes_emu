use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    // (test_db, "db"),
    (test_99, "99") // (test_fb, "fb"),
                    // (test_1b, "1b"),
                    // (test_3b, "3b"),
                    // (test_5b, "5b"),
                    // (test_7b, "7b")
);
