extern crate hex;
extern crate nes_emu;
extern crate serde;
extern crate sha3;
use nes_emu::NesEmulator;
use nes_emu::rom::load_rom;
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
    ("c32a9adc7129c3e54e2df3071cef2352369383effb0910e3cedaf89f8424171b", 90,
     "./tests/nes_test_roms/sprite_hit_tests_2005.10.05/10.timing_order.nes",
     sprite_timing_order),
    ("719218b3867506b4432ca754a2dfffbf9f9bbdb55d185857b2972ad7e8c5f9ec", 70,
     "./tests/nes_test_roms/sprite_hit_tests_2005.10.05/11.edge_timing.nes",
     sprite_edge_timing),
    ("671c5d5913a8d482d27b99efdcc01570348abdc436dd4e195cf1d15392adc196", 1000,
     "./tests/nes_test_roms/cpu_timing_test6/cpu_timing_test.nes",
     cpu_timing_test),
    ("e9f600fd64251c4b3406c43b29bbc9063beb66cd794087f72dc798c8cb5f478b", 20,
     "./tests/nes_test_roms/blargg_ppu_tests_2005.09.15b/vram_access.nes",
     vram_access),
    ("a3e7aab843909d5f7e1f8eeb2369953a79dd429aeccc6df7e702a868a1928da8", 20,
     "./tests/nes_test_roms/branch_timing_tests/2.Backward_Branch.nes",
     backward_branch),
    ("1308eaef18c0a1f7ca1286dbce5390cdc8c0a5fcfb072e3baa7afb767c36120b", 17,
     "./tests/nes_test_roms/branch_timing_tests/3.Forward_Branch.nes",
     forward_branch),
    ("1cf5bf323cdb11b2aac8cd23e41cb391b76916cc523732454a04bf98118bb84e", 182,
     "./tests/nes_test_roms/vbl_nmi_timing/1.frame_basics.nes",
     frame_basics),
    ("e8f42f51deceecdb188c407dde9160ee2a4d10d6c80afdf966a6366a2807e1b1", 250,
     "./tests/nes_test_roms/vbl_nmi_timing/2.vbl_timing.nes",
     vbl_timing),
    ("1caeb37376173ff09101b6984f6c523bc153dc9eef7558e13e524a7688ff1669", 131,
     "./tests/nes_test_roms/vbl_nmi_timing/4.vbl_clear_timing.nes",
     vbl_clear_timing),
    ("ae3d4e43ed8be383e3a05dedeebe831c044c224c8b2b3f24ad2229dbd4be5fa9", 151,
     "./tests/nes_test_roms/ppu_vbl_nmi/rom_singles/01-vbl_basics.nes",
     vbl_basics),
    ("678a177d334fe8d0ec62f7dc402d7168a5d6ca14bdbe2eebe8ef28be442558cf", 183,
     "./tests/nes_test_roms/ppu_vbl_nmi/rom_singles/02-vbl_set_time.nes",
     vbl_set_time),
    ("2a64667e1cf639022a8c7c1821132ad2072bb1e12f4b76f0d8ddf16ffa0a05c1", 185,
     "./tests/nes_test_roms/ppu_vbl_nmi/rom_singles/03-vbl_clear_time.nes",
     vbl_clear_time),
    ("807c87797a98f160664e28a62e8d009560d781a6dbc384dc4f7e243994954fc0", 171,
     "./tests/nes_test_roms/ppu_vbl_nmi/rom_singles/04-nmi_control.nes",
     nmi_control),
    ("e83c004cb2aacd68eb2d38cf9166e268483074ed57b40c68478b86c5a3022809", 208,
     "./tests/nes_test_roms/ppu_vbl_nmi/rom_singles/07-nmi_on_timing.nes",
     nmi_on_timing)
}
