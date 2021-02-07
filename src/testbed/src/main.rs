use minifb::{Key, Window, WindowOptions};

/*
use egui::*;
use egui_glium::{storage::FileStorage, RunMode};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
struct MyApp {
    //my_string: String,
    value: f32,
}

impl egui::app::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut dyn egui::app::Backend) {
        //let MyApp { value } = self;

        Area::new(Id::new("Side Panel"))
            .order(Order::Foreground)
            .fixed_pos(pos2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                Frame::window(ui.style()).show(ui, |ui| {
                    ui.expand_to_size(vec2(200.0, ui.input().screen_size.y));
                    ui.allocate_space(vec2(200.0, 200.0));

                    ui.button("Cores");
                    ui.button("Games");
                    ui.button("Demos");
                    ui.button("Music");
                    ui.button("Settings");
                })
            });
    }

    fn on_exit(&mut self, storage: &mut dyn egui::app::Storage) {
        egui::app::set_value(storage, egui::app::APP_KEY, self);
    }
}
*/

fn main() {
    const WIDTH: usize = 1280;
    const HEIGHT: usize = 720;

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
