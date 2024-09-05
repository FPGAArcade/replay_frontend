use smallvec::SmallVec;
use arena_allocator::TypedArena;
use crate::box_area::BoxArea;
use crate::box_area::StackFlags;
use crate::internal_error::InternalError as Error;

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
fn do_layout_axis(root: &BoxArea, boxes: &[BoxArea]) {
    do_layout_for(root, boxes, 0);
    do_layout_for(root, boxes, 1);
}

#[cfg_attr(feature = "tracing-instrument", tracing::instrument)]
fn do_layout_for(root: &BoxArea, boxes: &[BoxArea], axis: usize) {
    solve_independent_sizes_for(root, boxes, axis);
    solve_downward_dependent_sizes_for(root, boxes, axis);
    solve_upward_dependent_sizes_for(root, boxes, axis);
    solve_downward_dependent_sizes_for(root, boxes, axis);
    solve_size_violations(root, boxes, axis);
}

fn solve_independent_sizes_for(root: &BoxArea, boxes: &[BoxArea], axis: usize) {
    let inner = root.inner_borrow_mut(); 
    let size = inner.pref_size[axis];

    match size.kind {
        SizeKind::Pixels => {
            inner.calc_size[axis] = size.value;
        }
        SizeKind::Em => {
            inner.calc_size[axis] = (size.value * top_font_size()).floor();
        }
        SizeKind::TextContent => {
            if let Some(text_data) = &inner.text_data {
                //let paint = &text_data.paint;
                if axis == 0 {
                    // TODO: fix me
                    let text_width = 64.0;//paint.measure_text(&text_data.display_text);
                    let text_padding = text_data.text_edge_padding;
                    inner.calc_size[axis] = (text_width + 2.0 * text_padding).floor();
                } else {
                    // TODO: fix me
                    //let metrics = &paint.font_metrics;
                    let line_height = 40.0f32;//metrics.descent - metrics.top;
                    inner.calc_size[axis] = line_height.floor();
                }
            } else {
                panic!("box.text_data should not be None");
            }
        }
        _ => {}
    }

    let mut node = root.first(boxes);
    while let Some(p) = node {
        solve_independent_sizes_for(p, boxes, axis);
        node = p.next(boxes);
    }
}

fn solve_upward_dependent_sizes_for(root: &BoxArea, boxes: &[BoxArea], axis: usize) {
    let inner = root.inner_borrow_mut(); 
    let size = inner.pref_size[axis];

    if size.kind != SizeKind::PercentOfAncestor {
        return;
    }

    let mut ancestor: Option<&BoxArea> = None;
    let mut parent = root.parent(boxes);

    while let Some(p) = parent {
        let p_borrowed = p.inner_borrow();
        if p_borrowed.pref_size[axis].kind != SizeKind::ChildrenSum {
            ancestor = Some(p);
            break;
        }
        parent = p.parent(boxes);
    }

    if let Some(a) = ancestor {
        inner.calc_size[axis] = a.inner_borrow().calc_size[axis] * size.value;
    } else {
        println!("{} is left out of size calculations!", inner.display_string);
    }
}

fn solve_downward_dependent_sizes_for(root: &BoxArea, boxes: &[BoxArea], axis: usize) {
    let mut node = root.first(boxes);
    while let Some(next) = node {
        solve_downward_dependent_sizes_for(next, boxes, axis);
        node = next.next(boxes);
    }

    let axis = axis & 1;
    let inner = root.inner_borrow_mut(); 
    let size = inner.pref_size[axis];

    if size.kind != SizeKind::ChildrenSum {
        return;
    }

    let mut node = root.first(boxes);
    let mut sum = 0.0;
    while let Some(p) = node {
        let pi = p.inner_borrow(); 

        if axis == inner.child_layout_axis as usize {
            sum += pi.calc_size[axis];
        } else {
            sum = f32::max(sum, pi.calc_size[axis]);
        } 

        node = p.next(boxes); 
    }

    inner.calc_size[axis] = sum;

}

