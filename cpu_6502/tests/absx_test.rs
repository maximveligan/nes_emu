use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_9d, "9d"),
    (test_dd, "dd"),
    (test_bd, "bd"),
    (test_bc, "bc"),
    (test_7d, "7d"),
    (test_5d, "5d"),
    (test_3d, "3d"),
    (test_1d, "1d") // (test_9c, "9c"),
                    // (test_1c, "1c"),
                    // (test_3c, "3c"),
                    // (test_5c, "5c"),
                    // (test_7c, "7c"),
                    // (test_dc, "dc"),
                    // (test_fc, "fc")
);
