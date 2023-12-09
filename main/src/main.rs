use flowi::window::{WindowFlags, Window};
use flowi::button::Button;
use flowi::Application;
use flowi::font::Font;

struct App {
    _dummy: u32,
    montserrat_font: Font,
}

fn main_loop(app: &mut App) {
    Window::begin("Testing foobar", WindowFlags::NONE);
    
    Font::push(app.montserrat_font);

    if Button::regular("Hello, world!") {
        println!("Clicked!");
    }
    
    Font::pop();

    Window::end();
}

fn main() {
    let settings = flowi::ApplicationSettings { 
        width: 1280,
        height: 720,
    };

    let mut flowi_app = Application::new(&settings).unwrap();

    let app = Box::new(App {
        _dummy: 1337,
        montserrat_font: Font::load("data/fonts/montserrat/Montserrat-Regular.ttf", 32).unwrap(), 
    });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}

