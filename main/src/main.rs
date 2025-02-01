//use flowi::font::Font;
use flowi::Application;
use flowi::{Ui, FontHandle};
use flowi::{grow, Layout, LayoutDirection};
use crate::left_side_menu::LeftSideMenu;

mod config_loader;
mod left_side_menu;
mod systems_view;

//use left_side_menu::LeftSideMenu;
//use systems_view::SystemView;

#[allow(dead_code)]
pub(crate) enum State {
    Navigating,
    Hidden,
}

pub struct Fonts {
    pub _default: FontHandle,
}

#[allow(dead_code)]
pub(crate) struct App {
    left_side_menu: LeftSideMenu,
    fonts: Fonts,
    width: usize,
    height: usize,
}

#[rustfmt::skip]
fn main_loop(ui: &Ui, app: &mut App) {
    ui.with_layout(Some("main_view"), [
        Layout::new()
            .width(grow!())
            .height(grow!())
            .direction(LayoutDirection::LeftToRight)
            .end()], |ui| 
    {
        app.left_side_menu.update(ui);
    });
}

fn main() {
    let width = 1280;
    let height = 720;

    let settings = flowi::ApplicationSettings { width, height };

    let mut flowi_app = Application::new(&settings); //.unwrap();

    let fonts = Fonts {
        _default: flowi_app.ui.load_font("../data/fonts/roboto/Roboto-Regular.ttf", 36).unwrap(),
    };

    let app = Box::new(App {
        left_side_menu: LeftSideMenu::new(&flowi_app.ui),
        fonts,
        width,
        height,
    });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}
