mod image;
mod image_api;
mod internal_error;
mod io_handler;
mod box_area;
pub mod layout;
pub mod primitives;
pub mod input;
pub mod widgets;

use arena_allocator::{PodArena, Arena};
use layout::{Layout, LayoutScope, Size, Axis};
use box_area::{BoxArea, BoxAreaPtr};
use std::collections::HashMap;
use fileorama::Fileorama;
pub use io_handler::IoHandler;
use primitives::{Primitive, Color32};
use crate::box_area::Rect;

pub struct Flowi {
    pub(crate) vfs: Fileorama,
    pub(crate) io_handler: IoHandler,
    pub(crate) layout: Layout,
    pub(crate) root: BoxAreaPtr,
    pub(crate) boxes: Arena,
    pub(crate) owner: PodArena<BoxAreaPtr>,
    pub(crate) primitives: Arena,
    // Used to look up if we have a box created for a given hash key
    // TODO: Rewrite with custom hash map to get rid of std dependency
    box_lookup: HashMap<u64, BoxAreaPtr>,
    current_frame: u64,
}

impl Flowi {
    pub fn new() -> Box<Self> {
        let vfs = Fileorama::new(2);
        let io_handler = IoHandler::new(&vfs);
        crate::image_api::install_image_loader(&vfs);

        let reserve_size = 1024 * 1024 * 1024;

        let mut owner = PodArena::new(reserve_size).unwrap();
        let mut box_allocator = Arena::new(reserve_size).unwrap();
        let root = Self::create_root(&mut box_allocator);
        owner.push(root);

        Box::new(Flowi {
            vfs,
            owner,
            io_handler,
            layout: Layout::new().unwrap(),
            boxes: box_allocator, 
            box_lookup: HashMap::new(),
            current_frame: 0,
            primitives: Arena::new(reserve_size).unwrap(),
            root,
        })
    }

    pub fn begin(&mut self, _delta_time: f32, width: usize, height: usize) {
        // Set the root box to the size of the window
        let root = self.root.as_mut_unchecked();
        let inner = root.inner_borrow_mut();

        self.layout.pref_width.push(Size::in_pixels(width as f32));
        self.layout.pref_height.push(Size::in_pixels(height as f32));

        inner.pref_size[0] = Size::in_pixels(width as f32);
        inner.pref_size[1] = Size::in_pixels(height as f32);

        self.io_handler.update();
        self.primitives.rewind();
    }

    pub fn end(&mut self) {
        // Calculate the layout of all boxes
        self.layout.resolve_layout(self.root);

        // Only retain boxes that were created/updated in the current frame
        /*
        self.box_lookup.retain(|_, box_area| {
            let box_area = box_area.as_ref_unsafe();
            box_area.current_frame == self.current_frame
        });
        */

        // Generate primitives from all boxes
        self.generate_primitives();
        self.current_frame += 1;
    }

    pub fn with_layout(&mut self) -> LayoutScope { 
        LayoutScope::new(self)
    }

    fn generate_primitives(&mut self) {
        // TODO: We should prune the tree of boxes that wasn't created the current frame
        for box_area in self.boxes.get_array_by_type::<BoxArea>() {
            let primitive = unsafe { self.primitives.alloc::<Primitive>().unwrap() };
            let inner = box_area.inner_borrow();
            let rect = inner.rect;
            let color = Color32::new(0xff, 0xff, 0xff, 0xff);
            *primitive = Primitive::new(rect, color);
        }
    }

    fn create_box_inner(&mut self, parent: BoxAreaPtr) -> BoxAreaPtr {
        let box_area = self.boxes.alloc_init_ptr::<BoxArea>().unwrap();
        let box_area_ptr = BoxAreaPtr::new(box_area);

        // safe as the allocation above is guaranteed to be valid
        let box_area = box_area_ptr.as_mut_unchecked();  

        let inner = box_area.inner_borrow_mut();
        inner.pref_size[0] = self.layout.pref_width.last_or_default();
        inner.pref_size[1] = self.layout.pref_height.last_or_default(); 
        dbg!(inner.pref_size[0]);
        dbg!(inner.pref_size[1]);
        //inner.pref_size[0] = Size::in_pixels(100.0);
        //inner.pref_size[1] = Size::in_pixels(100.0);
        inner.calc_rel_position[0] = self.layout.fixed_x.last_or_default();
        inner.calc_rel_position[1] = self.layout.fixed_y.last_or_default();
        inner.flags = self.layout.flags.last_or_default();
        inner.child_layout_axis = self.layout.child_layout_axis.last_or_default();
        box_area.parent = parent;

        box_area_ptr
    }

    fn create_root(allocator: &mut Arena) -> BoxAreaPtr {
        let box_area = allocator.alloc_init::<BoxArea>().unwrap(); 
        let inner = box_area.inner_borrow_mut();

        inner.pref_size[0] = Size::in_pixels(100.0);
        inner.pref_size[1] = Size::in_pixels(100.0);
        inner.calc_rel_position[0] = 0.0;
        inner.calc_rel_position[1] = 0.0;
        inner.flags = 0;
        inner.child_layout_axis = Axis::Horizontal;
        inner.display_string = "root".to_string();
        box_area.hash_key = Self::hash_from_string(0, "root");
        
        BoxAreaPtr::new(box_area)
    }

    pub fn create_box(&mut self) {
        let parent_box = self.owner.last().copied().unwrap_or_default();

        let box_area = self.create_box_inner(parent_box); 
        let parent_box = parent_box.as_mut().unwrap();

        if let Some(p) = parent_box.last_mut() {
            p.next = box_area;
        } else {
            parent_box.first = box_area;
        }
    
        parent_box.last = box_area;
    }

    pub fn create_box_with_string(&mut self, display_string: &str) {
        let parent_box = self.owner.last_or_default();
        let hash = Self::hash_from_string(parent_box.as_ref_unsafe().hash_key, display_string);

        let box_area = if let Some(box_area) = self.box_lookup.get(&hash) {
            *box_area
        } else {
            let box_area = self.create_box_inner(parent_box);
            self.box_lookup.insert(hash, box_area);
            box_area
        };

        let parent_box = parent_box.as_mut().unwrap();

        if let Some(p) = parent_box.last_mut() {
            p.next = box_area;
        } else {
            parent_box.first = box_area;
        }

        parent_box.last = box_area;

        let box_area = box_area.as_mut_unchecked();

        // clear out the per-frame data
        box_area.parent = BoxAreaPtr::new(parent_box);
        box_area.first = BoxAreaPtr::default();
        box_area.last = BoxAreaPtr::default();
        box_area.next = BoxAreaPtr::default();

        let inner = box_area.inner_borrow_mut();
        inner.display_string = display_string.to_string();
    }

    pub fn primitives(&self) -> &[Primitive] {
        self.primitives.get_array_by_type::<Primitive>()
    }

    fn hash_from_string(seed: u64, string: &str) -> u64 {
        string.bytes().fold(seed, |result, byte| {
            (result << 5).wrapping_add(result + byte as u64)
        })
    }
}
