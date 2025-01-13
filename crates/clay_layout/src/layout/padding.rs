use crate::bindings::*;

#[derive(Debug, Copy, Clone, Default)]
pub struct Padding {
    pub x: u16,
    pub y: u16,
}

impl Padding {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl From<Clay_Padding> for Padding {
    fn from(value: Clay_Padding) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
impl From<Padding> for Clay_Padding {
    fn from(value: Padding) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<(u16, u16)> for Padding {
    fn from(other: (u16, u16)) -> Self {
        Self::new(other.0, other.1)
    }
}
