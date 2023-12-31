use flowi::{
    image::Image,
    math_data::Vec2,
    Color,
    window::{Window, WindowFlags},
    ui::Ui,
    font::Font,
    text::Text,
    layout::Cursor,
};

use crate::Fonts;

static TEST_AMIGA_TEXT: &str = "The Amiga is a family of personal computers marketed by Commodore in the 1980s and 1990s. The first model was launched in 1985 as a high-end home computer and became popular for its graphical, audio and multi-tasking abilities. The Amiga provided a significant upgrade from 8-bit computers, such as the Commodore 64, and the platform quickly grew in popularity among computer enthusiasts. The best selling model, the Amiga 500, was introduced in 1987 and became the leading home computer of the late 1980s and early 1990s in much of Western Europe";

pub struct System {
    name: String,
    release_date: String,
    developer: String,
    manufacturer: String,
    notes: String,
    image_url: String,
    image: Image,
}

pub struct SystemView {
    systems: Vec<System>,
}

impl SystemView {
    pub fn new() -> SystemView {
        let amiga_system = System {
            name: String::from("Amiga"),
            release_date: String::from("July 23, 1985"),
            developer: String::from("Commodore International"),
            manufacturer: String::from("Commodore International"),
            notes: String::from(TEST_AMIGA_TEXT),
            image_url: String::from("data/amiga.png"),
            image: Image::load("data/amiga.png"),
        };

        SystemView {
            systems: vec![amiga_system], 
        }
    }

    pub fn update(&mut self, fonts: &Fonts, start_x: i32, width: i32, height: i32) {
        Window::set_pos(Vec2::new(start_x as _, 0.0));
        Window::set_size(Vec2::new(width as _, height as _));

        Window::begin("system_view", WindowFlags::NO_DECORATION);

        if let Ok(info) = Image::get_info(self.systems[0].image) {
            Cursor::set_pos(Vec2::new(info.width as f32 - start_x as f32, 0.0));

            let image_size = Vec2::new(info.width as f32, info.height as f32);

            let color0 = Color::new(1.0, 1.0, 1.0, 0.0);
            let color1 = Color::new(1.0, 1.0, 1.0, 0.2);
            let color2 = Color::new(1.0, 1.0, 1.0, 0.2);
            let color3 = Color::new(1.0, 1.0, 1.0, 0.0);

            Ui::image_size_color_shade(self.systems[0].image, image_size, color0, color1, color2, color3);
        } 

        Font::push(fonts.system_header);
        Cursor::set_pos(Vec2::new(40.0, 4.0));
        Text::show(&self.systems[0].name);
        Font::pop();

        Font::push(fonts.system_text);
        Cursor::set_pos(Vec2::new(40.0, 100.0));
        Text::show_wrapped(&self.systems[0].notes);
        Font::pop();

        Window::end();
    }
}

/*
struct SystemsView {
    systems: Vec<System>,
}
*/
