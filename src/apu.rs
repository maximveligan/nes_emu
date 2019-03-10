pub struct Apu();

impl Apu {
    pub fn new() -> Apu {
        Apu()
    }

    pub fn load(&self, _val: u16) -> u8 {
        // unimplemented!("Loading from APU not supported");
        //        println!("Warning! Loading not implemented for APU");
        0
    }
    pub fn store(&self, _addr: u16, _val: u8) {
        // unimplemented!("Storing to APU not supported");
        //println!("Warning! Storing not implemented for APU");
    }
}
