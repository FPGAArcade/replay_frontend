use glam::Vec2;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseSource {
    /// Input is coming from an actual mouse.
    Mouse = 0,
    /// Input is coming from a touch screen
    /// (no hovering prior to initial press, less precise initial press aiming, dual-axis wheeling possible).
    TouchScreen = 1,
    /// Input is coming from a pressure/magnetic pen (often used in conjunction with high-sampling rates).
    Pen = 2,
}

/// A key identifier
#[allow(missing_docs)] // Self-describing
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Key {
    Tab,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    PageUp,
    PageDown,
    Home,
    End,
    Insert,
    Delete,
    Backspace,
    Space,
    Enter,
    Escape,
    LeftCtrl,
    LeftShift,
    LeftAlt,
    LeftSuper,
    RightCtrl,
    RightShift,
    RightAlt,
    RightSuper,
    Menu,
    Alpha0,
    Alpha1,
    Alpha2,
    Alpha3,
    Alpha4,
    Alpha5,
    Alpha6,
    Alpha7,
    Alpha8,
    Alpha9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Semicolon,
    Equal,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Keypad0,
    Keypad1,
    Keypad2,
    Keypad3,
    Keypad4,
    Keypad5,
    Keypad6,
    Keypad7,
    Keypad8,
    Keypad9,
    KeypadDecimal,
    KeypadDivide,
    KeypadMultiply,
    KeypadSubtract,
    KeypadAdd,
    KeypadEnter,
    KeypadEqual,
    GamepadStart,
    GamepadBack,
    GamepadFaceLeft,
    GamepadFaceRight,
    GamepadFaceUp,
    GamepadFaceDown,
    GamepadDpadLeft,
    GamepadDpadRight,
    GamepadDpadUp,
    GamepadDpadDown,
    GamepadL1,
    GamepadR1,
    GamepadL2,
    GamepadR2,
    GamepadL3,
    GamepadR3,
    GamepadLStickLeft,
    GamepadLStickRight,
    GamepadLStickUp,
    GamepadLStickDown,
    GamepadRStickLeft,
    GamepadRStickRight,
    GamepadRStickUp,
    GamepadRStickDown,
    MouseLeft,
    MouseRight,
    MouseMiddle,
    MouseX1,
    MouseX2,
    MouseWheelX,
    MouseWheelY,
    ReservedForModCtrl,
    ReservedForModShift,
    ReservedForModAlt,
    ReservedForModSuper,
    ModCtrl,
    ModShift,
    ModAlt,
    ModSuper,
    ModShortcut,
}

#[derive(Debug)]
pub struct InputSettings {
    pub mouse_threshold: f32,
    pub double_click_time: f32,
    pub double_click_max_dist_x2: f32,
    pub key_repeat_delay: f32,
    pub key_repeat_rate: f32,
}

#[derive(Debug, Default)]
pub(crate) struct MouseState {
    pub(crate) clicked_pos: Vec2,
    pub(crate) down: bool,
    pub(crate) clicked: bool,
    pub(crate) double_clicked: bool,
    pub(crate) released: bool,
    pub(crate) down_duration: f32,
    pub(crate) clicked_time: f32,
    pub(crate) released_time: f32,
    pub(crate) clicked_count: i32,
    pub(crate) down_duration_prev: f32,
}

#[derive(Debug)]
pub struct Input {
    pub(crate) settings: InputSettings,
    pub(crate) mouse_pos: Vec2,
    pub(crate) mouse_pos_prev: Vec2,
    pub(crate) mouse_buttons: [MouseState; 5],
    pub delta_time: f32,
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse_pos: Vec2::new(f32::NAN, f32::NAN),
            mouse_pos_prev: Vec2::new(f32::NAN, f32::NAN),
            mouse_buttons: Default::default(),
            settings: InputSettings {
                mouse_threshold: 0.0,
                double_click_time: 0.60,
                double_click_max_dist_x2: 6.0 * 6.0,
                key_repeat_delay: 0.250,
                key_repeat_rate: 0.050,
            },
            delta_time: 0.0,
        }
    }

    /// Queue a new key down/up event.
    /// Key should be "translated" (as in, generally [Key::A] matches the key end-user would use to emit an 'A' character)
    pub fn add_key_event(&mut self, _key: Key, _down: bool) {}

    /// Queue a new key down/up event for analog values (
    /// e.g. ImGuiKey_Gamepad_ values). Dead-zones should be handled by the backend.
    pub fn add_key_analog_event(&mut self, _key: Key, _down: bool, _value: f32) {}

    /// Queue a mouse position update. Use None to signify no mouse (e.g. app not focused and not hovered)
    pub fn add_mouse_pos_event(&mut self, pos: Option<(f32, f32)>) {
        if let Some((x, y)) = pos {
            self.mouse_pos = Vec2::new(x, y);
        }
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_pos = Vec2::new(x, y);
    }

    /// Queue a mouse button change
    pub fn add_mouse_button_event(&mut self, button: i32, down: bool) {
        self.mouse_buttons[button as usize].down = down;
    }

    /// Queue a mouse wheel update.
    /// wheel_y<0: scroll down, wheel_y>0: scroll up, wheel_x<0: scroll right, wheel_x>0: scroll left.
    pub fn add_mouse_wheel_event(_x: f32, _y: f32) {}

    /// Queue a mouse source change (Mouse/TouchScreen/Pen)
    pub fn add_mouse_source_event(_source: MouseSource) {}

    /// Queue a gain/loss of focus for the application (generally based on OS/platform focus of your window)
    pub fn add_focus_event(&mut self, _focused: bool) {}

    /// Queue a new character input
    pub fn add_char_event(&mut self, _c: i32) {}

    /// Update the state
    pub fn update(&mut self, time: f32, delta_time: f32) {
        self.update_mouse_state(time, delta_time);
    }

    // Inpseration taken from Dear imgui
    fn update_mouse_state(&mut self, time: f32, delta_time: f32) {
        // If mouse moved we re-enable mouse hovering in case it was disabled by keyboard/gamepad.
        // In theory should use a >0.0 threshold but would need to reset in everywhere we set this to true.
        //if io.mouse_delta.x != 0.0 || io.mouse_delta.y != 0.0 {
        //    ctx.nav_highlight_item_under_nav = false;
        //}

        for button in &mut self.mouse_buttons {
            button.clicked = button.down && button.down_duration < 0.0;
            button.clicked_count = 0;
            button.released = !button.down && button.down_duration >= 0.0;

            if button.released {
                button.released_time = time;
            }

            button.down_duration_prev = button.down_duration;
            button.down_duration = if button.down {
                if button.down_duration < 0.0 {
                    0.0
                } else {
                    button.down_duration + delta_time
                }
            } else {
                -1.0
            };

            if button.clicked {
                if time - button.clicked_time < self.settings.double_click_time {
                    let delta_from_click_pos = if self.mouse_pos != Vec2::new(f32::NAN, f32::NAN) {
                        self.mouse_pos - button.clicked_pos
                    } else {
                        Vec2::new(0.0, 0.0)
                    };

                    if delta_from_click_pos.length_squared()
                        < self.settings.double_click_max_dist_x2
                    {
                        button.clicked_count += 1;
                    }
                } else {
                    button.clicked_count = 1;
                };

                button.clicked_time = time;
                button.clicked_pos = self.mouse_pos;
            }

            button.double_clicked = button.clicked_count == 2;

            // Clicking any mouse button reactivate mouse hovering which may have been deactivated by keyboard/gamepad navigation
            //if button.clicked {
            //    ctx.nav_highlight_item_under_nav = false;
            //}
        }
    }
}
