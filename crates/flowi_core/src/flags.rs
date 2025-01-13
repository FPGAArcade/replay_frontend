bitflags! {
    pub struct BoxFlags: u64 {
        // Interaction
        const MOUSE_CLICKABLE            = 1 << 0;
        const KEYBOARD_CLICKABLE         = 1 << 1;
        const DROP_SITE                  = 1 << 2;
        const CLICK_TO_FOCUS             = 1 << 3;
        const SCROLL                     = 1 << 4;
        const VIEW_SCROLL_X              = 1 << 5;
        const VIEW_SCROLL_Y              = 1 << 6;
        const VIEW_CLAMP_X               = 1 << 7;
        const VIEW_CLAMP_Y               = 1 << 8;
        const FOCUS_HOT                  = 1 << 9;
        const FOCUS_ACTIVE               = 1 << 10;
        const FOCUS_HOT_DISABLED         = 1 << 11;
        const FOCUS_ACTIVE_DISABLED      = 1 << 12;
        const DEFAULT_FOCUS_NAV_X        = 1 << 13;
        const DEFAULT_FOCUS_NAV_Y        = 1 << 14;
        const DEFAULT_FOCUS_EDIT         = 1 << 15;
        const FOCUS_NAV_SKIP             = 1 << 16;
        const DISABLE_TRUNCATED_HOVER    = 1 << 17;
        const DISABLED                   = 1 << 18;

        // Layout
        const FLOATING_X                 = 1 << 19;
        const FLOATING_Y                 = 1 << 20;
        const FIXED_WIDTH                = 1 << 21;
        const FIXED_HEIGHT               = 1 << 22;
        const ALLOW_OVERFLOW_X           = 1 << 23;
        const ALLOW_OVERFLOW_Y           = 1 << 24;
        const SKIP_VIEW_OFF_X            = 1 << 25;
        const SKIP_VIEW_OFF_Y            = 1 << 26;

        // Appearance / Animation
        const DRAW_DROP_SHADOW           = 1 << 27;
        const DRAW_BACKGROUND_BLUR       = 1 << 28;
        const DRAW_BACKGROUND            = 1 << 29;
        const DRAW_BORDER                = 1 << 30;
        const DRAW_SIDE_TOP              = 1 << 31;
        const DRAW_SIDE_BOTTOM           = 1 << 32;
        const DRAW_SIDE_LEFT             = 1 << 33;
        const DRAW_SIDE_RIGHT            = 1 << 34;
        const DRAW_TEXT                  = 1 << 35;
        const DRAW_TEXT_FASTPATH_CODEPOINT = 1 << 36;
        const DRAW_TEXT_WEAK             = 1 << 37;
        const DRAW_HOT_EFFECTS           = 1 << 38;
        const DRAW_ACTIVE_EFFECTS        = 1 << 39;
        const DRAW_OVERLAY               = 1 << 40;
        const DRAW_BUCKET                = 1 << 41;
        const CLIP                       = 1 << 42;
        const ANIMATE_POS_X              = 1 << 43;
        const ANIMATE_POS_Y              = 1 << 44;
        const DISABLE_TEXT_TRUNC         = 1 << 45;
        const DISABLE_ID_STRING          = 1 << 46;
        const DISABLE_FOCUS_BORDER       = 1 << 47;
        const DISABLE_FOCUS_OVERLAY      = 1 << 48;
        const HAS_DISPLAY_STRING         = 1 << 49;
        const HAS_FUZZY_MATCH_RANGES     = 1 << 50;
        const ROUND_CHILDREN_BY_PARENT   = 1 << 51;

        // Bundles
        const CLICKABLE           = Self::MOUSE_CLICKABLE.bits() | Self::KEYBOARD_CLICKABLE.bits();
        const DEFAULT_FOCUS_NAV   = Self::DEFAULT_FOCUS_NAV_X.bits() | Self::DEFAULT_FOCUS_NAV_Y.bits() | Self::DEFAULT_FOCUS_EDIT.bits();
        const FLOATING            = Self::FLOATING_X.bits() | Self::FLOATING_Y.bits();
        const FIXED_SIZE          = Self::FIXED_WIDTH.bits() | Self::FIXED_HEIGHT.bits();
        const ALLOW_OVERFLOW      = Self::ALLOW_OVERFLOW_X.bits() | Self::ALLOW_OVERFLOW_Y.bits();
        const ANIMATE_POS         = Self::ANIMATE_POS_X.bits() | Self::ANIMATE_POS_Y.bits();
        const VIEW_SCROLL         = Self::VIEW_SCROLL_X.bits() | Self::VIEW_SCROLL_Y.bits();
        const VIEW_CLAMP          = Self::VIEW_CLAMP_X.bits() | Self::VIEW_CLAMP_Y.bits();
        const DISABLE_FOCUS_EFFECTS = Self::DISABLE_FOCUS_BORDER.bits() | Self::DISABLE_FOCUS_OVERLAY.bits();
    }
}

bitflags! {
    pub(crate) struct StackFlags : u32 {
        const OWNER = 1 << 1;
        const PREF_WIDTH = 1 << 2;
        const PREF_HEIGHT = 1 << 2;
        const FIXED_WIDTH = 1 << 3;
        const FIXED_HEIGHT = 1 << 4;
        const FLAGS = 1 << 5;
        const CHILD_LAYOUT_AXIS = 1 << 6;
    }
}

