use flowi::Application;
use flowi::Ui;
use flowi::{
    fixed, grow, Alignment, ClayColor, ImageHandle, Layout, LayoutAlignmentX, LayoutAlignmentY,
    LayoutDirection, Padding, Rectangle,
};
use log::*;

/*
pub struct Fonts {
    pub default: Font,
    pub system_header: Font,
    pub system_text: Font,
    pub rot_header: Font,
}
*/

#[allow(dead_code)]
pub(crate) struct App {
    width: usize,
    height: usize,
    image: ImageHandle,
}

struct NavigationEntry {
    title: String,
    authors: Vec<String>,
    relase_date: String,
    platforms: Vec<String>,
    image: ImageHandle,
}

fn draw_image_grid_unlimited_scroll(ui: &Ui, app: &App) {}

#[rustfmt::skip]
fn main_loop(ui: &Ui, _app: &mut App) {
    ui.with_layout(Some("main_container"), [
        Layout::new()
            .width(grow!())
            .height(grow!())
            .direction(LayoutDirection::LeftToRight)
            .end()], |ui|
   {
        ui.with_layout(Some("header"), [
            Layout::new()
                .height(grow!())
                .width(fixed!(220.0))
                .padding(Padding::all(8))
                .child_gap(16)
                .direction(LayoutDirection::TopToBottom)
                .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
                .end(),
            Rectangle::new()
                .color(ClayColor::rgba(100.0, 100.0, 100.0, 255.0))
                .end()], |ui| 
        {
            if ui.button("Foo").hovering() {
                //println!("Hovering over Foo");
            }

            if ui.button("Bar").hovering() {
                //println!("Hovering over Bar");
            }

            if ui.button("Settings").hovering() {
                //println!("Hovering over Settings");
            }

            //ui.button("Test");
        });

        ui.with_layout(Some("main"), [
            Layout::new()
                .height(grow!())
                .width(grow!())
                .child_gap(16)
                .direction(LayoutDirection::TopToBottom)
                .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
                .end(),
            Rectangle::new()
                .color(ClayColor::rgba(200.0, 200.0, 100.0, 255.0))
                .end()], |ui| 
        {
            ui.with_layout(Some("main2"), [
                Layout::new()
                    .height(grow!())
                    .width(grow!())
                    .child_gap(16)
                    .direction(LayoutDirection::LeftToRight)
                    .end(),
                Rectangle::new()
                    .color(ClayColor::rgba(20.0, 20.0, 10.0, 255.0))
                    .end()], |_ui| 
            {
                ui.image(_app.image);

                /*
                if ui.button("Foo2").hovering() {
                    //println!("Hovering over Foo");
                }
                */
            });
        });
    });
}

fn main() {
    let width = 1280;
    let height = 720;

    let _ = env_logger::builder()
        .filter_level(LevelFilter::max())
        .init();

    let settings = flowi::ApplicationSettings { width, height };

    let mut flowi_app = Application::new(&settings); //.unwrap();

    let _ = flowi_app
        .ui
        .load_font("../../data/fonts/roboto/Roboto-Regular.ttf", 36);
    //let image = flowi_app.ui.load_image("/Users/emoon/code/projects/replay_frontend/data/amiga.png").unwrap();
    let image = flowi_app
        .ui
        .load_image("/home/emoon/code/projects/replay_frontend/data/amiga.png")
        .unwrap();

    /*
    let fonts = Fonts {
        default: Font::load("data/fonts/montserrat/Montserrat-Regular.ttf", 56).unwrap(),
        system_header: Font::load("data/fonts/roboto/Roboto-Bold.ttf", 72).unwrap(),
        system_text: Font::load("data/fonts/roboto/Roboto-Regular.ttf", 48).unwrap(),
        rot_header: Font::load("data/fonts/roboto/Roboto-Bold.ttf", 56).unwrap(),
    };
    */

    let app = Box::new(App {
        width,
        height,
        image,
    });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}
