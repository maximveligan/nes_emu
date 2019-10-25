// Bubble bobble for some reason tries to write to addresses in the 0x4200+
// region
// Double dragon has chr table corruption
// Dragion warrior 3 has a grey screen on boot
// This could be due to timing issues/accuracy issues in other emulator
// components but it's hard to tell

use serde::Serialize;
use serde::Deserialize;
use rom::ScreenMode;
use rom::ScreenBank;

#[derive(Serialize, Deserialize, Copy, Clone)]
struct Shift {
    val: u8,
    index: u8,
}

impl Shift {
    fn reset(&mut self) {
        self.val = 0;
        self.index = 0;
    }

    fn push(&mut self, val: u8) -> Option<u8> {
        self.val = self.val | (val & 1) << self.index;
        if self.index == 4 {
            let tmp = self.val;
            self.reset();
            Some(tmp)
        } else {
            self.index += 1;
            None
        }
    }
}

bitfield! {
    #[derive(Clone, Debug ,Serialize, Deserialize)]
    struct Ctrl(u8);
    as_byte,      _ : 7, 0;
    chr_rom_mode, _ : 4;
    prg_rom_mode, _ : 3, 2;
    mirroring,    _ : 1, 0;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sxrom {
    shift: Shift,
    ctrl: Ctrl,
    chr_bank_0_offset: usize,
    chr_bank_1_offset: usize,
    prg_bank_offset: usize,
    prg_ram_enabled: bool,
    use_chr_ram: bool,
    last_page_start: usize,
}

impl Sxrom {
    pub fn new(use_chr_ram: bool, last_page_start: usize) -> Sxrom {
        Sxrom {
            shift: Shift {
                val: 0x10,
                index: 0,
            },
            ctrl: Ctrl(0x0C),
            chr_bank_0_offset: 0,
            chr_bank_1_offset: 0,
            prg_bank_offset: 0,
            use_chr_ram: use_chr_ram,
            last_page_start: last_page_start,
            prg_ram_enabled: true, //Default state is 0 = true
        }
    }

    pub fn store_prg(&mut self, address: u16, val: u8, prg_ram: &mut Vec<u8>) {
        if address < 0x6000 {
            info!("Storing to unmapped prg mem {:X}", address);
        } else if address < 0x8000 {
            prg_ram[address as usize - 0x6000] = val;
        } else {
            if (val & 0x80) != 0 {
                self.reset();
                self.ctrl = Ctrl(self.ctrl.as_byte() | 0x0C);
            } else {
                if let Some(val) = self.shift.push(val) {
                    match address {
                        0x8000..=0x9FFF => self.ctrl = Ctrl(val),
                        0xA000..=0xBFFF => {
                            // Equivalent to multiplying by 0x1000 which is
                            // the chr page size
                            let chr_bank_0 = (val & 0b11111) as usize % 8;
                            self.chr_bank_0_offset = chr_bank_0 << 12;
                        }
                        0xC000..=0xDFFF => {
                            let chr_bank_1 = (val & 0b11111) as usize % 8;
                            self.chr_bank_1_offset = chr_bank_1 << 12;
                        }
                        0xE000..=0xFFFF => {
                            let prg_bank_page = (val & 0b1111) as usize;
                            // Equivalent to multiplying by 0x4000 which is
                            // the page size
                            self.prg_bank_offset = prg_bank_page << 14;
                            self.prg_ram_enabled = (val & 0b10000) == 0;
                        }
                        _ => panic!("Impossible to get here"),
                    }
                }
            }
        }
    }

    pub fn ld_prg(
        &self,
        address: u16,
        prg_rom: &Vec<u8>,
        prg_ram: &Vec<u8>,
    ) -> u8 {
        match address {
            0x6000..=0x7FFF => prg_ram[address as usize - 0x6000],
            0x8000..=0xFFFF => prg_rom[self.get_prg_index(address)],
            addr => {
                info!("Reading from unmapped memory {:X}", addr);
                0
            }
        }
    }

    pub fn ld_chr(
        &self,
        address: u16,
        chr_rom: &Vec<u8>,
        chr_ram: &Vec<u8>,
    ) -> u8 {
        if self.use_chr_ram {
            chr_ram[self.get_chr_index(address)]
        } else {
            chr_rom[self.get_chr_index(address)]
        }
    }

    pub fn store_chr(&mut self, address: u16, val: u8, chr_ram: &mut Vec<u8>) {
        if self.use_chr_ram {
            chr_ram[self.get_chr_index(address)] = val;
        } else {
            info!("Attempting to write to chr rom {:X}", address);
        }
    }

    fn get_chr_index(&self, addr: u16) -> usize {
        match self.ctrl.chr_rom_mode() as u8 {
            // & with !0x1000 to ignore low bit in 8KB mode
            0 => ((self.chr_bank_0_offset & !0x1000) + addr as usize),
            1 => match addr {
                0x0000..=0x0FFF => (self.chr_bank_0_offset) + addr as usize,
                0x1000..=0x1FFF => {
                    (self.chr_bank_1_offset) + (addr as usize - 0x1000)
                }
                c => panic!("Chr indices are only 0000-1FFFF {:X}", c),
            },
            _ => panic!("only one bit used here"),
        }
    }

    fn get_prg_index(&self, addr: u16) -> usize {
        match self.ctrl.prg_rom_mode() {
            // Normally this code shifts right by 1, then left by 14, ANDing
            // the base by !0x2000 pulls out the bit that would be set to 0 if
            // we shifted by 1 first
            0 | 1 => {
                ((self.prg_bank_offset & !0x2000) >> 1)
                    + (addr as usize - 0x8000)
            }
            2 => match addr {
                0x8000..=0xBFFF => addr as usize - 0x8000,
                0xC000..=0xFFFF => {
                    self.prg_bank_offset + (addr as usize - 0xC000)
                }
                _ => panic!("addr can't be anything else"),
            },
            3 => match addr {
                0x8000..=0xBFFF => {
                    self.prg_bank_offset + (addr as usize - 0x8000)
                }
                0xC000..=0xFFFF => {
                    self.last_page_start + addr as usize - 0xC000
                }
                a => panic!("addr can't be anything else {:X}", a),
            },
            b => panic!("Can't get anything else {:b}", b),
        }
    }

    pub fn reset(&mut self) {
        self.shift.reset();
        self.ctrl = Ctrl(0x0C);
        self.chr_bank_0_offset = 0;
        self.chr_bank_1_offset = 0;
        self.prg_bank_offset = 0;
        self.prg_ram_enabled = true;
    }

    pub fn get_mirroring(&self) -> &ScreenMode {
        match self.ctrl.mirroring() {
            0 => &ScreenMode::OneScreenSwap(ScreenBank::Upper),
            1 => &ScreenMode::OneScreenSwap(ScreenBank::Lower),
            2 => &ScreenMode::Vertical,
            3 => &ScreenMode::Horizontal,
            _ => panic!("2 bit number can't be greater than 3"),
        }
    }
}
