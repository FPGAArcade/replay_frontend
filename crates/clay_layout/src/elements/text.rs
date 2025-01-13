use crate::{bindings::*, color::Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TextElementConfigWrapMode {
    /// Wraps on whitespaces not breaking words
    Words = Clay_TextElementConfigWrapMode_CLAY_TEXT_WRAP_WORDS,
    /// Only wraps on new line characters
    Newline = Clay_TextElementConfigWrapMode_CLAY_TEXT_WRAP_NEWLINES,
    /// Never wraps, can overflow of parent layout
    None = Clay_TextElementConfigWrapMode_CLAY_TEXT_WRAP_NONE,
}

pub struct TextElementConfig {
    inner: *mut Clay_TextElementConfig,
}

impl From<TextElementConfig> for *mut Clay_TextElementConfig {
    fn from(value: TextElementConfig) -> Self {
        value.inner
    }
}
impl From<*mut Clay_TextElementConfig> for TextElementConfig {
    fn from(value: *mut Clay_TextElementConfig) -> Self {
        Self { inner: value }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Text {
    pub color: Color,
    /// Clay doesn't manage font, it's up to you to assign an id to each font you are using, and
    /// passing them to the [font_id](Text::font_id) field
    pub font_id: u16,
    pub font_size: u16,
    pub letter_spacing: u16,
    pub line_height: u16,
    pub wrap_mode: TextElementConfigWrapMode,
}

impl Text {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    pub fn font_id(&mut self, id: u16) -> &mut Self {
        self.font_id = id;
        self
    }

    pub fn font_size(&mut self, size: u16) -> &mut Self {
        self.font_size = size;
        self
    }

    pub fn letter_spacing(&mut self, spacing: u16) -> &mut Self {
        self.letter_spacing = spacing;
        self
    }

    pub fn line_height(&mut self, height: u16) -> &mut Self {
        self.line_height = height;
        self
    }

    pub fn wrap_mode(&mut self, mode: TextElementConfigWrapMode) -> &mut Self {
        self.wrap_mode = mode;
        self
    }

    pub fn end(&self) -> TextElementConfig {
        let memory = unsafe { Clay__StoreTextElementConfig((*self).into()) };

        TextElementConfig { inner: memory }
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            color: Color::rgba(0., 0., 0., 0.),
            font_id: 0,
            font_size: 0,
            letter_spacing: 0,
            line_height: 0,
            wrap_mode: TextElementConfigWrapMode::Words,
        }
    }
}

impl From<Clay_TextElementConfig> for Text {
    fn from(value: Clay_TextElementConfig) -> Self {
        Self {
            color: value.textColor.into(),
            font_id: value.fontId,
            font_size: value.fontSize,
            letter_spacing: value.letterSpacing,
            line_height: value.lineHeight,
            wrap_mode: unsafe {
                core::mem::transmute::<u32, TextElementConfigWrapMode>(value.wrapMode)
            },
        }
    }
}
impl From<Text> for Clay_TextElementConfig {
    fn from(value: Text) -> Self {
        Self {
            textColor: value.color.into(),
            fontId: value.font_id,
            fontSize: value.font_size,
            letterSpacing: value.letter_spacing,
            lineHeight: value.line_height,
            wrapMode: value.wrap_mode as _,
        }
    }
}
