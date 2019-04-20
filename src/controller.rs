const SIG_BYTE: u8 = 0x40;

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Button {
    A = 0b0000_0001,
    B = 0b0000_0010,
    Select = 0b0000_0100,
    Start = 0b0000_1000,
    Up = 0b0001_0000,
    Down = 0b0010_0000,
    Left = 0b0100_0000,
    Right = 0b1000_0000,
}

pub struct Controller {
    ctrl_state: u8,
    strobe: bool,
    shift: usize,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            ctrl_state: 0,
            strobe: false,
            shift: 0,
        }
    }

    pub fn ld8(&mut self) -> u8 {
        let val = if self.shift < 8 {
            (self.ctrl_state >> self.shift) & 1
        } else {
            1
        };

        if !self.strobe {
            self.shift += 1;
        }
        // Required by some games. Check nesdev controllers for more info
        SIG_BYTE | val
    }

    pub fn store(&mut self, val: u8) {
        self.strobe = val & 1 != 0;
        if self.strobe {
            self.shift = 0;
        }
    }

    pub fn set_button_state(&mut self, button: Button, pressed: bool) {
        if pressed {
            self.ctrl_state |= button as u8;
        } else {
            self.ctrl_state &= !(button as u8);
        }
    }
}
