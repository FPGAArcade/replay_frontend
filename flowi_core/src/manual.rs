use crate::math_data::{IVec2, Vec2};
use core::ffi::c_void;

#[repr(C)]
pub struct FlString {
    string: *const c_void,
    length: u32,
}

#[repr(C)]
pub struct FlData {
    pub data: *const c_void,
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

impl FlString {
    pub fn new(s: &str) -> Self {
        FlString {
            string: s.as_ptr() as *const c_void,
            length: s.len() as u32,
        }
    }

    pub fn as_str(&self) -> &str {
        let s =
            unsafe { std::slice::from_raw_parts(self.string as *const u8, self.length as usize) };
        std::str::from_utf8(s).unwrap()
    }
}

#[derive(Debug)]
pub struct FlowiError {
    pub message: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl IVec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

pub fn get_last_error() -> FlowiError {
    // TODO: Implement
    FlowiError { message: 0 }
}

pub type Result<T> = core::result::Result<T, FlowiError>;
