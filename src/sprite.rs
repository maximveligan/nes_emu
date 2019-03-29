const SPRITE_ATTR: usize = 4;

#[derive(Copy, Clone)]
pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub pt_index: u8,
    pub attributes: SpriteAttr,
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
            x: oam[(index * SPRITE_ATTR) + 3],
            y: oam[(index * SPRITE_ATTR)].wrapping_add(1),
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
