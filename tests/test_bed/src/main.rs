use flowi::{Application};
use flowi::Ui;
use flowi::{
    fixed, grow, percent, Alignment, ClayColor, ImageHandle, LayoutAlignmentX, LayoutAlignmentY,
    LayoutDirection, FontHandle, Padding,
    BackgroundMode, Declaration,
};
//use log::*;
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
    ui.with_layout(&Declaration::new()
        .id(ui.id("entry_info"))
        .layout()
            .width(grow!())
            .height(percent!(0.5))
            .direction(LayoutDirection::TopToBottom)
            .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
            .child_gap(10)
         .end(), |ui|
            //.background_color(ClayColor::rgba(0.0, 110.0, 0.0, 255.0)), |ui|
    {
        ui.with_layout(&Declaration::new()
            .id(ui.id("tile_info"))
            .layout()
                .width(grow!())
                .height(fixed!(80.0))
                .child_gap(40)
                .direction(LayoutDirection::LeftToRight)
            .end(), |ui|
            //.background_color(ClayColor::rgba(150.0, 0.0, 0.0, 255.0)), |ui|
        {
            ui.set_font(app.fonts.thin);

            let text_size = ui.text_size(&entry.title, 78);

            ui.text_with_layout(&entry.title, 78,
                ClayColor::rgba(255.0, 255.0, 255.0, 255.0),
                &Declaration::new()
                    .layout()
                        .width(fixed!(text_size.width))
                        .height(fixed!(text_size.height))
                        .padding(Padding::horizontal(32))
                        .end());

            ui.text_with_layout("1992", 78,
                ClayColor::rgba(128.0, 128.0, 128.0, 255.0),
                &Declaration::new()
                    .layout()
                        .width(grow!())
                        .end());
        });

        ui.with_layout(&Declaration::new()
            .id(ui.id("platfrom_info"))
            .layout()
                .width(grow!())
                .height(fixed!(40.0))
                .padding(Padding::horizontal(32))
                .child_gap(16)
                .direction(LayoutDirection::LeftToRight)
            .end(), |ui|
        {
            ui.set_font(app.fonts.default);

            ui.button("DEMO");
            ui.button("AMIGA OCS/ECS");

            ui.text_with_layout("by", 36,
                ClayColor::rgba(255.0, 255.0, 255.0, 255.0),
                &Declaration::new()
                    .layout()
                        .width(fixed!(44.0))
                        .end());

            ui.set_font(app.fonts.bold);

            ui.text_with_layout("Spaceballs", 36,
                ClayColor::rgba(201.0, 22.0, 38.0, 255.0),
                &Declaration::new()
                    .layout()
                        .width(grow!())
                        .end());

        });

        /*
        ui.text(entry.authors.join(", "));
        ui.text(entry.release_date);
        ui.text(entry.platforms.join(", "));
        ui.text(entry.tags.join(", "));
        */
    });
}

#[allow(dead_code)]
#[rustfmt::skip]
fn draw_selection_entry(ui: &Ui, _app: &App, index: usize, is_selected: bool) {
    let size = if is_selected {
        (300.0, 300.0 * 1.4)
    } else {
        (250.0, 250.0 * 1.4)
    };

    ui.with_layout(&Declaration::new()
        .id(ui.id_index("demo_selection", index as _))
        .layout()
            .width(fixed!(size.0))
            .height(fixed!(size.1))
        .end()
        .corner_radius().all(16.0).end()
        .background_color(ClayColor::rgba(0.0, 0.0, 255.0, 255.0)), |_ui|
       {

       });
}

#[allow(dead_code)]
fn draw_image_grid_unlimited_scroll(ui: &Ui, _app: &App) {
    ui.with_layout(&Declaration::new()
        .id(ui.id("selection_grid"))
        .layout()
            .width(grow!())
            .height(percent!(0.5))
            .direction(LayoutDirection::LeftToRight)
            .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
            .child_gap(64)
            .padding(Padding::horizontal(64))
        .end(), |ui|
    {
        for i in 0..6 {
            draw_selection_entry(ui, _app, i, i == 0);
        }
    });
}

#[rustfmt::skip]
fn main_loop(ui: &Ui, _app: &mut App) {
    ui.with_layout(&Declaration::new()
        .layout()
            .width(grow!())
            .height(grow!())
            .direction(LayoutDirection::TopToBottom)
        .end(), |ui|
    {
        display_demo_entry(ui, &_app, &_app.navigantion_entries[0]);
        draw_image_grid_unlimited_scroll(ui, &_app);
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
