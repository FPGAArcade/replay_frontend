use flowi::Application;
use flowi::Ui;
use flowi::{
    fixed, grow, Alignment, ClayColor, ImageHandle, Layout, LayoutAlignmentX, LayoutAlignmentY,
    LayoutDirection, Padding, Rectangle,
};
use log::*;
use demozoo_fetcher::ProductionEntry;

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
    navigantion_entries: Vec<ProductionEntry>,
}


fn display_demo_entry(ui: &Ui, entry: &ProductionEntry) {
    ui.with_layout(Some("entry_info"), [
        Layout::new()
            .width(grow!())
            .height(fixed!(200.0))
            .padding(Padding::all(8))
            .child_gap(16)
            .direction(LayoutDirection::TopToBottom)
            .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
            .end()], |ui| 
    {
        ui.text_with_layout(&entry.title,
            144,
            ClayColor::rgba(255.0, 255.0, 255.0, 255.0),
            [Layout::new()
                .width(grow!())
                .height(fixed!(40.0))
                .end()]);

        /*
        ui.text(entry.authors.join(", "));
        ui.text(entry.release_date);
        ui.text(entry.platforms.join(", "));
        ui.text(entry.tags.join(", "));
        */
    });
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
        /*
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
        */

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
            display_demo_entry(ui, &_app.navigantion_entries[0]);
            /*
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
                //ui.image(_app.image);

                /*
                if ui.button("Foo2").hovering() {
                    //println!("Hovering over Foo");
                }
                */
            });
            */
        });
    });
}

fn main() {
    let width = 1920;
    let height = 1080;

    let _ = env_logger::builder()
        .filter_level(LevelFilter::max())
        .init();

    // This is obviously temporary but will do for now 
    let navigantion_entries = vec![
        demozoo_fetcher::get_demo_entry_by_file("../../crates/demozoo-fetcher/test-data/2.json"),
    ];

    let settings = flowi::ApplicationSettings { width, height };

    let mut flowi_app = Application::new(&settings); //.unwrap();

    let _ = flowi_app
        .ui
        .load_font("../../data/fonts/roboto/Roboto-Regular.ttf");
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
        navigantion_entries,
    });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}
