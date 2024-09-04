use arena_allocator::TypedArena;

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

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

}

#[derive(Debug, Default)]
struct DrawText {
    //codepoints: &'a [u32],
    position: Vec2,
    size: f32,
    font_id: usize,
    color: Color,
}

#[derive(Debug)]
pub enum DrawCorner {
    TopLeft(Vec2, f32, Color),
    TopRight(Vec2, f32, Color),
    BottomLeft(Vec2, f32, Color),
    BottomRight(Vec2, f32, Color),
}

impl Default for DrawCorner {
    fn default() -> Self {
        DrawCorner::TopLeft(Vec2::default(), 0.0, Color::default())
    }
}

#[derive(Debug, Default)]
struct DrawRect {
    position: Vec2,
    size: Vec2,
    color: Color,
}

struct Primitives {
    pub(crate) draw_text: TypedArena<DrawText>,
    pub(crate) draw_rect: TypedArena<DrawRect>,
    pub(crate) draw_corners: TypedArena<DrawCorner>,
}
