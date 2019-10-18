use serde::Serialize;
use serde::Deserialize;
use rom::ScreenMode;

#[derive(Serialize, Deserialize, Clone)]
pub struct Txrom {
    use_chr_ram: bool,
    bank_select: BankSelect,
    last_page_start: usize,
    mirroring: ScreenMode,
    ram_enabled: bool,
    allow_writes: bool,
}

bitfield! {
    #[derive(Serialize, Deserialize, Clone, Copy)]
    struct BankSelect(u8);
    byte, set_byte:       7, 0;
    chr_a12_inversion, _: 7;
    prg_rom_mode, _:      6;
    prg_ram_enabled, _:   5;
    bank_reg_select, _:   2, 0;
}

impl Txrom {
    pub fn new(use_chr_ram: bool, last_page_start: usize) -> Txrom {
        Txrom {
            use_chr_ram,
            bank_select: BankSelect(0),
            last_page_start,
            ram_enabled: false,
            allow_writes: false,
            mirroring: ScreenMode::Horizontal,
        }
    }

    pub fn ld_prg(&self, address: u16, prg_rom: &Vec<u8>, prg_ram: &Vec<u8>) -> u8 {
        match address {
            // This is optional, check what that means
            0x6000..=0x7FFF => prg_ram[address as usize - 0x6000],
            // 0x8000..=0x9FFF => (or $C000-$DFFF): 8 KB switchable PRG ROM bank
            // 0xA000..=0xBFFF =>  8 KB switchable PRG ROM bank
            // This one sometimes switches, read register here
            // 0xC000..=0xDFFF => prg_rom[last_page_start - 0x4000 + address as usize]
            0xE000..=0xFFFF => prg_rom[self.last_page_start - 0x2000 + address as usize],
            _ => panic!(),
        }
    }

    pub fn store_prg(&mut self, addr: u16, val: u8, prg_ram: &mut Vec<u8>) {
        match (addr, addr & 1) {
            (0x0000..=0x7FFF, _) => {prg_ram[addr as usize] = val},
            (0x8000..=0x9FFF, even_odd) => self.bank_ops(even_odd == 0, val),
            (0xA000..=0xBFFF, even_odd) => self.misc_ops(even_odd == 0, val),
            _ => panic!(),
        }
    }

    fn bank_ops(&mut self, even: bool, val: u8) {
        if even {
            self.bank_select.set_byte(val);
        } else {
            match self.bank_select.bank_reg_select() {
                0b000 => unimplemented!(),
                0b001 => unimplemented!(),
                0b010 => unimplemented!(),
                0b011 => unimplemented!(),
                0b100 => unimplemented!(),
                0b101 => unimplemented!(),
                0b110 => unimplemented!(),
                0b111 => unimplemented!(),
                _ => panic!("Can't get here"),
            }
        }
    }

    fn misc_ops(&mut self, even: bool, val: u8) {
        if even {
            self.mirroring = match val & 0b1 {
                0 => ScreenMode::Vertical,
                1 => ScreenMode::Horizontal,
                _ => panic!("Impossible"),
            }
        } else {
            // This should be >> 4 on MMC6, implement it at some point
            let shifted_val = val >> 4;
            self.allow_writes = shifted_val & 0b0100 == 0;
            self.ram_enabled = shifted_val & 0b1000 == 1;
        }
    }

    pub fn ld_chr(
        &self,
        _address: u16,
        _chr_rom: &Vec<u8>,
        _chr_ram: &Vec<u8>,
    ) -> u8 {
        0
    }

    pub fn store_chr(&mut self, _address: u16, _val: u8, _chr_ram: &mut Vec<u8>) {
    }
}
