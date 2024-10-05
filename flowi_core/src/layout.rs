use crate::box_area::BoxArea;
use crate::box_area::BoxAreaPtr;
use crate::box_area::StackFlags;
use crate::internal_error::InternalError as Error;
use crate::Flowi;
use arena_allocator::{Arena, PodArena};
use smallvec::SmallVec;

pub struct Layout {
    pub owner: PodArena<BoxAreaPtr>,
    pub pref_width: PodArena<Size>,
    pub pref_height: PodArena<Size>,
    pub fixed_x: PodArena<f32>,
    pub fixed_y: PodArena<f32>,
    pub flags: PodArena<u64>,
    pub child_layout_axis: PodArena<Axis>,
}

#[cfg(feature = "tracing-instrument")]
use tracing::instrument;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
enum SizeKind {
    #[default]
    Null,
    Pixels,
    Em,
    TextContent,
    PercentOfAncestor,
    ChildrenSum,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    kind: SizeKind,
    value: f32,
    strictness: f32,
}

impl Size {
    pub fn in_pixels(value: f32) -> Self {
        Self {
            kind: SizeKind::Pixels,
            value,
            strictness: 1.0,
        }
    }

    pub fn in_pixels_strict(value: f32, strictness: f32) -> Self {
        Self {
            kind: SizeKind::Pixels,
            value,
            strictness,
        }
    }

    pub fn from_children() -> Self {
        Self {
            kind: SizeKind::ChildrenSum,
            value: 0.0,
            strictness: 1.0,
        }
    }

    pub fn from_children_strict(strictness: f32) -> Self {
        Self {
            kind: SizeKind::ChildrenSum,
            value: 0.0,
            strictness,
        }
    }

    pub fn in_percent_of_ancestor(value: f32) -> Self {
        Self {
            kind: SizeKind::PercentOfAncestor,
            value,
            strictness: 1.0,
        }
    }

    pub fn in_percent_of_ancestor_strict(value: f32, strictness: f32) -> Self {
        Self {
            kind: SizeKind::PercentOfAncestor,
            value,
            strictness,
        }
    }
}

#[derive(Debug)]
struct Paint {
    font_metrics: FontMetrics,
}

