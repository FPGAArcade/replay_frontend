use flowi::Application;
use flowi::Ui;
use flowi::{fixed, grow, Id, Layout, LayoutDirection, Padding};
use flowi::Renderer;


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
}

#[rustfmt::skip]
fn main_loop(ui: &Ui, _app: &mut App) {
    ui.with_layout(Some("main_container"), [
        Layout::new()
            .width(grow!())
            .height(grow!())
            .direction(LayoutDirection::TopToBottom)
            .padding(Padding::all(16))
            .child_gap(16)
            .end()], |ui| 
   {
        ui.with_layout(Some("buttons"), [
            Layout::new()
                .width(grow!())
                .child_gap(16)
                .height(fixed!(160.0))
                .end()], |ui| 
        {
            ui.button("Test");
            ui.button("Test");
        });

        ui.with_layout(Some("buttons2"), [
            Layout::new()
                .width(grow!())
                .height(fixed!(160.0))
                .end()], |ui| 
        {
            ui.button("Test");
            ui.button("Test");
        });

    });
}

fn main() {
    let width = 1920;
    let height = 1080;

    let settings = flowi::ApplicationSettings { width, height };

    let mut flowi_app = Application::new(&settings); //.unwrap();

    let _ = flowi_app
        .ui
        .load_font("../../data/fonts/roboto/Roboto-Regular.ttf", 48);

    /*
    let fonts = Fonts {
        default: Font::load("data/fonts/montserrat/Montserrat-Regular.ttf", 56).unwrap(),
        system_header: Font::load("data/fonts/roboto/Roboto-Bold.ttf", 72).unwrap(),
        system_text: Font::load("data/fonts/roboto/Roboto-Regular.ttf", 48).unwrap(),
        rot_header: Font::load("data/fonts/roboto/Roboto-Bold.ttf", 56).unwrap(),
    };
    */

    let app = Box::new(App { width, height });

    if !flowi_app.run(app, main_loop) {
        println!("Failed to create main application");
    }
}

/*
//fn main() {
    let clay = Clay::new(Dimensions::new(WIDTH as f32, HEIGHT as f32));
    let mut renderer = Renderer::new(ColorSpace::Linear, (WIDTH, HEIGHT), (10, 12));

    // Limit to max ~60 fps update rate
    //window.set_target_fps(60);

    loop {
        //while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        let content_background_config = Rectangle::new()
            .color(Color::u_rgb(90, 90, 90))
            .corner_radius(CornerRadius::All(8.))
            .end();

        // Begin the layout
        clay.begin();

        // Adds a red rectangle with a corner radius of 5.
        // The Layout makes the rectangle have a width and height of 50.
        clay.with(
            [
                Id::new("OuterContainer"),
                Layout::new()
                    .width(grow!())
                    .height(grow!())
                    .direction(LayoutDirection::TopToBottom)
                    .padding(Padding::all(16))
                    .child_gap(16)
                    .end(),
                Rectangle::new()
                    .color(Color::u_rgb(43, 41, 51))
                    .corner_radius(CornerRadius::All(5.))
                    .end(),
            ],
            |clay| {
                clay.with(
                    [
                        Id::new("HeaderBar"),
                        Layout::new()
                            .width(grow!())
                            .height(fixed!(60.))
                            .padding(Padding::all(16))
                            .child_gap(16)
                            .child_alignment(Alignment::new(
                                LayoutAlignmentX::Left,
                                LayoutAlignmentY::Center,
                            ))
                            .end(),
                        content_background_config,
                    ],
                    |_| {},
                );

                clay.with(
                    [
                        Id::new("LowerContent"),
                        Layout::new()
                            .width(grow!())
                            .height(grow!())
                            .child_gap(16)
                            .end(),
                    ],
                    |clay| {
                        clay.with(
                            [
                                Id::new("Sidebar"),
                                Layout::new()
                                    .width(fixed!(250.))
                                    .height(grow!())
                                    .direction(LayoutDirection::TopToBottom)
                                    .padding(Padding::all(16))
                                    .end(),
                                content_background_config,
                            ],
                            |_| {},
                        );

                        clay.with(
                            [
                                Id::new("MainContent"),
                                Layout::new()
                                    .width(grow!())
                                    .height(grow!())
                                    .direction(LayoutDirection::TopToBottom)
                                    .end(),
                                content_background_config,
                            ],
                            |_| {},
                        );
                    },
                );
            },
        );

        // Return the list of render commands of your layout
        let render_commands = clay.end();

        renderer.begin_frame();

        for command in render_commands {
            let aabb = f32x4::new(
                command.bounding_box.x,
                command.bounding_box.y,
                command.bounding_box.x + command.bounding_box.width,
                command.bounding_box.y + command.bounding_box.height,
            );

            //println!("{:?}", aabb.to_array());

            match &command.config {
                RenderCommandConfig::Rectangle(rectangle) => {
                    let color = renderer.get_color_from_floats_0_255(
                        rectangle.color.r,
                        rectangle.color.g,
                        rectangle.color.b,
                        rectangle.color.a,
                    );

                    let corner_radius = match rectangle.corner_radius {
                        CornerRadius::All(radius) => f32x4::new(radius, radius, radius, radius),

                        CornerRadius::Individual {
                            top_left,
                            top_right,
                            bottom_left,
                            bottom_right,
                        } => f32x4::new(top_left, top_right, bottom_left, bottom_right),
                    };

                    let primitive = RenderPrimitive {
                        aabb,
                        color,
                        corner_radius,
                    };

                    renderer.add_primitive(primitive);
                }

                _ => {}
            }
        }

        renderer.flush_frame(&mut buffer);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        //window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
*/
