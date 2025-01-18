use crate::application::Window;
use flowi_core::input::{Input, Key};
use flowi_core::ApplicationSettings;

use sdl2::{
    //controller::{Axis, Button, GameController},
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::PixelFormatEnum,
    render::Texture,
    render::TextureAccess,
};

fn translate_sdl2_to_flowi_key(key: Keycode) -> Option<Key> {
    match key {
        Keycode::A => Some(Key::A),
        Keycode::B => Some(Key::B),
        Keycode::C => Some(Key::C),
        Keycode::D => Some(Key::D),
        Keycode::E => Some(Key::E),
        Keycode::F => Some(Key::F),
        Keycode::G => Some(Key::G),
        Keycode::H => Some(Key::H),
        Keycode::I => Some(Key::I),
        Keycode::J => Some(Key::J),
        Keycode::K => Some(Key::K),
        Keycode::L => Some(Key::L),
        Keycode::M => Some(Key::M),
        Keycode::N => Some(Key::N),
        Keycode::O => Some(Key::O),
        Keycode::P => Some(Key::P),
        Keycode::Q => Some(Key::Q),
        Keycode::R => Some(Key::R),
        Keycode::S => Some(Key::S),
        Keycode::T => Some(Key::T),
        Keycode::U => Some(Key::U),
        Keycode::V => Some(Key::V),
        Keycode::W => Some(Key::W),
        Keycode::X => Some(Key::X),
        Keycode::Y => Some(Key::Y),
        Keycode::Z => Some(Key::Z),
        Keycode::Num0 => Some(Key::Keypad0),
        Keycode::Num1 => Some(Key::Keypad1),
        Keycode::Num2 => Some(Key::Keypad2),
        Keycode::Num3 => Some(Key::Keypad3),
        Keycode::Num4 => Some(Key::Keypad4),
        Keycode::Num5 => Some(Key::Keypad5),
        Keycode::Num6 => Some(Key::Keypad6),
        Keycode::Num7 => Some(Key::Keypad7),
        Keycode::Num8 => Some(Key::Keypad8),
        Keycode::Num9 => Some(Key::Keypad9),
        Keycode::Escape => Some(Key::Escape),
        Keycode::LCtrl => Some(Key::LeftCtrl),
        Keycode::LShift => Some(Key::LeftShift),
        Keycode::LAlt => Some(Key::LeftAlt),
        Keycode::LGui => Some(Key::LeftSuper),
        Keycode::RCtrl => Some(Key::RightCtrl),
        Keycode::RShift => Some(Key::RightShift),
        Keycode::RAlt => Some(Key::RightAlt),
        Keycode::RGui => Some(Key::RightSuper),
        Keycode::Application => Some(Key::Menu),
        Keycode::LeftBracket => Some(Key::LeftBracket),
        Keycode::RightBracket => Some(Key::RightBracket),
        Keycode::Semicolon => Some(Key::Semicolon),
        Keycode::Comma => Some(Key::Comma),
        Keycode::Period => Some(Key::Period),
        //Keycode::Apostrophe => Some(Key::Apostrophe),
        Keycode::Slash => Some(Key::Slash),
        Keycode::Backslash => Some(Key::Backslash),
        //Keycode::Grave => Some(Key::GraveAccent),
        Keycode::Equals => Some(Key::Equal),
        Keycode::Minus => Some(Key::Minus),
        Keycode::Space => Some(Key::Space),
        Keycode::Return => Some(Key::Enter),
        Keycode::Backspace => Some(Key::Backspace),
        Keycode::Tab => Some(Key::Tab),
        Keycode::PageUp => Some(Key::PageUp),
        Keycode::PageDown => Some(Key::PageDown),
        Keycode::End => Some(Key::End),
        Keycode::Home => Some(Key::Home),
        Keycode::Insert => Some(Key::Insert),
        Keycode::Delete => Some(Key::Delete),
        Keycode::Left => Some(Key::LeftArrow),
        Keycode::Right => Some(Key::RightArrow),
        Keycode::Up => Some(Key::UpArrow),
        Keycode::Down => Some(Key::DownArrow),
        Keycode::Kp0 => Some(Key::Keypad0),
        Keycode::Kp1 => Some(Key::Keypad1),
        Keycode::Kp2 => Some(Key::Keypad2),
        Keycode::Kp3 => Some(Key::Keypad3),
        Keycode::Kp4 => Some(Key::Keypad4),
        Keycode::Kp5 => Some(Key::Keypad5),
        Keycode::Kp6 => Some(Key::Keypad6),
        Keycode::Kp7 => Some(Key::Keypad7),
        Keycode::Kp8 => Some(Key::Keypad8),
        Keycode::Kp9 => Some(Key::Keypad9),
        Keycode::F1 => Some(Key::F1),
        Keycode::F2 => Some(Key::F2),
        Keycode::F3 => Some(Key::F3),
        Keycode::F4 => Some(Key::F4),
        Keycode::F5 => Some(Key::F5),
        Keycode::F6 => Some(Key::F6),
        Keycode::F7 => Some(Key::F7),
        Keycode::F8 => Some(Key::F8),
        Keycode::F9 => Some(Key::F9),
        Keycode::F10 => Some(Key::F10),
        Keycode::F11 => Some(Key::F11),
        Keycode::F12 => Some(Key::F12),
        Keycode::Pause => Some(Key::Pause),
        _ => None,
    }
}

