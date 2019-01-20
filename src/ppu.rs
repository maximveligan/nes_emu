pub struct Ppu {
    regs: PRegisters
}

struct PRegisters {
    PPUCTRL: u8,
    PPUMASK: u8,
    PPUSTATUS: u8,
    OAMADDR: u8,
    OAMDATA: u8,
    PPUSCROLL: u8,
    PPUADDR: u8,
    PPUDATA: u8,
    OAMDMA: u8
}
