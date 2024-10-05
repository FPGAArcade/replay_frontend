mod box_area;
mod image;
mod image_api;
pub mod input;
mod internal_error;
mod io_handler;
pub mod layout;
pub mod primitives;
pub mod signal;
pub mod widgets;

use arena_allocator::{Arena, PodArena};
use box_area::{BoxArea, BoxAreaPtr};
use fileorama::Fileorama;
pub use io_handler::IoHandler;
use layout::{Axis, Layout, LayoutScope, Size};
use primitives::{Color32, Primitive};
use std::collections::HashMap;
use signal::Signal;

type FlowiKey = u64;

pub(crate) struct Flowi {
    pub(crate) vfs: Fileorama,
    pub(crate) io_handler: IoHandler,
    pub(crate) layout: Layout,
    pub(crate) root: BoxAreaPtr,
    pub(crate) boxes: Arena,
    pub(crate) primitives: Arena,
    pub(crate) hot_item: FlowiKey,
    // Used to look up if we have a box created for a given hash key
    // TODO: Rewrite with custom hash map to get rid of std dependency
    pub(crate) box_lookup: HashMap<u64, BoxAreaPtr>,
    pub(crate) current_frame: u64,
    // If we build in debug and don't have the instance_thread_local feature enabled we
    // so we can validate that the instance is only accessed from the main thread
    #[cfg(all(debug_assertions, not(feature = "instance_thread_local")))]
    pub(crate) thread_id: std::thread::ThreadId,
}

impl Flowi {
    pub fn new() -> Box<Self> {
        let vfs = Fileorama::new(2);
        let io_handler = IoHandler::new(&vfs);
        crate::image_api::install_image_loader(&vfs);

        let reserve_size = 1024 * 1024 * 1024;

        let mut layout = Layout::new().unwrap();
        let mut box_allocator = Arena::new(reserve_size).unwrap();

        let root = Self::create_root(&mut box_allocator);
        layout.owner.push(root);

        Box::new(Flowi {
            vfs,
            io_handler,
            layout,
            hot_item: 0,
            boxes: box_allocator,
            box_lookup: HashMap::new(),
            current_frame: 0,
            primitives: Arena::new(reserve_size).unwrap(),
            root,
            #[cfg(all(debug_assertions, not(feature = "instance_thread_local")))]
            thread_id: std::thread::current().id(),
        })
    }