pub(crate) struct Sdl2Window {
    sdl_context: sdl2::Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
    texture: Texture,
    time: f64,
    shift: u8,
    should_close: bool,
}

impl Sdl2Window {
    fn update_input(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = translate_sdl2_to_flowi_key(keycode) {
                        // Close window with ESC
                        if key == Key::Escape {
                            self.should_close = true;
                        }
                        Input::add_key_event(key, true);
                    } else {
                        println!("Unknown key: {:?}", keycode);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = translate_sdl2_to_flowi_key(keycode) {
                        Input::add_key_event(key, false);
                    }
                }
                //Event::MouseMotion { x, y, .. } => {
                Event::MouseMotion { .. } => {
                    //Input::add_mouse_pos_event(x as f32, y as f32);
                }
                //Event::MouseButtonDown { mouse_btn, .. } => {
                Event::MouseButtonDown { .. } => {
                    /*
                    Input::add_mouse_button_event(
                        Self::translate_sdl2_mouse_button(mouse_btn),
                        true,
                    );
                    */
                }
                //Event::MouseButtonUp { mouse_btn, .. } => {
                Event::MouseButtonUp { .. } => {
                    /*
                    Input::add_mouse_button_event(
                        Self::translate_sdl2_mouse_button(mouse_btn),
                        false,
                    );
                    */
                }
                Event::Window {
                    win_event: sdl2::event::WindowEvent::FocusGained,
                    ..
                } => {
                    Input::add_focus_event(true);
                }
                Event::Window {
                    win_event: sdl2::event::WindowEvent::FocusLost,
                    ..
                } => {
                    Input::add_focus_event(false);
                }
                // Handle other events as needed
                _ => {}
            }
        }

        self.update_modifiers();
        self.update_mouse_data();
        //self.update_pad();
    }

    fn update_modifiers(&mut self) {
        let keyboard_state = self.event_pump.keyboard_state();
        let ctrl = keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::LCtrl)
            || keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::RCtrl);
        let shift = keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::LShift)
            || keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::RShift);
        let alt = keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::LAlt)
            || keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::RAlt);
        let super_key = keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::LGui)
            || keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::RGui);

        Input::add_key_event(Key::LeftCtrl, ctrl);
        Input::add_key_event(Key::LeftShift, shift);
        Input::add_key_event(Key::LeftAlt, alt);
        Input::add_key_event(Key::LeftSuper, super_key);
    }

    fn update_mouse_data(&mut self) {
        // Assuming window focus is handled elsewhere or not relevant for SDL2
        let mouse_state = self.event_pump.mouse_state();
        let _x = mouse_state.x();
        let _y = mouse_state.y();

        //Input::add_mouse_pos_event(x as f32, y as f32);
    }

    #[allow(dead_code)]
    fn translate_sdl2_mouse_button(button: MouseButton) -> i32 {
        match button {
            MouseButton::Left => 1,
            MouseButton::Middle => 2,
            MouseButton::Right => 3,
            // Map other buttons as needed
            _ => 0,
        }
    }

    /*
    fn update_pad(&mut self, controller: &GameController) {
        let digital_buttons = [
            (Key::GamepadBack, Button::Back, 6),
            (Key::GamepadStart, Button::Start, 7),
            (Key::GamepadFaceLeft, Button::X, 2), // Xbox X, PS Square
            (Key::GamepadFaceRight, Button::B, 1), // Xbox B, PS Circle
            (Key::GamepadFaceUp, Button::Y, 3),   // Xbox Y, PS Triangle
            (Key::GamepadFaceDown, Button::A, 0), // Xbox A, PS Cross
            (Key::GamepadDpadLeft, Button::DPadLeft, 14),
            (Key::GamepadDpadRight, Button::DPadRight, 12),
            (Key::GamepadDpadUp, Button::DPadUp, 11),
            (Key::GamepadDpadDown, Button::DPadDown, 13),
            (Key::GamepadL1, Button::LeftShoulder, 4),
            (Key::GamepadR1, Button::RightShoulder, 5),
            (Key::GamepadL3, Button::LeftStick, 8),
            (Key::GamepadR3, Button::RightStick, 9),
        ];

        let analog_buttons = [
            (Key::GamepadL2, Axis::TriggerLeft, 4, -32768, 32767),
            (Key::GamepadR2, Axis::TriggerRight, 5, -32768, 32767),
            (Key::GamepadLStickLeft, Axis::LeftX, 0, -32768, 0),
            (Key::GamepadLStickRight, Axis::LeftX, 0, 0, 32767),
            (Key::GamepadLStickUp, Axis::LeftY, 1, -32768, 0),
            (Key::GamepadLStickDown, Axis::LeftY, 1, 0, 32767),
            (Key::GamepadRStickLeft, Axis::RightX, 2, -32768, 0),
            (Key::GamepadRStickRight, Axis::RightX, 2, 0, 32767),
            (Key::GamepadRStickUp, Axis::RightY, 3, -32768, 0),
            (Key::GamepadRStickDown, Axis::RightY, 3, 0, 32767),
        ];

        // Digital buttons
        for (key, button, _index) in digital_buttons.iter() {
            let pressed = controller.button(*button);
            // Assuming Input::add_key_event exists and handles the logic for key events
            Input::add_key_event(*key, pressed);
        }

        // Analog buttons and sticks
        for (key, axis, _index, min, max) in analog_buttons.iter() {
            let value = controller.axis(*axis) as i32;
            let normalized_value = (value - min) as f32 / (max - min) as f32;
            // Assuming Input::add_key_analog_event exists and handles the logic for analog input events
            Input::add_key_analog_event(
                *key,
                normalized_value > 0.0,
                normalized_value.clamp(0.0, 1.0),
            );
        }
    }
    */

    fn update_texture(texture: &mut Texture, color_shift: u8) -> Result<(), String> {
        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..1080 - 1 {
                    for x in 0..1920 - 1 {
                        let offset = y * pitch + x * 4;
                        buffer[offset] = color_shift; // Red
                        buffer[offset + 1] = 64; // Green
                        buffer[offset + 2] = 255 - color_shift; // Blue
                        buffer[offset + 3] = 0; // Alpha 
                    }
                }
            })
            .map_err(|e| e.to_string())
    }
}

