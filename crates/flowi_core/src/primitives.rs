#[repr(C)]
pub struct FlData {
    pub data: *const core::ffi::c_void,
    pub size: u64,
}

impl Default for FlData {
    fn default() -> Self {
        Self {
            data: std::ptr::null(),
            size: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Uv {
    pub u: f32,
    pub v: f32,
}

impl Uv {
    pub fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }

    pub fn interpolate(a: Uv, b: Uv, t: f32) -> Uv {
        Uv {
            u: a.u + (b.u - a.u) * t,
            v: a.v + (b.v - a.v) * t,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

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
