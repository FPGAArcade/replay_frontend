//! Example of how to use Egui

//#![deny(warnings)]
#![warn(clippy::all)]

use egui::*;
use egui_glium::{storage::FileStorage, RunMode};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
struct MyApp {
    //my_string: String,
    value: f32,
}

impl egui::app::App for MyApp {
    /// This function will be called whenever the Ui needs to be shown,
    /// which may be many times per second.
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

fn main() {
    let title = "Replay Arcade Frontend";
    let storage = FileStorage::from_path(".egui_example_glium.json".into()); // Where to persist app state
    let app: MyApp = egui::app::get_value(&storage, egui::app::APP_KEY).unwrap_or_default(); // Restore `MyApp` from file, or create new `MyApp`.
    egui_glium::run(title, RunMode::Reactive, storage, app);
}

/*
fn my_save_function() {
    // dummy
}
*/