#[derive(Debug)]
struct FontMetrics {
    descent: f32,
    top: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Axis {
    #[default]
    Horizontal,
    Vertical,
}

#[cfg_attr(feature = "tracing-instrument", tracing::instrument)]
fn do_layout_axis(root: &mut BoxArea) {
    do_layout_for(root, 0);
    do_layout_for(root, 1);
}

#[cfg_attr(feature = "tracing-instrument", tracing::instrument)]
fn do_layout_for(root: &mut BoxArea, axis: usize) {
    solve_independent_sizes_for(root, axis);
    solve_downward_dependent_sizes_for(root, axis);
    solve_upward_dependent_sizes_for(root, axis);
    solve_downward_dependent_sizes_for(root, axis);
    solve_size_violations(root, axis);
}

fn solve_independent_sizes_for(root: &mut BoxArea, axis: usize) {
    let size = root.pref_size[axis];

    match size.kind {
        SizeKind::Pixels => {
            root.calc_size[axis] = size.value;
        }
        SizeKind::Em => {
            root.calc_size[axis] = (size.value * top_font_size()).floor();
        }
        SizeKind::TextContent => {
            if let Some(text_data) = &root.text_data {
                //let paint = &text_data.paint;
                if axis == 0 {
                    // TODO: fix me
                    let text_width = 64.0; //paint.measure_text(&text_data.display_text);
                    let text_padding = text_data.text_edge_padding;
                    root.calc_size[axis] = (text_width + 2.0 * text_padding).floor();
                } else {
                    // TODO: fix me
                    //let metrics = &paint.font_metrics;
                    let line_height = 40.0f32; //metrics.descent - metrics.top;
                    root.calc_size[axis] = line_height.floor();
                }
            } else {
                panic!("box.text_data should not be None");
            }
        }
        _ => {}
    }

    let mut node = root.first_mut();
    while let Some(p) = node {
        solve_independent_sizes_for(p, axis);
        node = p.next_mut();
    }
}

fn solve_upward_dependent_sizes_for(root: &mut BoxArea, axis: usize) {
    let size = root.pref_size[axis];

    if size.kind != SizeKind::PercentOfAncestor {
        return;
    }

    let mut ancestor: Option<&BoxArea> = None;
    let mut parent = root.parent();

    while let Some(p) = parent {
        if p.pref_size[axis].kind != SizeKind::ChildrenSum {
            ancestor = Some(p);
            break;
        }
        parent = p.parent();
    }

    if let Some(a) = ancestor {
        root.calc_size[axis] = a.calc_size[axis] * size.value;
    } else {
        println!("{} is left out of size calculations!", root.display_string);
    }
}

fn solve_downward_dependent_sizes_for(root: &mut BoxArea, axis: usize) {
    let axis = axis & 1;

    let mut node = root.first_mut();
    while let Some(next) = node {
        solve_downward_dependent_sizes_for(next, axis);
        node = next.next_mut();
    }

    let size = root.pref_size[axis];

    if size.kind != SizeKind::ChildrenSum {
        return;
    }

    let mut node = root.first();
    let mut sum = 0.0;
    while let Some(p) = node {
        if axis == root.child_layout_axis as usize {
            sum += p.calc_size[axis];
        } else {
            sum = f32::max(sum, p.calc_size[axis]);
        }

        node = p.next();
    }

    root.calc_size[axis] = sum;
}

fn solve_size_violations(root: &mut BoxArea, axis: usize) {
    let available_space = root.calc_size[axis];

    let mut taken_space = 0.0;
    let mut total_fixup_budget = 0.0;
    let mut children: SmallVec<[BoxAreaPtr; 256]> = SmallVec::new();
    let mut non_floating_children: SmallVec<[BoxAreaPtr; 128]> = SmallVec::new();

    let mut node = root.first_mut();

    while let Some(p) = node {
        if !p.is_floating_on(axis as _) {
            non_floating_children.push(BoxAreaPtr::new(p));
        }

        children.push(BoxAreaPtr::new(p));

        node = p.next_mut();
    }

    if !root.is_overflowing_on(axis as u32) {
        for p in &non_floating_children {
            let p = p.as_ref_unsafe();
            let child_axis_size = p.calc_size[axis];
            if p.child_layout_axis as usize == axis {
                taken_space += child_axis_size;
            } else {
                taken_space = f32::max(taken_space, child_axis_size);
            }

            let fixup_budget_this_child = child_axis_size * (1.0 - p.pref_size[axis].strictness);
            total_fixup_budget += fixup_budget_this_child;
        }
    }

    if !root.is_overflowing_on(axis as u32) {
        let violation = taken_space - available_space;
        if violation > 0.0 && total_fixup_budget > 0.0 {
            for p in &non_floating_children {
                let p = p.as_mut_unsafe();
                let fixup_budget_this_child =
                    p.calc_size[axis] * (1.0 - p.pref_size[axis].strictness);

                let fixup_size_this_child = if root.child_layout_axis as usize == axis {
                    fixup_budget_this_child * (violation / total_fixup_budget)
                } else {
                    p.calc_size[axis] - available_space
                }
                .clamp(0.0, fixup_budget_this_child);

                p.calc_size[axis] -= fixup_size_this_child;
            }
        }
    }

    if root.child_layout_axis as usize == axis {
        let mut cur_pos = 0.0;
        for child in &non_floating_children {
            let child = child.as_mut_unsafe();
            child.calc_rel_position[axis] = cur_pos;
            cur_pos += child.calc_size[axis];
        }
    } else {
        for child in &non_floating_children {
            let child = child.as_mut_unsafe();
            // TODO: Validate
            child.calc_rel_position[axis] = 0.0;

            /*
            if ci.fill_implicit_layout_axis {
                ci.calc_size[axis] = inner.calc_size[axis];
            }
            */
        }
    }

    for child in &children {
        let child = child.as_mut_unsafe();
        let parent_pos = if child.is_floating_on(axis as _) {
            0.0
        } else {
            root.rect.min[axis]
        };

        child.rect.min[axis] = parent_pos + child.calc_rel_position[axis] - root.view_off[axis];
        child.rect.max[axis] = child.rect.min[axis] + child.calc_size[axis];
    }

    for child in &children {
        let child = child.as_mut_unsafe();
        solve_size_violations(child, axis);
    }
}

fn top_font_size() -> f32 {
    // Dummy function to represent the top font size
    16.0
}

impl Paint {
    fn measure_text(&self, text: &str) -> f32 {
        // Dummy function to represent text measurement
        text.len() as f32 * 8.0
    }
}

impl Layout {
    pub fn new() -> Result<Self, Error> {
        let reserve_size = 1024 * 1024 * 1024;

        Ok(Self {
            owner: PodArena::new(reserve_size)?,
            pref_width: PodArena::new(reserve_size)?,
            pref_height: PodArena::new(reserve_size)?,
            fixed_x: PodArena::new(reserve_size)?,
            fixed_y: PodArena::new(reserve_size)?,
            flags: PodArena::new(reserve_size)?,
            child_layout_axis: PodArena::new(reserve_size)?,
        })
    }

