use flowi::{Application};
use flowi::Ui;
use flowi::{
    fixed, grow, Alignment, ClayColor, ImageHandle, Layout, LayoutAlignmentX, LayoutAlignmentY,
    LayoutDirection, Padding, Rectangle, FontHandle,
    BackgroundMode,
};
use log::*;
use demozoo_fetcher::ProductionEntry;

pub struct Fonts {
    pub default: FontHandle,
    pub thin: FontHandle,
    pub bold: FontHandle,
    pub light: FontHandle,
}

#[allow(dead_code)]
pub(crate) struct App {
    width: usize,
    height: usize,
    image: ImageHandle,
    navigantion_entries: Vec<ProductionEntry>,
    fonts: Fonts,
}


#[rustfmt::skip]
fn display_demo_entry(ui: &Ui, app: &App, entry: &ProductionEntry) {
    ui.with_layout(Some("entry_info"), [
        Layout::new()
            .width(grow!())
            .height(grow!())
            .child_gap(100)
            .direction(LayoutDirection::TopToBottom)
            .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Top))
            .end()], |ui| 
    {
        ui.with_layout(Some("entry_title"), [
            Layout::new()
                .width(grow!())
                .height(fixed!(40.0))
                .child_gap(0)
                .direction(LayoutDirection::LeftToRight)
                //.child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
                .end()], |ui| 
        {
            ui.set_font(app.fonts.thin);

            ui.text_with_layout(&entry.title,
                78,
                ClayColor::rgba(255.0, 255.0, 255.0, 255.0),
                [Layout::new()
                    .width(fixed!(680.0))
                    .padding(Padding::all(40))
                    .end()]);
            
            ui.text_with_layout("1992",
                78,
                ClayColor::rgba(128.0, 128.0, 128.0, 255.0),
                [Layout::new()
                    .width(grow!())
                    .padding(Padding::all(40))
                    .end()]);
        });
        
        ui.with_layout(Some("platform_info"), [
            Layout::new()
                .width(grow!())
                .height(fixed!(40.0))
                .padding(Padding::all(0))
                .child_gap(16)
                .direction(LayoutDirection::LeftToRight)
                //.child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
                .end()], |ui| 
        {
            ui.set_font(app.fonts.default);

            ui.button("DEMO");
            ui.button("AMIGA OCS/ECS");
            //ui.button(&entry.platforms[0].name);
            
            /*
            ui.text_with_layout("Demo",
                78,
                ClayColor::rgba(128.0, 128.0, 128.0, 255.0),
                [Layout::new()
                    .width(grow!())
                    .padding(Padding::all(40))
                    .height(fixed!(140.0))
                    .end()]);
            */
        });

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
            .direction(LayoutDirection::TopToBottom)
            .end()], |ui|
   {
        ui.with_layout(Some("main"), [
            Layout::new()
                .height(grow!())
                .width(grow!())
                .child_gap(2)
                .direction(LayoutDirection::TopToBottom)
                //.child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
                .end(),
            Rectangle::new()
                .color(ClayColor::rgba(0.0, 0.0, 0.0, 255.0))
                .end()], |ui| 
        {
            display_demo_entry(ui, &_app, &_app.navigantion_entries[0]);
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

    /*
    let _ = env_logger::builder()
        .filter_level(LevelFilter::max())
        .init();
    */

    // This is obviously temporary but will do for now 
    let navigantion_entries = vec![
        demozoo_fetcher::get_demo_entry_by_file("data/2.json"),
    ];

    let settings = flowi::ApplicationSettings { width, height };

    let mut flowi_app = Application::new(&settings); //.unwrap();
    let ui = &mut flowi_app.ui;

    //let image = flowi_app.ui.load_image("/Users/emoon/code/projects/replay_frontend/data/amiga.png").unwrap();
    let image = ui
        .load_background_image("data/test_data/image_cache/b9519e5917ab222fa311e1b642d03f227ce51cfb11f42e87e1f74f2bd23f2e90.png", (width as _, height as _))
        .unwrap();

    ui.set_background_image(image, BackgroundMode::AlignTopRight);

    let fonts = Fonts {
        bold: ui.load_font("data/fonts/roboto/Roboto-Bold.ttf").unwrap(),
        default: ui.load_font("data/fonts/roboto/Roboto-Regular.ttf").unwrap(),
        thin: ui.load_font("data/fonts/roboto/Roboto-Thin.ttf").unwrap(),
        light: ui.load_font("data/fonts/roboto/Roboto-Light.ttf").unwrap(),
    };

    let app = Box::new(App {
        width,
        height,
        image,
        navigantion_entries,
        fonts,
    });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}
