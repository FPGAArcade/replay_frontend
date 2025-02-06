pub mod font;
mod image;
mod image_api;
pub mod input;
mod internal_error;
mod io_handler;
pub mod primitives;
pub mod render;
pub mod signal;
pub mod widgets;
use crate::input::Input;
use crate::io_handler::IoHandle;
use glam::Vec4;

use crate::image::ImageInfo;
use arena_allocator::Arena;
use background_worker::WorkSystem;
use clay_layout::{
    math::Dimensions, render_commands::RenderCommand as ClayRenderCommand,
    render_commands::RenderCommandConfig, Clay, Clay_Dimensions, Clay_StringSlice,
    Clay_TextElementConfig, TypedConfig,
};
use fileorama::Fileorama;
use internal_error::InternalResult;
pub use io_handler::IoHandler;
use signal::Signal;
use std::cell::UnsafeCell;
use std::collections::HashMap;

pub use crate::io_handler::IoHandle as ImageHandle;
use font::{CachedString, FontHandle};

pub use clay_layout::{
    color::Color as ClayColor,
    elements::image::Image as ClayImage,
    elements::{rectangle::Rectangle, text::Text, CornerRadius},
    fixed, grow,
    id::Id,
    layout::alignment::LayoutAlignmentX,
    layout::alignment::LayoutAlignmentY,
    layout::{alignment::Alignment, padding::Padding, sizing::Sizing, Layout, LayoutDirection},
};

use flowi_renderer::{
    Color, DrawBorderData, DrawImage, DrawRectRoundedData, DrawTextBufferData, RenderCommand,
    RenderType, Renderer, StringSlice,
};

type FlowiKey = u64;

#[derive(Debug, Default)]
#[allow(dead_code)]
struct ItemState {
    aabb: Vec4,
    was_hovered: bool,
    was_clicked: bool,
    hot: f32,
    frame: u64,
}

pub enum BackgroundMode {
    AlignTopRight,
}

struct BackgroundImage {
    handle: IoHandle,
    mode: BackgroundMode,
}

#[allow(dead_code)]
pub(crate) struct State<'a> {
    pub(crate) text_generator: font::TextGenerator,
    pub(crate) vfs: Fileorama,
    pub(crate) io_handler: IoHandler,
    pub(crate) input: Input,
    pub(crate) primitives: Arena,
    pub(crate) hot_item: FlowiKey,
    pub(crate) current_frame: u64,
    pub(crate) layout: Clay<'a>,
    pub(crate) button_id: u32,
    pub(crate) renderer: Box<dyn Renderer>,
    pub(crate) bg_worker: WorkSystem,
    pub(crate) item_states: HashMap<u32, ItemState>, // TODO: Arena hashmap
    pub(crate) active_font: FontHandle,
    pub(crate) background_image: Option<BackgroundImage>,
    pub(crate) screen_size: (usize, usize),
}

#[allow(dead_code)]
pub struct Ui<'a> {
    state: UnsafeCell<State<'a>>,
}

impl<'a> Ui<'a> {
    pub fn new(renderer: Box<dyn Renderer>) -> Box<Self> {
        let vfs = Fileorama::new(2);
        let io_handler = IoHandler::new(&vfs);
        let bg_worker = WorkSystem::new(2);

        crate::image_api::install_image_loader(&vfs);

        let reserve_size = 1024 * 1024 * 1024;
        let state = State {
            vfs,
            io_handler,
            text_generator: font::TextGenerator::new(&bg_worker),
            hot_item: 0,
            input: Input::new(),
            current_frame: 0,
            primitives: Arena::new(reserve_size).unwrap(),
            layout: Clay::new(Dimensions::new(1280.0, 720.0)),
            item_states: HashMap::new(),
            button_id: 0,
            renderer,
            bg_worker,
            active_font: 0,
            background_image: None,
            screen_size: (0, 0),
        };

        let data = Box::new(Ui {
            state: UnsafeCell::new(state),
        });

        // This is a hack. To be fixed later
        unsafe {
            let raw_ptr = Box::into_raw(data);
            clay_layout::Clay::set_measure_text_function_unsafe(
                Self::measure_text_trampoline,
                raw_ptr as usize,
            );
            Box::from_raw(raw_ptr)
        }
    }

