//use flowi::font::Font;
use flowi::Application;

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
    /*
    left_side_menu: LeftSideMenu,
    system_view: SystemView,
    state: State,
    fonts: Fonts,
    */
    width: usize,
    height: usize,
}

fn main_loop(_app: &mut App) {
    /*
    if !app.left_side_menu.update(&app.fonts, app.width, app.height) {
        return;
    }

    app.system_view.update(
        &app.fonts,
        app.left_side_menu.width,
        app.width - app.left_side_menu.width,
        app.height,
    );
    */
}

fn main() {
    let width = 1920;
    let height = 1080;

    let settings = flowi::ApplicationSettings { width, height };

    let mut flowi_app = Application::new(&settings); //.unwrap();

    /*
    let fonts = Fonts {
        default: Font::load("data/fonts/montserrat/Montserrat-Regular.ttf", 56).unwrap(),
        system_header: Font::load("data/fonts/roboto/Roboto-Bold.ttf", 72).unwrap(),
        system_text: Font::load("data/fonts/roboto/Roboto-Regular.ttf", 48).unwrap(),
        rot_header: Font::load("data/fonts/roboto/Roboto-Bold.ttf", 56).unwrap(),
    };
    */

    let app = Box::new(App {
        //state: State::Navigating,
        //system_view: SystemView::new(),
        //left_side_menu: LeftSideMenu::new(width, height),
        //fonts,
        width,
        height,
    });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}
