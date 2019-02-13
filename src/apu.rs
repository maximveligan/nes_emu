pub struct Apu();

impl Apu {
    pub fn new() -> Apu {
        Apu()
    }

    pub fn load(&self, val: u16) -> u8 {
        unimplemented!();
    }
    pub fn store(&self, addr: u16, val: u8) {
        unimplemented!();
    }
}