    unsafe extern "C" fn measure_text_trampoline(
        text_slice: Clay_StringSlice,
        config: *mut Clay_TextElementConfig,
        user_data: usize,
    ) -> Clay_Dimensions {
        let text = core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            text_slice.chars as *const u8,
            text_slice.length as _,
        ));

        let text_config = Text::from(*config);
        let ui = &*(user_data as *const Ui);

        ui.measure_text(text, &text_config).into()
    }

    fn measure_text(&self, text: &str, config: &Text) -> Dimensions {
        let state = unsafe { &mut *self.state.get() };
        // TODO: Proper error handling
        let size = state
            .text_generator
            .measure_text_size(text, state.active_font, config.font_size as _)
            .unwrap();

        Dimensions::new(size.0 as _, size.1 as _)
    }

    pub fn set_font(&self, font_id: FontHandle) {
        let state = unsafe { &mut *self.state.get() };
        state.active_font = font_id;
    }

    pub fn begin(&mut self, _delta_time: f32, width: usize, height: usize) {
        let state = unsafe { &mut *self.state.get() };
        state
            .layout
            .layout_dimensions(Dimensions::new(width as f32, height as f32));
        state.layout.begin();
        state.io_handler.update();
        state.primitives.rewind();
        state.button_id = 0;
        state.screen_size = (width, height);
    }

    pub fn with_layout<F: FnOnce(&Ui), const N: usize>(
        &self,
        id: Option<&'a str>,
        configs: [TypedConfig; N],
        f: F,
    ) {
        let state = unsafe { &mut *self.state.get() };

        state.layout.with(id, configs, |_clay| {
            f(self);
        });
    }

    pub fn button_with_layout<const N: usize>(
        &self,
        name: &str,
        configs: [TypedConfig; N],
    ) -> Signal {
        let state = unsafe { &mut *self.state.get() };
        let mut signal = Signal::new();

        state.layout.with(Some(name), configs, |_clay| {
            signal = self.button_test(name);
        });

        signal
    }

    pub fn image(&self, handle: ImageHandle) {
        let state = unsafe { &mut *self.state.get() };

        if let Some(image) = state.io_handler.get_loaded_as::<ImageInfo>(handle) {
            let source_dimensions = Dimensions::new(image.width as _, image.height as _);

            state.layout.with(
                Some("image_test"),
                [
                    Layout::new()
                        .width(fixed!(source_dimensions.width as _))
                        .height(fixed!(source_dimensions.height as _))
                        .padding(Padding::all(30))
                        .end(),
                    ClayImage {
                        data: image.data.as_ptr() as _,
                        source_dimensions,
                    }
                    .end(),
                ],
                |_ui| {},
            );
        }
    }

    pub fn text_with_layout<const N: usize>(&self, text: &str, font_size: u32, color: ClayColor, configs: [TypedConfig; N]) {
        let state = unsafe { &mut *self.state.get() };
        state.layout.with(Some(text), configs, |_clay| {
            let font_id = state.active_font;

            let _ = state.text_generator.queue_generate_text(text, font_size, font_id, &state.bg_worker);

            state.layout.text(text, Text::new()
                .font_id(font_id as u16)
                .font_size(font_size as _)
                .color(color)
                .end());
        });
    }

    fn bounding_box(render_command: &ClayRenderCommand) -> [f32; 4] {
        let bb = render_command.bounding_box;
        [bb.x, bb.y, bb.x + bb.width, bb.y + bb.height]
    }

    fn color(color: ClayColor) -> flowi_renderer::Color {
        flowi_renderer::Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }

    fn translate_clay_render_commands(
        state: &State,
        commands: impl Iterator<Item = ClayRenderCommand<'a>>,
    ) -> Vec<RenderCommand> {
        // TODO: Arena
        let mut primitives = Vec::with_capacity(1024);

        if let Some(bg_image) = state.background_image.as_ref() {
            if let Some(image) = state.io_handler.get_loaded_as::<ImageInfo>(bg_image.handle) {
                let width = state.screen_size.0 as f32;
                let height = state.screen_size.1 as f32;

                let x0 = width - image.width as f32;
                let y0 = 0.0;
                let x1 = width;
                let y1 = image.height as f32;

                let render_command = RenderCommand {
                    bounding_box: [x0, y0, x1, y1],
                    render_type: RenderType::DrawBackground(DrawImage {
                        rounded_corners: [0.0, 0.0, 0.0, 0.0],
                        width: image.width as _,
                        height: image.height as _,
                        handle: image.data.as_ptr() as _,
                        rounding: false,
                    }),
                    color: Color::new(1.0, 1.0, 1.0, 1.0),
                };

                primitives.push(render_command);
            }
        }

        for command in commands {
            let (cmd, color) = match command.config {
                RenderCommandConfig::Rectangle(config) => match config.corner_radius {
                    CornerRadius::All(0.0) => (RenderType::DrawRect, Self::color(config.color)),
                    CornerRadius::All(radius) => (
                        RenderType::DrawRectRounded(DrawRectRoundedData {
                            corners: [radius, radius, radius, radius],
                        }),
                        Self::color(config.color),
                    ),
                    CornerRadius::Individual {
                        top_left,
                        top_right,
                        bottom_left,
                        bottom_right,
                    } => (
                        RenderType::DrawRectRounded(DrawRectRoundedData {
                            corners: [top_left, top_right, bottom_left, bottom_right],
                        }),
                        Self::color(config.color),
                    ),
                },

                RenderCommandConfig::Text(text, config) => {
                    let text = StringSlice::new(text);

                    let gen = if let Some(text_data) = state.text_generator.get_text(
                        text.as_str(),
                        config.font_size as _,
                        config.font_id as _,
                    ) {
                        DrawTextBufferData {
                            data: text_data.data,
                            handle: text_data.id,
                            width: text_data.width as _,
                            height: text_data.height as _,
                        }
                    } else {
                        DrawTextBufferData::default()
                    };

                    (RenderType::DrawTextBuffer(gen), Self::color(config.color))
                }

                RenderCommandConfig::Image(image) => (
                    RenderType::DrawImage(DrawImage {
                        rounded_corners: [0.0, 0.0, 0.0, 0.0],
                        width: image.source_dimensions.width as _,
                        height: image.source_dimensions.height as _,
                        handle: image.data as _,
                        rounding: false,
                    }),
                    Color::new(1.0, 1.0, 1.0, 1.0),
                ),

                RenderCommandConfig::Border(border) => {
                    let outer_radius = match border.corner_radius {
                        CornerRadius::All(radius) => [radius, radius, radius, radius],
                        CornerRadius::Individual {
                            top_left,
                            top_right,
                            bottom_left,
                            bottom_right,
                        } => [top_left, top_right, bottom_left, bottom_right],
                    };

                    let inner_radius = [
                        outer_radius[0] - border.left.width as f32,
                        outer_radius[1] - border.right.width as f32,
                        outer_radius[2] - border.top.width as f32,
                        outer_radius[3] - border.bottom.width as f32,
                    ];

                    (
                        RenderType::DrawBorder(DrawBorderData {
                            outer_radius,
                            inner_radius,
                        }),
                        Self::color(border.left.color),
                    )
                }

                RenderCommandConfig::ScissorStart() => {
                    (RenderType::ScissorStart, Color::new(1.0, 1.0, 1.0, 1.0))
                }

                RenderCommandConfig::ScissorEnd() => {
                    (RenderType::ScissorEnd, Color::new(1.0, 1.0, 1.0, 1.0))
                }

                RenderCommandConfig::Custom(_) => {
                    (RenderType::Custom, Color::new(1.0, 1.0, 1.0, 1.0))
                }
                _ => (RenderType::None, Color::new(1.0, 1.0, 1.0, 1.0)),
            };

            let cmd = RenderCommand {
                bounding_box: Self::bounding_box(&command),
                render_type: cmd,
                color,
            };

            primitives.push(cmd);
        }

        primitives
    }

    pub fn end(&mut self) {
        let state = unsafe { &mut *self.state.get() };

        // TODO: Fix me
        let primitives = Self::translate_clay_render_commands(state, state.layout.end());
        state.renderer.render(&primitives);

        // Generate primitives from all boxes
        //state.generate_primitives();
        state.current_frame += 1;

        state.input.mouse_pos_prev = state.input.mouse_pos;
    }

    pub fn input(&self) -> &mut Input {
        let state = unsafe { &mut *self.state.get() };
        &mut state.input
    }

    pub fn load_font(&mut self, path: &str) -> InternalResult<FontHandle> {
        let state = unsafe { &mut *self.state.get() };
        state.text_generator.load_font(path, &state.bg_worker)
    }

    pub fn load_image(&mut self, path: &str) -> InternalResult<IoHandle> {
        let state = unsafe { &mut *self.state.get() };
        Ok(crate::image_api::load(state, path))
    }

    pub fn load_background_image(&mut self, path: &str, target_size: (u32, u32)) -> InternalResult<IoHandle> {
        let state = unsafe { &mut *self.state.get() };
        Ok(crate::image_api::load_background(state, path, target_size))
    }

    pub fn set_background_image(&mut self, handle: IoHandle, mode: BackgroundMode) {
        let state = unsafe { &mut *self.state.get() };
        state.background_image = Some(BackgroundImage {
            handle,
            mode,
        });
    }

    pub fn queue_generate_text(
        &mut self,
        text: &str,
        font_size: u32,
        font_id: FontHandle,
    ) -> Option<font::CachedString> {
        let state = unsafe { &mut *self.state.get() };
        state
            .text_generator
            .queue_generate_text(text, font_size, font_id, &state.bg_worker)
    }

    #[rustfmt::skip]
    pub fn button(&self, text: &str) -> Signal {
        let state = unsafe { &mut *self.state.get() };
        let id_name = text;
        let mut signal = Signal::new();

        // TODO: Cache
        let text_size = state.text_generator.measure_text_size(text, state.active_font, 36).unwrap();

        state.layout.with(Some(id_name), [
            Layout::new()
                .width(fixed!(text_size.0 as f32 + 16.0))
                .child_alignment(Alignment::new(LayoutAlignmentX::Center, LayoutAlignmentY::Center))
                .padding(Padding::all(0)).end(),
             Rectangle::new()
                .color(ClayColor::rgba(204.0, 40.0, 40.0, 255.0))
                .corner_radius(CornerRadius::All(16.0))
                .end()], |_ui|
            {
                let font_id = state.active_font;
                // TODO: Fix me
                let _ = state.text_generator.queue_generate_text(text, 36, font_id, &state.bg_worker);

                state.layout.text(text, Text::new()
                    .font_id(font_id as u16)
                    .font_size(36)
                    .color(ClayColor::rgba(225.0, 225.0, 225.0, 255.0))
                    .end());

                let id = clay_layout::id::Id::new(id_name);

                if let Some(aabb) = state.layout.bounding_box(id) {
                    let item = state.item_states.entry(id.id.id).or_insert(ItemState {
                        aabb: Vec4::new(aabb.x, aabb.y, aabb.x + aabb.width, aabb.y + aabb.height),
                        ..Default::default()
                    });
                    item.aabb = Vec4::new(aabb.x, aabb.y, aabb.x + aabb.width, aabb.y + aabb.height);
                    signal = self.signal(item)
                }
            },
        );

        state.button_id += 1;
        signal
    }

    #[rustfmt::skip]
    pub fn button_test(&self, text: &str) -> Signal {
        let state = unsafe { &mut *self.state.get() };
        let id_name = text;
        let mut signal = Signal::new();

        let font_id = state.active_font;
        let _ = state.text_generator.queue_generate_text(text, 36, font_id, &state.bg_worker);

        state.layout.text(text, Text::new()
            .font_id(font_id as u16)
            .font_size(36)
            .color(ClayColor::rgba(255.0, 255.0, 255.0, 205.0))
            .end());

        let id = clay_layout::id::Id::new(id_name);

        if let Some(aabb) = state.layout.bounding_box(id) {
            let item = state.item_states.entry(id.id.id).or_insert(ItemState {
                aabb: Vec4::new(aabb.x, aabb.y, aabb.x + aabb.width, aabb.y + aabb.height),
                ..Default::default()
            });
            item.aabb = Vec4::new(aabb.x, aabb.y, aabb.x + aabb.width, aabb.y + aabb.height);
            signal = self.signal(item)
        }

        state.button_id += 1;
        signal
    }

    #[allow(dead_code)]
    fn signal(&self, item_state: &mut ItemState) -> Signal {
        let state = unsafe { &mut *self.state.get() };

        let mut signal = Signal::new();

        fn contains(aabb: Vec4, pos: glam::Vec2) -> bool {
            pos.x >= aabb.x && pos.y >= aabb.y && pos.x < aabb.z && pos.y < aabb.w
        }

        let is_hovered = if contains(item_state.aabb, state.input.mouse_pos) {
            signal.flags.insert(signal::SignalFlags::HOVERING);
            true
        } else {
            false
        };

        if is_hovered && !item_state.was_hovered {
            signal.flags.insert(signal::SignalFlags::ENTER_HOVER);
            item_state.was_hovered = true;
        }

        if !is_hovered && item_state.was_hovered {
            signal.flags.insert(signal::SignalFlags::EXIT_HOVER);
            item_state.was_hovered = false;
        }

        /*
        let mut signal = Signal::new();
        let box_area = box_area.as_mut_unsafe();

        dbg!(&box_area.rect);

        if box_area.rect.contains(self.input.mouse_position) {
            signal.flags.insert(signal::SignalFlags::HOVERING);
        }
        */

        signal
    }

    pub fn renderer(&mut self) -> &Box<dyn Renderer> {
        let state = unsafe { &mut *self.state.get() };
        &state.renderer
    }

    pub fn update(&mut self) {
        let state = unsafe { &mut *self.state.get() };
        state.text_generator.update();
    }

    pub fn get_text(&self, text: &str, size: u32, handle: FontHandle) -> Option<&CachedString> {
        let state = unsafe { &mut *self.state.get() };
        state.text_generator.get_text(text, size, handle)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ApplicationSettings {
    pub width: usize,
    pub height: usize,
}
