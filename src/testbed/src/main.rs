//! Example of how to use Egui

#![deny(warnings)]
#![warn(clippy::all)]

use egui::{Window};
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

        // Example used in `README.md`.
        Window::new("").show(ui.ctx(), |ui| {
            ui.button("Collection");
            ui.button("Cores");
            ui.button("Demos");
            ui.button("Settings");
        });
    }

    fn on_exit(&mut self, storage: &mut dyn egui::app::Storage) {
        egui::app::set_value(storage, egui::app::APP_KEY, self);
    }
}

fn main() {
    let title = "My Egui Window";
    let storage = FileStorage::from_path(".egui_example_glium.json".into()); // Where to persist app state
    let app: MyApp = egui::app::get_value(&storage, egui::app::APP_KEY).unwrap_or_default(); // Restore `MyApp` from file, or create new `MyApp`.
    egui_glium::run(title, RunMode::Reactive, storage, app);
}

/*
fn my_save_function() {
    // dummy
}
*/