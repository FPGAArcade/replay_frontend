mod content_selector;
mod online_demo_selector;
mod content_provider;
mod demozoo_fetcher;

use arena_allocator;
use flowi::{Application};
use flowi::Ui;
use flowi::{
    fixed, grow, percent, Alignment, ClayColor, ImageHandle, LayoutAlignmentX, LayoutAlignmentY,
    LayoutDirection, FontHandle, Padding,
    BackgroundMode, Declaration,
    ImageInfo,
    Dimensions,
};
use image::Color16;
//use log::*;
use demozoo_fetcher::ProductionEntry;
use crate::online_demo_selector::OnlineDemoSelector;

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
    fonts: Fonts,
    online_demo_selector: OnlineDemoSelector,
}

#[rustfmt::skip]
fn main_loop(ui: &Ui, app: &mut App) {
    ui.with_layout(&Declaration::new()
        .layout()
            .width(grow!())
            .height(grow!())
            .direction(LayoutDirection::TopToBottom)
        .end(), |ui|
    {
        app.online_demo_selector.update(ui);
        //display_demo_entry(ui, &_app, &_app.demo_entries[0]);
        //draw_image_grid_unlimited_scroll(ui, _app);
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

    let settings = flowi::ApplicationSettings { width, height };

    let mut flowi_app = Application::new(&settings); //.unwrap();
    let ui = &mut flowi_app.ui;

    //ui.set_background_image(image, BackgroundMode::AlignTopRight);

    let fonts = Fonts {
        bold: ui.load_font("data/fonts/roboto/Roboto-Bold.ttf").unwrap(),
        default: ui.load_font("data/fonts/roboto/Roboto-Regular.ttf").unwrap(),
        thin: ui.load_font("data/fonts/roboto/Roboto-Thin.ttf").unwrap(),
        light: ui.load_font("data/fonts/roboto/Roboto-Light.ttf").unwrap(),
    };

    let app = Box::new(App {
        width,
        height,
        fonts,
        online_demo_selector: OnlineDemoSelector::new(),
    });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}

/*
struct DemoEntry {
    metadata: ProductionEntry,
    thumbnail_screenshots: Vec<ImageHandle>,
    preview_image: OutputImage,
}

use std::cmp::min;

struct OutputImage {
    data: Vec<Color16>,
    width: usize,
    height: usize,
}

impl OutputImage {
    fn new(width: usize, height: usize) -> Self {
        let white = Color16::new_splat(32767);
        Self {
            data: vec![white; (width + 2) * (height + 4)],
            width: width + 2,
            height: height + 2,
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: Color16) {
        if x < self.width-1 && y < self.height-1 {
            self.data[y * self.width + x] = color;
        }
    }

    fn blit_image(&mut self, img: &ImageInfo, x_offset: usize, y_offset: usize) {

        for y in 0..img.height as usize {
            for x in 0..img.width as usize {
                let target_x = x_offset + x;
                let target_y = y_offset + y;

                if target_x < self.width-1 && target_y < self.height-1 {
                    let color = img.data[y * img.width as usize + x];
                    self.set_pixel(target_x + 1, target_y + 1, color);
                }
            }
        }
    }
}

fn merge_images(output_width: usize, output_height: usize, img1: &ImageInfo, img2: &ImageInfo, img3: &ImageInfo, img4: &ImageInfo) -> OutputImage {
    let mut output = OutputImage::new(output_width, output_height);

    let half_width = output_width / 2;
    let half_height = output_height / 2;

    // Compute placements (centered as best as possible)
    let x1 = (half_width - min(img1.width as usize, half_width)) / 2;
    let y1 = (half_height - min(img1.height as _, half_height)) / 2;

    let x2 = half_width + (half_width - min(img2.width as _, half_width)) / 2;
    let y2 = (half_height - min(img2.height as _, half_height)) / 2;

    let x3 = (half_width - min(img3.width as _, half_width)) / 2;
    let y3 = half_height + (half_height - min(img3.height as _, half_height)) / 2;

    let x4 = half_width + (half_width - min(img4.width as _, half_width)) / 2;
    let y4 = half_height + (half_height - min(img4.height as _, half_height)) / 2;

    // Blit images
    output.blit_image(img1, x1, y1);
    output.blit_image(img2, x2, y2);
    output.blit_image(img3, x3, y3);
    output.blit_image(img4, x4, y4);

    output
}


impl DemoEntry {
    fn new(metadata: ProductionEntry) -> Self {
        Self {
            metadata,
            thumbnail_screenshots: Vec::new(),
            preview_image: OutputImage {
                data: Vec::new(),
                width: 0,
                height: 0,
            },
        }
    }
}


#[rustfmt::skip]
fn display_demo_entry(ui: &Ui, app: &App, entry: &DemoEntry) {
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

            let text_size = ui.text_size(&entry.metadata.title, 78);

            ui.text_with_layout(&entry.metadata.title, 78,
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
            .id(ui.id("platform_info"))
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

            ui.text_with_layout(&entry.metadata.author_nicks[0].name, 36,
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
fn draw_selection_entry(ui: &Ui, _app: &mut App, index: usize, is_selected: bool) {
    let size = if is_selected {
        (300.0, 300.0)
    } else {
        (250.0, 250.0)
    };

    let entry = &mut _app.demo_entries[index];

    // Generate preview image. This should be on a separate thread later
    if entry.preview_image.data.is_empty() {
        let mut has_all_images = true;
        for img in &entry.thumbnail_screenshots {
            if ui.get_image(*img).is_none() {
                has_all_images = false;
                break;
            }
        }

        /*
        if has_all_images {
            let img1 = ui.get_image(entry.thumbnail_screenshots[0]).unwrap();
            let img2 = ui.get_image(entry.thumbnail_screenshots[1]).unwrap();
            let img3 = ui.get_image(entry.thumbnail_screenshots[2]).unwrap();
            let img4 = ui.get_image(entry.thumbnail_screenshots[3]).unwrap();
            entry.preview_image = merge_images(size.0 as _, size.1 as _, &img1, &img2, &img3, &img4);

            println!("Generated preview image for entry {}", index);
        }
         */
    }

    if !entry.preview_image.data.is_empty() {
        let source_dimensions = Dimensions::new(
            entry.preview_image.width as _,
            entry.preview_image.height as _);

        unsafe {
            ui.with_layout(&Declaration::new()
                .id(ui.id_index("demo_selection", index as _))
                .layout()
                    .width(fixed!(entry.preview_image.width  as _))
                    .height(fixed!(entry.preview_image.height as _))
                .end()
                .corner_radius().all(16.0).end()
                .image()
                    .data_ptr(entry.preview_image.data.as_ptr() as _)
                    .source_dimensions(source_dimensions)
                .end()
                .background_color(ClayColor::rgba(0.0, 0.0, 255.0, 255.0)), |_ui|
            {

            });
        }
    } else {
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
}

#[allow(dead_code)]
fn draw_image_grid_unlimited_scroll(ui: &Ui, _app: &mut App) {
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
        display_demo_entry(ui, &_app, &_app.demo_entries[0]);
        draw_image_grid_unlimited_scroll(ui, _app);
    });
}
*/

