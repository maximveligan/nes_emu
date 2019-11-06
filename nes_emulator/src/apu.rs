use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize)]
pub struct Apu {
    pulse1: u8,
    pulse2: u8,
    triangle: u8,
    noise: u8,
    dmc: u8,
    control: u8,
    status: u8,
    frame_counter: u8,
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            pulse1: 0,
            pulse2: 0,
            triangle: 0,
            noise: 0,
            dmc: 0,
            control: 0,
            status: 0,
            frame_counter: 0,
        }
    }

    pub fn load(&mut self, addr: u16) -> u8 {
        match addr {
            0x15 => self.read_status(),
            _ => panic!(
                "No other addresses are mapped to reading from the APU {:X}",
                addr
            ),
        }
    }
    pub fn store(&self, _addr: u16, _val: u8) {
        // unimplemented!("Storing to APU not supported");
        //println!("Warning! Storing not implemented for APU");
    }

    fn read_status(&mut self) -> u8 {
        //TODO: this is a placeholder
        self.status
    }
}