fn solve_size_violations(root: &BoxArea, boxes: &[BoxArea], axis: usize) {
    let inner = root.inner_borrow_mut();
    let available_space = inner.calc_size[axis];

    let mut taken_space = 0.0;
    let mut total_fixup_budget = 0.0;
    let mut children: SmallVec<[&BoxArea; 256]> = SmallVec::new();
    let mut non_floating_children: SmallVec<[&BoxArea; 128]> = SmallVec::new();
        
    let mut node = root.first(boxes);

    while let Some(p) = node {
        let pi = p.inner_borrow(); 

        if !pi.is_floating_on(axis as _) {
            non_floating_children.push(p);
        }

        children.push(p);

        node = p.next(boxes); 
    }

    if !inner.is_overflowing_on(axis as u32) {
        for p in &non_floating_children {
            let pi = p.inner_borrow(); 

            let child_axis_size = pi.calc_size[axis];
            if pi.child_layout_axis as usize == axis {
                taken_space += child_axis_size;
            } else {
                taken_space = f32::max(taken_space, child_axis_size);
            }

            let fixup_budget_this_child = child_axis_size * (1.0 - pi.pref_size[axis].strictness);
            total_fixup_budget += fixup_budget_this_child;
        }
    }

    if !inner.is_overflowing_on(axis as u32) {
        let violation = taken_space - available_space;
        if violation > 0.0 && total_fixup_budget > 0.0 {
            for p in &non_floating_children {
                let pi = p.inner_borrow_mut(); 

                let fixup_budget_this_child = pi.calc_size[axis] * (1.0 - pi.pref_size[axis].strictness);

                let fixup_size_this_child = if inner.child_layout_axis as usize == axis {
                    fixup_budget_this_child * (violation / total_fixup_budget)
                } else {
                    pi.calc_size[axis] - available_space
                }
                .clamp(0.0, fixup_budget_this_child);

                pi.calc_size[axis] -= fixup_size_this_child;
            }
        }
    }

    if inner.child_layout_axis as usize == axis {
        let mut cur_pos = 0.0;
        for child in &non_floating_children {
            let ci = child.inner_borrow_mut();
            ci.calc_rel_position[axis] = cur_pos;
            cur_pos += ci.calc_size[axis];
        }
    } else {
        for child in &non_floating_children {
            // TODO: Validate
            let ci = child.inner_borrow_mut();
            ci.calc_rel_position[axis] = 0.0;

            /* 
            if ci.fill_implicit_layout_axis {
                ci.calc_size[axis] = inner.calc_size[axis];
            }
            */
        }
    }

    for child in &children {
        let ci = child.inner_borrow_mut();
        let parent_pos = if ci.is_floating_on(axis as _) {
            0.0
        } else {
            inner.rect.min[axis]
        };

        ci.rect.min[axis] = parent_pos + ci.calc_rel_position[axis] - inner.view_off[axis];
        ci.rect.max[axis] = ci.rect.min[axis] + ci.calc_size[axis];
    }

    for child in &children {
        solve_size_violations(child, boxes, axis);
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

pub struct Layout {
    // TODO: Change to arenas
    /*
    owner: TypedArena<usize>,
    pref_width: TypedArena<Size>,
    pref_height: TypedArena<Size>,
    fixed_x: TypedArena<f32>,
    fixed_y: TypedArena<f32>,
    flags: TypedArena<u32>,
    child_layout_axis: TypedArena<Axis>,
    root: usize,
    boxes: TypedArena<BoxArea>,
    */
    owner: Vec<usize>,
    pref_width: Vec<Size>,
    pref_height: Vec<Size>,
    fixed_x: Vec<f32>,
    fixed_y: Vec<f32>,
    flags: Vec<u64>,
    child_layout_axis: Vec<Axis>,
    root: usize,
    boxes: Vec<BoxArea>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            owner: Vec::new(),
            pref_width: Vec::new(),
            pref_height: Vec::new(),
            fixed_x: Vec::new(),
            fixed_y: Vec::new(),
            flags: Vec::new(),
            child_layout_axis: Vec::new(),
            root: 0,
            boxes: Vec::new(),
        }
    }

    pub fn create_box(&mut self) {
        let parent_index = self.owner.last().copied().unwrap_or_default();
        let box_area = self.create_box_inner(parent_index); 

        if let Some(p) = self.boxes[parent_index].last {
            let p = &mut self.boxes[p];
            p.next = Some(box_area);
        } else {
            self.boxes[parent_index].first = Some(box_area);
        }
    
        self.boxes[parent_index].last = Some(box_area);
    }

    pub fn create_box_with_string(&mut self, display_string: &str) {
        let parent_index = self.owner.last().copied().unwrap_or_default();
        let box_area = self.create_box_inner(parent_index); 
        //let parent = &mut self.boxes[parent_index];

        if let Some(p) = self.boxes[parent_index].last {
            let p = &mut self.boxes[p];
            p.next = Some(box_area);
        } else {
            self.boxes[parent_index].first = Some(box_area);
        }
    
        self.boxes[parent_index].last = Some(box_area);

        let inner = self.boxes.last_mut().unwrap().inner_borrow_mut();
        inner.display_string = display_string.to_string();
    }

    pub fn create_box_inner(&mut self, parent_index: usize) -> usize {
        let index = self.boxes.len();
        self.boxes.push(BoxArea::default());

        //let parent = &mut self.boxes[parent_index];
        let box_area = self.boxes.last_mut().unwrap();
        let inner = box_area.inner_borrow_mut();
        // TODO: Optimize
        inner.pref_size[0] = self.pref_width.last().copied().unwrap_or_default(); 
        inner.pref_size[1] = self.pref_height.last().copied().unwrap_or_default(); 
        inner.calc_rel_position[0] = self.fixed_x.last().copied().unwrap_or_default();
        inner.calc_rel_position[1] = self.fixed_y.last().copied().unwrap_or_default();
        inner.flags = self.flags.last().copied().unwrap_or_default();
        inner.child_layout_axis = self.child_layout_axis.last().copied().unwrap_or_default();
        box_area.parent = Some(parent_index);
        index
    }

    pub fn create_root(&mut self) {
        let box_area = BoxArea::default();
        let inner = box_area.inner_borrow_mut();

        inner.pref_size[0] = Size::in_pixels(100.0);
        inner.pref_size[1] = Size::in_pixels(100.0);
        inner.calc_rel_position[0] = 0.0;
        inner.calc_rel_position[1] = 0.0;
        inner.flags = 0;
        inner.child_layout_axis = Axis::Horizontal;
        inner.display_string = "root".to_string();

        self.boxes.push(box_area);
    }

    pub fn resolve_layout(&mut self) {
        do_layout_axis(&self.boxes[self.root], &self.boxes);
    }

}