/*
// This is obviously temporary but will do for now
let mut demo_entries = vec![
    DemoEntry::new(demozoo_fetcher::get_demo_entry_by_file("data/2.json")),
    DemoEntry::new(demozoo_fetcher::get_demo_entry_by_file("data/5312.json")),
    DemoEntry::new(demozoo_fetcher::get_demo_entry_by_file("data/5313.json")),
    DemoEntry::new(demozoo_fetcher::get_demo_entry_by_file("data/5314.json")),
    DemoEntry::new(demozoo_fetcher::get_demo_entry_by_file("data/5315.json")),
    DemoEntry::new(demozoo_fetcher::get_demo_entry_by_file("data/5316.json")),
];

// TODO: This should be done one-demand

/*
for entry in demo_entries.iter_mut() {
    for screenshot in entry.metadata.screenshots.iter().take(1) {
        println!("loading {:?}", &screenshot.thumbnail_url);
        let local_path = demozoo_fetcher::get_image(&screenshot.thumbnail_url).unwrap();
        let image = ui.load_image(&local_path).unwrap();
        entry.thumbnail_screenshots.push(image);
    }
}
    :
 */

//let image = flowi_app.ui.load_image("/Users/emoon/code/projects/replay_frontend/data/amiga.png").unwrap();
let image = ui
.load_background_image("data/test_data/image_cache/b9519e5917ab222fa311e1b642d03f227ce51cfb11f42e87e1f74f2bd23f2e90.png", (width as _, height as _))
.unwrap();

 */

