use super::pregisters::Ctrl;

const SPRITE_ATTR: usize = 4;

#[derive(Copy, Clone)]
pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub pt_index: u8,
    pub attributes: SpriteAttr,
    pub index: u8,
    pub low_byte: u8,
    pub high_byte: u8,
}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct SpriteAttr(u8);
    pub palette,  _: 1, 0;
    pub priority, _:    5;
    pub flip_x,   _:    6;
    pub flip_y,   _:    7;
}

impl Sprite {
    pub fn new(index: usize, oam: &[u8; 256]) -> Sprite {
        Sprite {
            index: index as u8,
            x: oam[(index * SPRITE_ATTR) + 3],
            y: oam[(index * SPRITE_ATTR)],
            pt_index: oam[(index * SPRITE_ATTR) + 1],
            attributes: SpriteAttr(oam[(index * SPRITE_ATTR) + 2]),
            low_byte: 0,
            high_byte: 0,
        }
    }

    pub fn in_bounding_box(&self, x: u8) -> bool {
        !(self.x > x || self.x + 8 <= x)
    }

    pub fn get_pt_address(&self, ctrl: &Ctrl, y: u16) -> u16 {
        let pt_i = match ctrl.sprite_size() {
            8 => ctrl.sprite_pt_addr() + (16 * (self.pt_index as u16)),
            16 => {
                let offset = ((self.pt_index & !1) as u16) * 16;
                let base = ((self.pt_index & 1) as u16) * 0x1000;
                base + offset
            }
            _ => panic!("No other sprite sizes"),
        };

        let tmp = y - self.y as u16;

        let y = if self.attributes.flip_y() {
            ctrl.sprite_size() as u16 - 1 - tmp
        } else {
            tmp
        };

        // Grabs the adjacent tile if this is a 16 bit sprite and the y value
        // is greater than 7
        let y_offset = if y < 8 { 0 } else { 8 };

        pt_i + y + y_offset
    }
}

pub enum Priority {
    Foreground,
    Background,
}

impl Priority {
    pub fn from_attr(bit: u8) -> Priority {
        match bit {
            0 => Priority::Foreground,
            1 => Priority::Background,
            _ => panic!("Can't get number either than 0 or 1 after anding"),
        }
    }
}
