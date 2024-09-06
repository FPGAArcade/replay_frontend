use bitflags::bitflags;
use crate::box_area::BoxFlags;
use crate::Flowi;

bitflags! {
    pub(crate) struct SignalFlags: u32 {
        const LEFT_PRESSED             = 1 << 0;
        const MIDDLE_PRESSED           = 1 << 1;
        const RIGHT_PRESSED            = 1 << 2;

        const LEFT_DRAGGING            = 1 << 3;
        const MIDDLE_DRAGGING          = 1 << 4;
        const RIGHT_DRAGGING           = 1 << 5;
/*
        const LEFT_DOUBLE_DRAGGING     = 1 << 6;
        const MIDDLE_DOUBLE_DRAGGING   = 1 << 7;
        const RIGHT_DOUBLE_DRAGGING    = 1 << 8;

        const LEFT_TRIPLE_DRAGGING     = 1 << 9;
        const MIDDLE_TRIPLE_DRAGGING   = 1 << 10;
        const RIGHT_TRIPLE_DRAGGING    = 1 << 11;
*/
        const LEFT_CLICKED             = 1 << 15;
        const MIDDLE_CLICKED           = 1 << 16;
        const RIGHT_CLICKED            = 1 << 17;

        const LEFT_RELEASED            = 1 << 12;
        const MIDDLE_RELEASED          = 1 << 13;
        const RIGHT_RELEASED           = 1 << 14;

        const LEFT_DOUBLE_CLICKED      = 1 << 18;
        const MIDDLE_DOUBLE_CLICKED    = 1 << 19;
        const RIGHT_DOUBLE_CLICKED     = 1 << 20;

        const LEFT_TRIPLE_CLICKED      = 1 << 21;
        const MIDDLE_TRIPLE_CLICKED    = 1 << 22;
        const RIGHT_TRIPLE_CLICKED     = 1 << 23;

        const KEYBOARD_PRESSED         = 1 << 24;

        const HOVERING                 = 1 << 25;
        const MOUSE_OVER               = 1 << 26;

        const COMMIT                   = 1 << 27;

        // High-level combinations
        const PRESSED                  = Self::LEFT_PRESSED.bits() | Self::KEYBOARD_PRESSED.bits();
        const RELEASED                 = Self::LEFT_RELEASED.bits();

        const CLICKED                  = Self::LEFT_CLICKED.bits() | Self::KEYBOARD_PRESSED.bits();

        const DOUBLE_CLICKED           = Self::LEFT_DOUBLE_CLICKED.bits();
        const TRIPLE_CLICKED           = Self::LEFT_TRIPLE_CLICKED.bits();
        const DRAGGING                 = Self::LEFT_DRAGGING.bits();
    }
}

// Define a struct to hold the flags
pub struct Signals {
    pub(crate) flags: SignalFlags,
}

impl Signals {
    pub fn new() -> Self {
        Signals {
            flags: SignalFlags::empty(),
        }
    }

    #[inline]
    pub fn is_left_pressed(&self) -> bool {
        self.flags.contains(SignalFlags::LEFT_PRESSED)
    }

    #[inline]
    pub fn is_middle_pressed(&self) -> bool {
        self.flags.contains(SignalFlags::MIDDLE_PRESSED)
    }

    #[inline]
    pub fn is_right_pressed(&self) -> bool {
        self.flags.contains(SignalFlags::RIGHT_PRESSED)
    }
/*
    #[inline]
    pub fn is_left_dragging(&self) -> bool {
        self.flags.contains(SignalFlags::LEFT_DRAGGING)
    }

    #[inline]
    pub fn is_middle_dragging(&self) -> bool {
        self.flags.contains(SignalFlags::MIDDLE_DRAGGING)
    }

    #[inline]
    pub fn is_right_dragging(&self) -> bool {
        self.flags.contains(SignalFlags::RIGHT_DRAGGING)
    }
*/

    #[inline]
    pub fn is_left_clicked(&self) -> bool {
        self.flags.contains(SignalFlags::LEFT_CLICKED)
    }

    #[inline]
    pub fn is_hovering(&self) -> bool {
        self.flags.contains(SignalFlags::HOVERING)
    }
}

impl Flowi {
    pub fn button(&mut self, text: &str) {
        //let box = build_box_from_string(
        let b = BoxFlags::DRAW_BACKGROUND |
        BoxFlags::DRAW_BORDER |
        BoxFlags::DRAW_TEXT;

    }

    // move
    fn ui_build_box_from_string(&mut self, flags: BoxFlags, text: &str) {

    } 
}
