use pregisters::Ctrl;

const SPRITE_ATTR: usize = 4;

#[derive(Copy, Clone)]
pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub pt_index: u8,
    pub attributes: SpriteAttr,
    pub index: u8,
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
        }
    }

    pub fn in_bounding_box(&self, x: u8, y: u8, left8_on: bool) -> bool {
        !(self.x > x || self.x + 8 <= x
        || ((x <= 8) && !left8_on)
        // These last 2 cases are for accounting for overscan
        || y <= 8
        // This is screenheight - 8
        || y >= 240 - 8)
    }

    // TODO: There is a bug somewhere in here that has to do with 16 pixel tall sprites
    pub fn get_tile_values(&self, ctrl: &Ctrl, x: u8, y: u16) -> (u16, u8) {
        let pt_i = match ctrl.sprite_size() {
            8 => (ctrl.sprite_pt_addr() + self.pt_index as u16),
            16 => {
                let tile_num = self.pt_index & !1;
                let offset = ((self.pt_index as u16) & 1) * 0x1000;
                (tile_num as u16 + offset)
            }
            _ => panic!("No other sprite sizes"),
        } * 16;

        let x = if self.attributes.flip_x() {
            7 - (x.wrapping_sub(self.x))
        } else {
            x.wrapping_sub(self.x)
        };

        //TODO: No idea why the - 1 is necassary here, it's a complete hack
        let tmp = ((y - 1 - self.y as u16) % ctrl.sprite_size() as u16) as u16;

        let y = if self.attributes.flip_y() {
            ctrl.sprite_size() as u16 - 1 - tmp
        } else {
            tmp
        };
        
        ((pt_i + y + if y < 8 { 0 } else { 8 }), x)
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
