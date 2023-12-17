use flowi::window::{Window, WindowFlags};
use flowi::math_data::Vec2;
use flowi::ui::Ui;
use flowi::layout::Cursor;
use flowi::button::Button;

const MENU_SELECTION_NAMES_COUNT: usize = 5;

enum MenuSelection {
    None,
    Systems,
    Games,
    Demos,
    Settings,
    Debug,
}

static MENU_SELECTION_NAMES: [&str; MENU_SELECTION_NAMES_COUNT] = ["Systems", "Games", "Demos", "Settings", "Debug"];

pub struct LeftSideMenu {
    selection: MenuSelection,
    positions: Vec<Vec2>,
    width: u32,
    height: u32,
}

impl LeftSideMenu {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            selection: MenuSelection::None,
            positions: vec![Vec2 { x: 0.0, y: 0.0 }; MENU_SELECTION_NAMES_COUNT],
            width,
            height,
        }
    }

    /// Update the size of the left side menu when the window is resized
    pub fn update_size(&mut self, width: u32, height: u32, width_ratio: f32, height_ratio: f32) {
        let mut items_sizes = [Vec2 { x: 0.0, y: 0.0 }; MENU_SELECTION_NAMES_COUNT];

        let x_margin = 10.0 * width_ratio;
        let x_icon_start = 10.0 * width_ratio; 
        let x_text_start = 10.0 * width_ratio;

        // TODO: Add to config
        let spacing_between_items = 10.0 * height_ratio;
        let mut total_height = 0.0f32;
        let mut max_text_width = 0.0f32;

        // Get the size of each menu item
        for i in 0..MENU_SELECTION_NAMES.len() { 
            items_sizes[i] = Ui::calc_text_size(MENU_SELECTION_NAMES[i]);
            total_height += items_sizes[i].y + spacing_between_items;
            max_text_width = max_text_width.max(items_sizes[i].x);
        }

        let text_start = 10.0;//x_icon_start + max_text_width + x_text_start;
        let total_width = x_margin + x_icon_start + x_text_start + max_text_width;
        let items_starting_y = (height as f32 - total_height) / 2.0f32;

        for i in 0..MENU_SELECTION_NAMES.len() {
            self.positions[i].x = text_start;
            self.positions[i].y = items_starting_y + i as f32 * (items_sizes[i].y + spacing_between_items);
        }

        self.width = total_width as u32;
        self.height = height as u32;
    }

    pub fn show(&mut self) {
        Window::set_pos(Vec2 { x: 0.0, y: 0.0 });
        Window::set_size(Vec2 { x: self.width as _, y: self.height as _ });

        Window::begin("left_side_menu", WindowFlags::NO_DECORATION);

        for i in 0..MENU_SELECTION_NAMES.len() {
            Cursor::set_pos(self.positions[i]);
            Button::regular(MENU_SELECTION_NAMES[i]);
        }

        Window::end();
    }
}