pub struct LayoutScope<'a> {
    layout: &'a mut Layout,
    used_stacks: StackFlags,
}

impl<'a> LayoutScope<'a> {
    pub fn new(layout: &'a mut Layout) -> Self {
        Self {
            layout,
            used_stacks: StackFlags::empty(),
        }
    }

    pub fn set_pref_width(&mut self, size: Size) -> &mut Self {
        self.layout.pref_width.push(size);
        self.used_stacks |= StackFlags::PREF_WIDTH;
        self
    }

    pub fn set_pref_height(&mut self, size: Size) {
        self.layout.pref_height.push(size);
        self.used_stacks |= StackFlags::PREF_HEIGHT;
    }

    pub fn set_fixed_x(&mut self, value: f32) {
        self.layout.fixed_x.push(value);
        self.used_stacks |= StackFlags::FIXED_WIDTH;
    }

    pub fn set_fixed_y(&mut self, value: f32) {
        self.layout.fixed_y.push(value);
        self.used_stacks |= StackFlags::FIXED_HEIGHT;
    }

    pub fn set_flags(&mut self, flags: u64) {
        self.layout.flags.push(flags);
        self.used_stacks |= StackFlags::FLAGS;
    }

    pub fn set_child_layout_axis(&mut self, axis: Axis) {
        self.layout.child_layout_axis.push(axis);
        self.used_stacks |= StackFlags::CHILD_LAYOUT_AXIS;
    }

    pub fn end_box(&mut self) {
        self.used_stacks = StackFlags::empty();
    }
}

impl<'a> Drop for LayoutScope<'a> {
    fn drop(&mut self) {
        if self.used_stacks.contains(StackFlags::PREF_WIDTH) {
            self.layout.pref_width.pop();
        }

        if self.used_stacks.contains(StackFlags::PREF_HEIGHT) {
            self.layout.pref_height.pop();
        }

        if self.used_stacks.contains(StackFlags::FIXED_WIDTH) {
            self.layout.fixed_x.pop();
        }

        if self.used_stacks.contains(StackFlags::FIXED_HEIGHT) {
            self.layout.fixed_y.pop();
        }

        if self.used_stacks.contains(StackFlags::FLAGS) {
            self.layout.flags.pop();
        }

        if self.used_stacks.contains(StackFlags::CHILD_LAYOUT_AXIS) {
            self.layout.child_layout_axis.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_recursive(node: &BoxArea, boxes: &[BoxArea], count: &mut usize, level: usize) {
        *count += 1;
        
        let mut node = node.first(boxes);

        while let Some(p) = node {
            count_recursive(p, boxes, count, level + 1);
            node = p.next(boxes);
        }
    }

    #[test]
    fn test_tree() {
        let mut layout = Layout::new();
        layout.pref_width.push(Size::in_pixels(100.0));
        layout.pref_height.push(Size::in_pixels(100.0));
        layout.fixed_x.push(0.0);
        layout.fixed_y.push(0.0);
        layout.flags.push(0);
        layout.child_layout_axis.push(Axis::Horizontal);

        layout.create_root();
        layout.create_box_with_string("1"); // 0
        layout.create_box_with_string("2"); // 1
        layout.create_box_with_string("3"); // 2
        layout.owner.push(2);
        layout.create_box_with_string("3"); // 3
        layout.create_box_with_string("4"); // 4

        let mut count = 0;
        count_recursive(&layout.boxes[0], &layout.boxes, &mut count, 0);

        assert_eq!(count, 6);
    }

    #[test]
    fn basic_layout() {
        let mut layout = Layout::new();
        layout.pref_width.push(Size::in_pixels(100.0));
        layout.pref_height.push(Size::in_pixels(100.0));
        layout.fixed_x.push(0.0);
        layout.fixed_y.push(0.0);
        layout.flags.push(0);
        layout.child_layout_axis.push(Axis::Horizontal);

        layout.create_root();
        layout.resolve_layout();



    }
}


