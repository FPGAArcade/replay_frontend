use flowi::button::Button;
use flowi::font::Font;
use flowi::image::Image;
use flowi::ui::Ui;
use flowi::window::{Window, WindowFlags};
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
    test_image: Image,
    first_frame: bool,
    width: u32,
    height: u32,
}

fn main_loop(app: &mut App) {
    Font::push(app.montserrat_font);

    // Kinda a hack, but will do for now
    if !app.first_frame {
        app.first_frame = true;
        app.left_side_menu.update_size(app.width, app.height, 1.0, 1.0);
    }

    app.left_side_menu.show();

    Font::pop();

    /*
    Window::begin("Testing foobar", WindowFlags::NONE);


    if Button::regular("Hello, world!") {
        println!("Clicked!");
    }

    Ui::image(app.test_image);

    Font::pop();

    Window::end();
    */
}

fn main() {
    let width = 1920;
    let height = 1080;

    let settings = flowi::ApplicationSettings { width, height };

    let mut flowi_app = Application::new(&settings).unwrap();

    let app = Box::new(App {
        state: State::Navigating,
        left_side_menu: LeftSideMenu::new(width, height),
        montserrat_font: Font::load("data/fonts/montserrat/Montserrat-Regular.ttf", 32).unwrap(),
        test_image: Image::create_from_file("data/test_data/planet.png"),
        first_frame: false,
        width,
        height,
    });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}
