#[derive(Copy, Clone)]
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
    index: usize,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            ctrl_state: 0,
            strobe: false,
            index: 0,
        }
    }

    pub fn ld8(&mut self) -> u8 {
        let val = if self.index < 8 {
            self.ctrl_state >> self.index & 1
        } else {
            1
        };

        if !self.strobe {
            self.index += 1;
        }
	0x40 | val
    }

    pub fn store(&mut self, val: u8) {
        self.strobe = val & 1 != 0;
        if self.strobe {
            self.index = 0;
        }
    }

    pub fn set_button_state(&mut self, button: Button, pressed: bool) {
        self.ctrl_state &= !(button as u8);
        if pressed {
            self.ctrl_state |= button as u8;
        }
    }
}
