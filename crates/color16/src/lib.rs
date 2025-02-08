#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Color16 {
    pub r: i16,
    pub g: i16,
    pub b: i16,
    pub a: i16,
}

impl Color16 {
    pub fn new_splat(value: i16) -> Self {
        Self::new(value, value, value, value)
    }

    pub fn new(r: i16, g: i16, b: i16, a: i16) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color16 {
    fn default() -> Self {
        Color16::new_splat(0)
    }
}