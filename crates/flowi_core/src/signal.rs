use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct SignalFlags: u32 {
        const LEFT_PRESSED = 1 << 0;
        const LEFT_RELEASED = 1 << 12;
        const LEFT_CLICKED = 1 << 15;
        const LEFT_DOUBLE_CLICKED = 1 << 18;

        const KEYBOARD_PRESSED = 1 << 24;
        const HOVERING = 1 << 25;
        const MOUSE_OVER = 1 << 26;
        const ENTER_HOVER = 1 << 27;
        const EXIT_HOVER = 1 << 28;

        const PRESSED = Self::LEFT_PRESSED.bits() | Self::KEYBOARD_PRESSED.bits();
        const RELEASED = Self::LEFT_RELEASED.bits();
        const CLICKED = Self::LEFT_CLICKED.bits() | Self::KEYBOARD_PRESSED.bits();
        const DOUBLE_CLICKED = Self::LEFT_DOUBLE_CLICKED.bits();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Signal {
    pub flags: SignalFlags,
}

impl Signal {
    #[inline]
    pub fn new() -> Self {
        Self {
            flags: SignalFlags::empty(),
        }
    }

    #[inline]
    pub fn clicked(&self) -> bool {
        self.flags.contains(SignalFlags::CLICKED)
    }

    #[inline]
    pub fn double_clicked(&self) -> bool {
        self.flags.contains(SignalFlags::DOUBLE_CLICKED)
    }

    #[inline]
    pub fn pressed(&self) -> bool {
        self.flags.contains(SignalFlags::PRESSED)
    }

    #[inline]
    pub fn released(&self) -> bool {
        self.flags.contains(SignalFlags::RELEASED)
    }

    #[inline]
    pub fn hovering(&self) -> bool {
        self.flags.contains(SignalFlags::HOVERING)
    }

    #[inline]
    pub fn mouse_over(&self) -> bool {
        self.flags.contains(SignalFlags::MOUSE_OVER)
    }
}
