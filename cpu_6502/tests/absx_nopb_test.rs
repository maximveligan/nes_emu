use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_fe, "fe"),
    (test_de, "de"),
    (test_9d, "9d"),
    (test_7e, "7e"),
    (test_5e, "5e"),
    (test_3e, "3e"),
    //(test_df, "df"),
    (test_1e, "1e") // (test_7f, "7f"),
                    // (test_3f, "3f"),
                    // (test_1f, "1f"),
                    // (test_ff, "ff")
                    // (test_5f, "5f")
);
