use flowi::window::{WindowFlags, Window};
use flowi::button::Button;
use flowi::Application;

struct App {
    dummy: u32,
}

fn main_loop(app: &mut App) {
    Window::begin("Testing foobar", WindowFlags::NONE);

    if Button::regular("Hello, world!") {
        println!("Clicked!");
    }

    Window::end();
}

fn main() {
    let settings = flowi::ApplicationSettings { 
        width: 1280,
        height: 720,
    };

    let mut flowi_app = Application::new(&settings).unwrap();

    let app = Box::new(App {
        dummy: 1337,
    });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}

