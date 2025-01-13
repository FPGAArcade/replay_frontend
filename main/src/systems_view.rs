/*
use flowi::{
    font::Font,
    image::Image,
    layout::Cursor,
    math_data::Vec2,
    painter::Painter,
    text::Text,
    ui::Ui,
    window::{Window, WindowFlags},
    Color,
};

use crate::Fonts;

static TEST_AMIGA_TEXT: &str = "The Amiga is a family of personal computers marketed by Commodore in the 1980s and 1990s. The first model was launched in 1985 as a high-end home computer and became popular for its graphical, audio and multi-tasking abilities. The Amiga provided a significant upgrade from 8-bit computers, such as the Commodore 64, and the platform quickly grew in popularity among computer enthusiasts.";

pub struct System {
    name: String,
    _release_date: String,
    _developer: String,
    _manufacturer: String,
    notes: String,
    _image_url: String,
    image_rot_url: String,
    image: Image,
    image_rot: Image,
}

impl Default for System {
    fn default() -> Self {
        Self {
            name: String::new(),
            _release_date: String::new(),
            _developer: String::new(),
            _manufacturer: String::new(),
            notes: String::new(),
            _image_url: String::new(),
            image_rot_url: String::new(),
            image: Image { handle: 0 },
            image_rot: Image { handle: 0 },
        }
    }
}

pub struct SystemView {
    systems: Vec<System>,
    select_system: usize,
}

impl SystemView {
    pub fn new() -> SystemView {
        let amiga_system = System {
            name: String::from("Amiga"),
            _release_date: String::from("July 23, 1985"),
            _developer: String::from("Commodore International"),
            _manufacturer: String::from("Commodore International"),
            notes: String::from(TEST_AMIGA_TEXT),
            _image_url: String::from("data/amiga.png"),
            image_rot_url: String::from("data/amiga_rot.png"),
            image: Image::load("data/amiga.png"),
            ..Default::default()
        };

        let c64_system = System {
            image_rot_url: String::from("data/c64_rot.png"),
            ..Default::default()
        };

        let nes_system = System {
            image_rot_url: String::from("data/nes_rot.png"),
            ..Default::default()
        };

        let mut systems = vec![amiga_system, c64_system, nes_system];

        for system in &mut systems {
            system.image_rot = Image::load(&system.image_rot_url);
        }

        SystemView {
            systems,
            select_system: 0,
        }
    }

    fn draw_selection(&self, fonts: &Fonts, start_x: i32, _width: i32, _height: i32) {
        Font::push(fonts.rot_header);
        Cursor::set_pos(Vec2::new(40.0, 414.0));
        Text::show("My Systems");
        Font::pop();

        let start = Vec2::new(start_x as f32 + 40.0, 500.0);
        let size = Vec2::new(300.0, 300.0);

        /*
        let p1 = start;
        let p2 = Vec2::new(p1.x + size.x, p1.y + size.y);
        let color = Color::new(1.0, 1.0, 1.0, 1.0);
        let rounding = 0.0;

        Painter::draw_rect_filled(p1, p2, color, rounding);

        Cursor::set_pos(Vec2::new(start.x - start_x as f32, start.y));
        Ui::image_size(self.systems[0].image_rot, size);
        */

        let mut pos = start;

        for (i, system) in self.systems.iter().enumerate() {
            if i == self.select_system {
                let p1 = pos;
                let p2 = Vec2::new(p1.x + size.x, p1.y + size.y);
                let color = Color::new(1.0, 1.0, 1.0, 1.0);
                let rounding = 0.0;

                Painter::draw_rect_filled(p1, p2, color, rounding);

                Cursor::set_pos(Vec2::new(pos.x - start_x as f32, pos.y));
                Ui::image_size(system.image_rot, size);
            } else {
                let new_size = Vec2::new(size.x * 0.7, size.y * 0.7);
                let p1 = Vec2::new(pos.x + size.x * 0.2, pos.y + size.y * 0.2);
                let p2 = Vec2::new(p1.x + size.x * 0.7, p1.y + size.y * 0.7);
                //let p1 = pos;
                //let p2 = Vec2::new(p1.x + size.x, p1.y + size.y);
                let color = Color::new(0.6, 0.6, 0.6, 0.6);
                let rounding = 0.0;

                Painter::draw_rect_filled(p1, p2, color, rounding);

                Cursor::set_pos(Vec2::new(p1.x - start_x as f32, p1.y));
                Ui::image_size(system.image_rot, new_size);
            }

            pos.x += size.x + 40.0;
        }
    }

    fn show_system_info(system: &System, fonts: &Fonts, start_x: i32, width: i32, _height: i32) {
        if let Ok(info) = Image::get_info(system.image) {
            let image_width = (info.width as f32) / 1.6;

            Cursor::set_pos(Vec2::new(
                start_x as f32 + (width as f32 - image_width),
                0.0,
            ));
            let image_size = Vec2::new(image_width, (info.height as f32) / 1.6);

            let color0 = Color::new(1.0, 1.0, 1.0, 0.0);
            let color1 = Color::new(1.0, 1.0, 1.0, 0.2);
            let color2 = Color::new(1.0, 1.0, 1.0, 0.2);
            let color3 = Color::new(1.0, 1.0, 1.0, 0.0);

            Ui::image_size_color_shade(system.image, image_size, color0, color1, color2, color3);
        }

        Font::push(fonts.system_header);
        Cursor::set_pos(Vec2::new(40.0, 4.0));
        Text::show(&system.name);
        Font::pop();

        Font::push(fonts.system_text);
        Cursor::set_pos(Vec2::new(40.0, 100.0));
        Text::show_wrapped(&system.notes);
        Font::pop();
    }

    pub fn update(&mut self, fonts: &Fonts, start_x: i32, width: i32, height: i32) {
        Window::set_pos(Vec2::new(start_x as _, 0.0));
        Window::set_size(Vec2::new(width as _, height as _));

        Window::begin("system_view", WindowFlags::NO_DECORATION);

        Self::show_system_info(&self.systems[0], fonts, start_x, width, height);

        self.draw_selection(fonts, start_x, width, height);

        Window::end();
    }
}
*/