    pub fn resolve_layout(&mut self, root: BoxAreaPtr) {
        do_layout_axis(root.as_mut_unsafe());
    }
}

pub struct LayoutScope<'a> {
    core: &'a mut Flowi,
    used_stacks: StackFlags,
}

impl<'a> LayoutScope<'a> {
    pub fn new(core: &'a mut Flowi) -> Self {
        Self {
            core,
            used_stacks: StackFlags::empty(),
        }
    }

    pub fn set_pref_width(mut self, size: Size) -> Self {
        self.core.layout.pref_width.push(size);
        self.used_stacks |= StackFlags::PREF_WIDTH;
        self
    }

    pub fn set_pref_height(mut self, size: Size) -> Self {
        self.core.layout.pref_height.push(size);
        self.used_stacks |= StackFlags::PREF_HEIGHT;
        self
    }

    pub fn set_fixed_x(mut self, value: f32) -> Self {
        self.core.layout.fixed_x.push(value);
        self.used_stacks |= StackFlags::FIXED_WIDTH;
        self
    }

    pub fn set_fixed_y(mut self, value: f32) -> Self {
        self.core.layout.fixed_y.push(value);
        self.used_stacks |= StackFlags::FIXED_HEIGHT;
        self
    }

    pub fn set_flags(mut self, flags: u64) -> Self {
        self.core.layout.flags.push(flags);
        self.used_stacks |= StackFlags::FLAGS;
        self
    }

    pub fn set_child_layout_axis(mut self, axis: Axis) -> Self {
        self.core.layout.child_layout_axis.push(axis);
        self.used_stacks |= StackFlags::CHILD_LAYOUT_AXIS;
        self
    }

    // The `appyl` method takes a closure, runs it, and ensures the layout state is restored after
    pub fn apply<F>(&mut self, f: F)
    where
        F: FnOnce(),
    {
        f();
    }
}

impl<'a> Drop for LayoutScope<'a> {
    fn drop(&mut self) {
        if self.used_stacks.contains(StackFlags::PREF_WIDTH) {
            self.core.layout.pref_width.pop();
        }

        if self.used_stacks.contains(StackFlags::PREF_HEIGHT) {
            self.core.layout.pref_height.pop();
        }

        if self.used_stacks.contains(StackFlags::FIXED_WIDTH) {
            self.core.layout.fixed_x.pop();
        }

        if self.used_stacks.contains(StackFlags::FIXED_HEIGHT) {
            self.core.layout.fixed_y.pop();
        }

        if self.used_stacks.contains(StackFlags::FLAGS) {
            self.core.layout.flags.pop();
        }

        if self.used_stacks.contains(StackFlags::CHILD_LAYOUT_AXIS) {
            self.core.layout.child_layout_axis.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_recursive(node: &BoxArea, count: &mut usize, level: usize) {
        *count += 1;

        let mut node = node.first();

        while let Some(p) = node {
            count_recursive(p, count, level + 1);
            node = p.next();
        }
    }

    #[test]
    fn test_tree() {
        let mut layout = Layout::new().unwrap();
        /*
        layout.pref_width.push(Size::in_pixels(100.0));
        layout.pref_height.push(Size::in_pixels(100.0));
        layout.fixed_x.push(0.0);
        layout.fixed_y.push(0.0);
        layout.flags.push(0);
        layout.child_layout_axis.push(Axis::Horizontal);

        layout.create_root();
        */
        layout.create_box_with_string("1"); // 0
        layout.create_box_with_string("2"); // 1
        layout.create_box_with_string("3"); // 2
                                            //layout.owner.push(2);
                                            //layout.create_box_with_string("3"); // 3
                                            //layout.create_box_with_string("4"); // 4

        let mut count = 0;
        count_recursive(layout.root.as_ref_unsafe(), &mut count, 0);

        assert_eq!(count, 4);
    }

    #[test]
    fn basic_layout() {
        /*
        let mut layout = Layout::new();
        layout.pref_width.push(Size::in_pixels(100.0));
        layout.pref_height.push(Size::in_pixels(100.0));
        layout.fixed_x.push(0.0);
        layout.fixed_y.push(0.0);
        layout.flags.push(0);
        layout.child_layout_axis.push(Axis::Horizontal);

        layout.create_root();
        layout.resolve_layout();
        */
    }
}
