extern crate hex;
extern crate nes_emu;
extern crate serde;
extern crate sha3;
use nes_emu::rom::load_rom;
use nes_emu::NesEmulator;
use sha3::Digest;
use sha3::Sha3_256;
use std::fs::File;
use std::io::Read;

macro_rules! hash_test {
    ( $(($hash:literal, $frame_num:literal,
         $rom_path:literal, $test_name:ident)),* ) => {
            $(
                #[test]
                fn $test_name() {
                    let mut raw_bytes = Vec::new();
                    let mut raw_rom = File::open($rom_path).expect(
                        "Expected a valid path");
                    raw_rom.read_to_end(&mut raw_bytes).expect(
                        "Failure to read the file");
                    let rom = load_rom(&raw_bytes).expect(
                        "Expected a valid rom");
                    let mut nes = NesEmulator::new(rom);
                    for _ in 0..$frame_num {
                        nes.next_frame();
                    }
                    assert_eq!(
                        $hash,
                        hex::encode(Sha3_256::digest(nes.get_pixel_buffer())));
                }
            )*
    };
}

hash_test! {
    ("a96ed5458e27b41e7b87dc9d77cdc62cdcf2762bacf9183e4acb4fd09a1b8b31", 54,
     "./tests/nes_test_roms/sprite_hit_tests_2005.10.05/01.basics.nes",
     sprite_basics),
    ("4c141634e8ab8bbd3d67e0b248af8b4d503ce5e40f4f9e6f6b9627d5c9b562fc", 33,
     "./tests/nes_test_roms/sprite_hit_tests_2005.10.05/02.alignment.nes",
     sprite_alignment),
    ("c8593784f93d3b6b08907d31d1b1f26c69c5474f1d9a6beb1cd1ecce6e2c98ad", 25,
     "./tests/nes_test_roms/sprite_hit_tests_2005.10.05/03.corners.nes",
     sprite_corners),
    ("5d85b331a8e820583e2522b78703311454cbcb0d5590457bfdea49e5a8385c48", 23,
     "./tests/nes_test_roms/sprite_hit_tests_2005.10.05/04.flip.nes",
     sprite_flip),
    ("a2751701225f8c577b08c512e04226bf9b3addedb8d2953221d4c7925bac4843", 33,
     "./tests/nes_test_roms/sprite_hit_tests_2005.10.05/08.double_height.nes",
     sprite_double_height),
    ("c32a9adc7129c3e54e2df3071cef2352369383effb0910e3cedaf89f8424171b", 60,
     "./tests/nes_test_roms/sprite_hit_tests_2005.10.05/10.timing_order.nes",
     sprite_timing_order),
    ("719218b3867506b4432ca754a2dfffbf9f9bbdb55d185857b2972ad7e8c5f9ec", 70,
     "./tests/nes_test_roms/sprite_hit_tests_2005.10.05/11.edge_timing.nes",
     sprite_edge_timing),
    ("bd0f14f44d6acc6e7c4f6b3f57f86146b5777b6daa2392823fd57d239f255b5e", 2354,
     "./tests/nes_test_roms/instr_test-v3/all_instrs.nes",
     cpu_all_instrs),
    ("671c5d5913a8d482d27b99efdcc01570348abdc436dd4e195cf1d15392adc196", 631,
     "./tests/nes_test_roms/cpu_timing_test6/cpu_timing_test.nes",
     cpu_timing_test),
    ("abce9d533a663bc88e0f5b88671d8e5d8e0267e6fb75fe301ea18e90158c189b", 660,
     "./tests/nes_test_roms/blargg_nes_cpu_test5/official.nes",
     cpu_official),
    ("e9f600fd64251c4b3406c43b29bbc9063beb66cd794087f72dc798c8cb5f478b", 20,
     "./tests/nes_test_roms/blargg_ppu_tests_2005.09.15b/vram_access.nes",
     vram_access)
}
