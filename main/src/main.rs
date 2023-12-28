use flowi::font::Font;
use flowi::Application;

mod left_side_menu;

use left_side_menu::LeftSideMenu;

pub(crate) enum State {
    Navigating,
    Hidden,
}

pub(crate) struct App {
    left_side_menu: LeftSideMenu,
    state: State,
    montserrat_font: Font,
    first_frame: bool,
    width: u32,
    height: u32,
}

fn main_loop(app: &mut App) {
    Font::push(app.montserrat_font);

    app.left_side_menu.update(app.width, app.height);

    Font::pop();
}

fn main() {
    let width = 1920;
    let height = 1080;

    let settings = flowi::ApplicationSettings { width, height };

    let mut flowi_app = Application::new(&settings).unwrap();

    let app = Box::new(App {
        state: State::Navigating,
        left_side_menu: LeftSideMenu::new(width, height),
        montserrat_font: Font::load("data/fonts/montserrat/Montserrat-Regular.ttf", 56).unwrap(),
        first_frame: false,
        width,
        height,
    });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}