    pub fn begin(&mut self, _delta_time: f32, width: usize, height: usize) {
        // Set the root box to the size of the window
        let root = self.root.as_mut_unsafe(); 

        self.layout.pref_width.push(Size::in_pixels(width as f32));
        self.layout.pref_height.push(Size::in_pixels(height as f32));

        root.pref_size[0] = Size::in_pixels(width as f32);
        root.pref_size[1] = Size::in_pixels(height as f32);

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
            let color = Color32::new(0xff, 0xff, 0xff, 0xff);
            *primitive = Primitive::new(box_area.rect, color);
        }
    }

    fn create_box_inner(&mut self, parent: BoxAreaPtr) -> BoxAreaPtr {
        let box_area = self.boxes.alloc_init_ptr::<BoxArea>().unwrap();
        let box_area_ptr = BoxAreaPtr::new(box_area);
        let box_area = box_area_ptr.as_mut_unsafe(); // safe as the allocation above is guaranteed to be valid

        box_area.pref_size[0] = self.layout.pref_width.last_or_default();
        box_area.pref_size[1] = self.layout.pref_height.last_or_default();
        //dbg!(inner.pref_size[0]);
        //dbg!(inner.pref_size[1]);
        box_area.pref_size[0] = Size::in_pixels(100.0);
        box_area.pref_size[1] = Size::in_pixels(100.0);
        box_area.calc_rel_position[0] = self.layout.fixed_x.last_or_default();
        box_area.calc_rel_position[1] = self.layout.fixed_y.last_or_default();
        box_area.flags = self.layout.flags.last_or_default();
        box_area.child_layout_axis = self.layout.child_layout_axis.last_or_default();
        box_area.parent = parent;

        box_area_ptr
    }

    fn create_root(allocator: &mut Arena) -> BoxAreaPtr {
        let box_area = allocator.alloc_init::<BoxArea>().unwrap();

        box_area.pref_size[0] = Size::in_pixels(100.0);
        box_area.pref_size[1] = Size::in_pixels(100.0);
        box_area.calc_rel_position[0] = 0.0;
        box_area.calc_rel_position[1] = 0.0;
        box_area.flags = 0;
        box_area.child_layout_axis = Axis::Horizontal;
        box_area.display_string = "root".to_string();
        box_area.hash_key = Self::hash_from_string(0, "root");

        BoxAreaPtr::new(box_area)
    }

    pub fn create_box(&mut self) {
        let parent_box = self.layout.owner.last().copied().unwrap_or_default();

        let box_area = self.create_box_inner(parent_box);
        let parent_box = parent_box.as_mut().unwrap();

        if let Some(p) = parent_box.last_mut() {
            p.next = box_area;
        } else {
            parent_box.first = box_area;
        }

        parent_box.last = box_area;
    }

    pub fn create_box_with_string(&mut self, display_string: &str) -> BoxAreaPtr {
        let parent_box = self.layout.owner.last_or_default();
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

        let b = box_area;
        let box_area = box_area.as_mut_unsafe();

        // clear out the per-frame data
        box_area.parent = BoxAreaPtr::new(parent_box);
        box_area.first = BoxAreaPtr::default();
        box_area.last = BoxAreaPtr::default();
        box_area.next = BoxAreaPtr::default();

        // TODO: String allocator
        box_area.display_string = display_string.to_string();

        b
    }

    pub fn button(&mut self, text: &str) -> Signal {
        let box_area = self.create_box_with_string(text);
        self.signal(box_area)
    }

    fn signal(&mut self, box_area: BoxAreaPtr) -> Signal {
        let signal = Signal::new();
        let _box_area = box_area.as_mut_unsafe();
        signal
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

#[cfg(feature = "instance_thread_local")]
thread_local! {
    pub static UI_INSTANCE: UnsafeCell<Flowi> = UnsafeCell::new(Flowi::new());
}

#[cfg(not(feature = "instance_thread_local"))]
static mut UI_INSTANCE: *mut Flowi = core::ptr::null_mut();

#[cfg(feature = "instance_thread_local")]
fn ui_instance() -> &mut Flowi {
    UI_INSTANCE.with(|ui| unsafe { &mut *ui.get() })
}

#[cfg(not(feature = "instance_thread_local"))]
fn ui_instance<'a>() -> &'a mut Flowi {
    #[cfg(debug_assertions)]
    if unsafe { UI_INSTANCE.is_null() } {
        panic!("UI instance accessed before it was created");
    }

    let instance = unsafe { &mut *UI_INSTANCE };

    // If we debug without thread_local we make sure that the instance is only accessed from the main thread
    #[cfg(debug_assertions)]
    if !instance.thread_id.eq(&std::thread::current().id()) {
        panic!("UI instance accessed from a different thread than the one it was created on");
    }

    instance
}

pub struct Ui;

impl Ui {
    #[cfg(not(feature = "instance_thread_local"))]
    pub fn create() {
        unsafe {
            UI_INSTANCE = Box::into_raw(Flowi::new());
        }
    }

    #[cfg(feature = "instance_thread_local")]
    pub fn create() {}

    pub fn begin(delta_time: f32, width: usize, height: usize) {
        ui_instance().begin(delta_time, width, height);
    }

    pub fn end() {
        ui_instance().end();
    }

    pub fn with_layout<'a>() -> LayoutScope<'a> {
        ui_instance().with_layout()
    }

    pub fn create_box() {
        ui_instance().create_box();
    }

    pub fn create_box_with_string(display_string: &str) -> BoxAreaPtr {
        ui_instance().create_box_with_string(display_string)
    }

    pub fn primitives<'a>() -> &'a [Primitive] {
        ui_instance().primitives()
    }
}
