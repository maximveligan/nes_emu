use cpu_6502::cpu::Cpu;
use utilities::TestData;
use utilities::TestMem;
use utilities::test_op;

extern crate utilities;

test_op!(
    (test_fd, "fd"),
    (test_ec, "ec"),
    (test_ae, "ae"),
    (test_ce, "ce"),
    (test_cd, "cd"),
    (test_cc, "cc"),
    (test_bd, "bd"),
    (test_bc, "bc"),
    (test_8e, "8e"),
    (test_8c, "8c"),
    (test_7d, "7d"),
    (test_6e, "6e"),
    (test_5d, "5d"),
    (test_4e, "4e"),
    (test_4c, "4c"),
    (test_3e, "3e"),
    (test_3d, "3d"),
    (test_2c, "2c"),
    (test_1d, "1d"),
    (test_0e, "0e"),
    (test_8f, "8f"),
    (test_af, "af") // Undocumented opcodes
                    // (test_cf, "cf"),
                    // (test_ef, "ef"),
                    // (test_0f, "0f"),
                    // (test_2f, "2f"),
                    // (test_4f, "4f"),
                    // (test_0c, "0c"),
                    // (test_6f, "6f")
);