impl Window for Sdl2Window {
    fn new(_settings: &ApplicationSettings) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let width = 1920; //core::cmp::max(settings.width as u32, 800);
        let height = 1080; //core::cmp::max(settings.height as u32, 600);

        let window = video_subsystem
            .window("Flowi", width, height)
            .position_centered()
            .build()
            .expect("Failed to create SDL window.");

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 0, 0));

        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture(
                PixelFormatEnum::RGBA8888,
                TextureAccess::Streaming,
                width,
                height,
            )
            .unwrap();

        canvas.clear();
        canvas.present();

        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            sdl_context,
            canvas,
            texture,
            shift: 0,
            event_pump,
            time: 0.0,
            should_close: false,
        }
    }

    fn update(&mut self) {
        let current_time = f64::max(
            self.time + 0.00001,
            self.sdl_context.timer().unwrap().ticks() as f64 / 1000.0,
        );
        let _delta_time = if self.time > 0.0 {
            current_time - self.time
        } else {
            1.0 / 60.0
        };

        let _display_size = self.canvas.window().drawable_size();
        let _window_size = self.canvas.window().size();

        /*
        Input::update_screen_size_time(
            display_size.0 as _,
            display_size.1 as _,
            window_size.0 as _,
            window_size.1 as _,
            delta_time as _,
        );
        */

        self.time = current_time;

        // In SDL2, input and window events are handled through the event pump in the main loop
        // Thus, methods like update_input(), update_mouse_data(), update_modifiers(), and update_pad()
        // should be integrated into the main event loop or adapted accordingly.

        self.update_input();

        Self::update_texture(&mut self.texture, self.shift).unwrap();

        self.canvas
            .copy(
                &self.texture,
                None,
                Some(sdl2::rect::Rect::new(0, 0, 1920, 1080)),
            )
            .unwrap();
        self.canvas.present();

        self.shift = self.shift.wrapping_add(1);
    }

    /*
    fn is_focused(&self) -> bool {
        // SDL2 window focus management
        self.canvas.window().window_flags()
            & sdl2::sys::SDL_WindowFlags::SDL_WINDOW_INPUT_FOCUS as u32
            != 0
    }
    */

    fn should_close(&mut self) -> bool {
        self.should_close
    }
}

// Note: ApplicationSettings struct is assumed to be defined elsewhere in your code.
