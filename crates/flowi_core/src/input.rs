use crate::primitives::Vec2;

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

#[derive(Debug, Default)]
struct MouseState {
    clicked_pos: Vec2,
    down: bool,
    clicked: bool,
    double_clicked: bool,
    released: bool,
    down_duration: f32,
}

pub struct Input {
    pub(crate) mouse_position: Vec2,
    pub(crate) mouse_buttons: [MouseState; 5],
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse_position: Vec2::default(),
            mouse_buttons: Default::default(),
        }
    }

    /// Queue a new key down/up event.
    /// Key should be "translated" (as in, generally [Key::A] matches the key end-user would use to emit an 'A' character)
    pub fn add_key_event(_key: Key, _down: bool) {}

    /// Queue a new key down/up event for analog values (
    /// e.g. ImGuiKey_Gamepad_ values). Dead-zones should be handled by the backend.
    pub fn add_key_analog_event(_key: Key, _down: bool, _value: f32) {}

    /// Queue a mouse position update. Use None to signify no mouse (e.g. app not focused and not hovered)
    pub fn add_mouse_pos_event(&mut self, pos: Option<(f32, f32)>) {
        if let Some((x, y)) = pos {
            self.mouse_position = Vec2::new(x, y);
        }
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = Vec2::new(x, y);
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
    pub fn add_focus_event(_focused: bool) {}

    /// Queue a new character input
    pub fn add_char_event(_c: i32) {}

    /// Update the state
    pub fn update(&mut self, delta_time: f32) {
        for mb in self.mouse_buttons.iter_mut() {
            mb.clicked = mb.down && mb.down_duration < 0.0;
            mb.released = !mb.down && mb.down_duration >= 0.0;

            mb.down_duration = if mb.down {
                if mb.down_duration < 0.0 {
                    0.0
                } else {
                    mb.down_duration + delta_time
                }
            } else {
                -1.0
            };
        }
    }
}
