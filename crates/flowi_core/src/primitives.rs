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

#[derive(Clone, Copy, Debug, Default)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color32 {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_color(color: Color) -> Self {
        Self {
            r: (color.r * 255.0) as u8,
            g: (color.g * 255.0) as u8,
            b: (color.b * 255.0) as u8,
            a: (color.a * 255.0) as u8,
        }
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct IRect {
    pub min: [i32; 2],
    pub max: [i32; 2],
}

impl IRect {
    pub fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Self {
            min: [min_x, min_y],
            max: [max_x, max_y],
        }
    }
}

pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            min: Vec2::new(x, y),
            max: Vec2::new(x + w, y + h),
        }
    }

    pub fn from_x0y0x1y1(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            min: Vec2::new(x0, y0),
            max: Vec2::new(x1, y1),
        }
    }
}

pub struct Primitive {
    pub rect: Rect,
    pub uvs: [Uv; 4],
    pub colors: [Color32; 4],
    pub _corners: [f32; 4],
    pub _texture_handle: u64,
}

impl Primitive {
    pub fn new(rect: Rect, color: Color32) -> Self {
        Self {
            rect,
            uvs: [Uv::new(0.0, 0.0); 4],
            colors: [color; 4],
            _corners: [0.0; 4],
            _texture_handle: 0,
        }
    }
}
