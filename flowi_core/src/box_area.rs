use bitflags::bitflags;
use core::cell::UnsafeCell;
use crate::layout::{Size, Axis};
use crate::primitives::Vec2;


//static BOX_FLAG_FIXED_WIDTH: u32 = 1 << 19;
//static BOX_FLAG_FIXED_HEIGHT: u32 = 1 << 20;
static BOX_FLAG_FLOATING_X: u32 = 1 << 21;
//static BOX_FLAG_FLOATING_Y: u32 = 1 << 22;
static BOX_FLAG_ALLOW_OVERFLOW_X: u32 = 1 << 23;
//static BOX_FLAG_ALLOW_OVERFLOW_Y: u32 = 1 << 24;
//static BOX_FLAG_ANIMATE_X: u32 = 1 << 25;
//static BOX_FLAG_ANIMATE_Y: u32 = 1 << 26;

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

#[derive(Debug, Default)]
pub struct Rect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

#[derive(Debug)]
pub(crate) struct TextData {
    pub(crate) display_text: String,
    pub(crate) text_edge_padding: f32,
    //pub(crate) paint: Paint,
}

#[derive(Debug, Default)]
pub(crate) struct BoxAreaInner {
    pub(crate) pref_size: [Size; 2],
    pub(crate) calc_size: [f32; 2],
    pub(crate) text_data: Option<TextData>,
    pub(crate) child_layout_axis: Axis,
    pub(crate) calc_rel_position: [f32; 2],
    pub(crate) flags: u32,
    pub(crate) rect: Rect,
    pub(crate) view_off: [f32; 2],
    pub(crate) display_string: String,
}

#[derive(Debug, Default)]
pub struct BoxArea {
    inner: UnsafeCell<BoxAreaInner>,
    parent: Option<usize>,
    first: Option<usize>,
    last: Option<usize>,
    next: Option<usize>,
}

impl BoxAreaInner {
    #[inline]
    pub(crate) fn has_flag(&self, flag: u32) -> bool {
        (self.flags & flag) == flag
    }

    #[inline]
    pub(crate) fn is_floating_on(&self, axis: usize) -> bool {
        self.has_flag(BOX_FLAG_FLOATING_X << axis)
    }

    #[inline]
    pub(crate) fn is_overflowing_on(&self, axis: u32) -> bool {
        self.has_flag(BOX_FLAG_ALLOW_OVERFLOW_X << axis)
    }
}

impl BoxArea {
    #[inline]
    pub(crate) fn inner_borrow(&self) -> &BoxAreaInner {
        unsafe {
            &*self.inner.get()
        }
    }

    #[inline]
    pub(crate) fn inner_borrow_mut(&self) -> &mut BoxAreaInner {
        unsafe {
            &mut *self.inner.get()
        }
    }

    #[inline]
    pub(crate) fn parent<'a>(&self, boxes: &'a [BoxArea]) -> Option<&'a BoxArea> {
        self.parent.map(|p| &boxes[p])
    }

    #[inline]
    pub(crate) fn parent_mut<'a>(&self, boxes: &'a mut [BoxArea]) -> Option<&'a mut BoxArea> {
        self.parent.map(move |p| &mut boxes[p])
    }

    #[inline]
    pub(crate) fn next<'a>(&self, boxes: &'a [BoxArea]) -> Option<&'a BoxArea> {
        self.next.map(|n| &boxes[n])
    }

    #[inline]
    pub(crate) fn next_mut<'a>(&self, boxes: &'a mut [BoxArea]) -> Option<&'a mut BoxArea> {
        self.next.map(move |n| &mut boxes[n])
    }

    #[inline]
    pub(crate) fn first<'a>(&self, boxes: &'a [BoxArea]) -> Option<&'a BoxArea> {
        self.first.map(|n| &boxes[n])
    }

    #[inline]
    pub(crate) fn first_mut<'a>(&self, boxes: &'a mut [BoxArea]) -> Option<&'a mut BoxArea> {
        self.first.map(move |n| &mut boxes[n])
    }

    #[inline]
    pub(crate) fn last<'a>(&self, boxes: &'a [BoxArea]) -> Option<&'a BoxArea> {
        self.last.map(|n| &boxes[n])
    }

    #[inline]
    pub(crate) fn last_mut<'a>(&self, boxes: &'a mut [BoxArea]) -> Option<&'a mut BoxArea> {
        self.last.map(move |n| &mut boxes[n])
    }
}
