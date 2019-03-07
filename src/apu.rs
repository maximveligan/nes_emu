pub struct Apu();

impl Apu {
    pub fn new() -> Apu {
        Apu()
    }

    pub fn load(&self, val: u16) -> u8 {
        // unimplemented!("Loading from APU not supported");
        //        println!("Warning! Loading not implemented for APU");
        0
    }
    pub fn store(&self, addr: u16, val: u8) {
        // unimplemented!("Storing to APU not supported");
        //println!("Warning! Storing not implemented for APU");
    }
}
