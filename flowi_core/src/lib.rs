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
use layout::{Layout, Size, Axis};
use box_area::{BoxArea, BoxAreaPtr};
use fileorama::Fileorama;
pub use io_handler::IoHandler;

pub struct Flowi {
    pub(crate) vfs: Fileorama,
    pub(crate) io_handler: IoHandler,
    pub(crate) layout: Layout,
    pub(crate) root: BoxAreaPtr,
    pub(crate) boxes: Arena,
    pub(crate) owner: PodArena<BoxAreaPtr>,
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
            root,
        })
    }

    fn create_box_inner(&mut self, parent: BoxAreaPtr) -> BoxAreaPtr {
        let box_area = self.boxes.alloc_init_ptr::<BoxArea>().unwrap();
        let box_area_ptr = BoxAreaPtr::new(box_area);

        // safe as the allocation above is guaranteed to be valid
        let box_area = box_area_ptr.as_mut_unchecked();  

        let inner = box_area.inner_borrow_mut();
        inner.pref_size[0] = self.layout.pref_width.last_or_default();
        inner.pref_size[1] = self.layout.pref_height.last_or_default(); 
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
        let parent_box = self.owner.last().unwrap().clone();
        let box_area = self.create_box_inner(parent_box); 
        let parent_box = parent_box.as_mut().unwrap();

        if let Some(p) = parent_box.last_mut() {
            p.next = box_area;
        } else {
            parent_box.first = box_area;
        }

        parent_box.last = box_area;

        let box_area = box_area.as_mut_unchecked();

        let inner = box_area.inner_borrow_mut();
        inner.display_string = display_string.to_string();
    }

    fn hash_from_string(seed: u64, string: &str) -> u64 {
        string.bytes().fold(seed, |result, byte| {
            (result << 5) + result + byte as u64
        })
    }
}
