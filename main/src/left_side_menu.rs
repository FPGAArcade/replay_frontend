use flowi::{
    image::{Image, ImageLoadStatus, ImageOptions},
    layout::Cursor,
    font::Font,
    painter::Painter,
    text::Text,
    math_data::{Vec2, IVec2},
    ui::Ui,
    window::{Window, WindowFlags},
    Color,
};
    
use crate::Fonts;

const MENU_SELECTION_NAMES_COUNT: usize = 6;

#[derive(Copy, Clone)]
enum MenuSelection {
    Search,
    Systems,
    Games,
    Demos,
    Settings,
    Debug,
}

struct MenuItemInfo {
    selection_id: MenuSelection,
    text: &'static str,
    path: &'static str,
}

static MENU_ITEMS: [MenuItemInfo; MENU_SELECTION_NAMES_COUNT] = [
    MenuItemInfo {
        text: "Search",
        path: "data/svgs/icons8-search.svg",
        selection_id: MenuSelection::Search,
    },
    MenuItemInfo {
        text: "Systems",
        path: "data/svgs/core.svg",
        selection_id: MenuSelection::Systems,
    },
    MenuItemInfo {
        text: "Games",
        path: "data/svgs/games-buttons-svgrepo-com.svg",
        selection_id: MenuSelection::Games,
    },
    MenuItemInfo {
        text: "Demos",
        path: "data/svgs/cube-svgrepo-com.svg",
        selection_id: MenuSelection::Demos,
    },
    MenuItemInfo {
        text: "Settings",
        path: "data/svgs/settings-svgrepo-com.svg",
        selection_id: MenuSelection::Settings,
    },
    MenuItemInfo {
        text: "Debug",
        path: "data/svgs/bug-debug-fix-fixing-qa-svgrepo-com.svg",
        selection_id: MenuSelection::Debug,
    },
];

struct MenuItem {
    pos: Vec2,
    text_size: IVec2,
    selection_id: MenuSelection,
    text: &'static str,
    icon_path: &'static str,
    icon_pos: Vec2,
    icon_size: Vec2,
    icon: Image,
    color: Color,
}

enum State {
    CalculatingTextSizes,
    WatingForAssets,
    CalculateLayout,
    Ready,
}

pub struct LeftSideMenu {
    selection: MenuSelection,
    items: Vec<MenuItem>,
    logo: Image,
    logo_pos: Vec2,
    state: State,
    pub width: i32,
    pub height: i32,
    // margin to the left screen edge
    icons_left_margin: i32,
    // margin between the icons and the text
    icons_text_margin: i32,
}

impl LeftSideMenu {
    pub fn new(width: i32, height: i32) -> Self {
        let items = MENU_ITEMS
            .iter()
            .map(|item| MenuItem {
                pos: Vec2::default(),
                text_size: IVec2::default(),
                icon_pos: Vec2::default(),
                selection_id: item.selection_id,
                icon_path: item.path,
                text: item.text,
                icon: Image { handle: 0 },
                icon_size: Vec2::default(),
                color: Color::new(1.0, 1.0, 1.0, 1.0),
            })
            .collect::<Vec<_>>();

        Self {
            logo: Image::load("data/logo.png"),
            selection: MenuSelection::Systems,
            logo_pos: Vec2::default(),
            state: State::CalculatingTextSizes,
            items,
            width,
            height,
            icons_left_margin: 30,
            icons_text_margin: 30,
        }
    }

    // Calculate the text sizes so we know how large the icon images has to be.
    // We assume that the caller has loaded the font already at this point and will set it
    fn calculate_text_size(&mut self) {
        let mut options = ImageOptions::default();
        options.color = Color::new(1.0, 1.0, 1.0, 0.0); 

        for menu_item in &mut self.items {
            menu_item.text_size = Ui::calc_text_size(menu_item.text);

            // We only set the height of the image, the width will be calculated automatically to keep the aspect ratio
            options.size = IVec2::new(0, (menu_item.text_size.y as f32 * 0.7) as _);
        
            menu_item.icon = Image::load_with_options(menu_item.icon_path, &options);
        }

        self.state = State::WatingForAssets;
    }

    // Wait for all assets to be loaded
    fn wait_for_assets(&mut self) {
        // TODO: Handle error
        if Image::get_status(self.logo) == ImageLoadStatus::Loading {
            return;
        }

        for menu_item in &self.items {
            match Image::get_status(menu_item.icon) {
                ImageLoadStatus::Loading => return,
                ImageLoadStatus::Failed => panic!("Error loading image {}", menu_item.icon_path),
                ImageLoadStatus::Loaded => (),
            }
        }

        dbg!("All assets loaded");
        self.state = State::CalculateLayout;
    }

    fn calculate_layout(&mut self, width: i32, height: i32) -> flowi::Result<()> {
        let logo_info = Image::get_info(self.logo)?;
        let logo_size = IVec2::new(logo_info.width, logo_info.height);

        let mut max_width_icons = 0i32;
        let mut _icons_center_x = 0;
        let mut spacing_between_items = 14i32;
        let mut total_height = 0i32;
        let mut max_text_width = 0i32;

        // Get the size of each icon and also handle if the data is not loaded yet. We return false
        // from this function and it will be called again next frame until we returnt true.
        for menu_item in &mut self.items {
            let icon = Image::get_info(menu_item.icon)?;
            total_height += (menu_item.text_size.y as i32) + spacing_between_items;

            max_width_icons = max_width_icons.max(icon.width);
            max_text_width = max_text_width.max(menu_item.text_size.x as i32);

            menu_item.icon_size = Vec2::new(icon.width as f32, icon.height as f32); 

            _icons_center_x += icon.width;
        }

        _icons_center_x /= self.items.len() as i32;
        let x_icons_start = self.icons_left_margin;

        let text_start = x_icons_start + max_width_icons + self.icons_text_margin;

        let total_width = (x_icons_start + max_text_width + self.icons_text_margin + max_width_icons) + 40;
        let items_starting_y = (height - total_height) / 2;

        let mut y = items_starting_y;

        for menu_item in &mut self.items {
            // adjust y pos for the icon based on the text size
            let icon_y_offset = (menu_item.text_size.y - menu_item.icon_size.y as i32) / 2;

            menu_item.icon_pos.x = x_icons_start as _;
            menu_item.icon_pos.y = (icon_y_offset + y) as _;
            menu_item.pos.x = text_start as _;
            menu_item.pos.y = y as _;
            y += menu_item.text_size.y + spacing_between_items;
        }

        self.logo_pos.x = ((total_width - logo_size.x) / 2) as _;
        self.logo_pos.y = 4.0;
        self.width = (total_width + 40) as i32;
        self.height = height;

        self.state = State::Ready;

        Ok(())
    }

    fn draw(&mut self) {
        Window::set_pos(Vec2::new(0.0, 0.0));
        Window::set_size(Vec2::new(self.width as _, self.height as _));

        Window::begin("left_side_menu", WindowFlags::NO_DECORATION);

        Cursor::set_pos(self.logo_pos);
        Ui::image(self.logo);

        let start = Vec2::new(0.0, self.items[1].pos.y);
        let text_size = self.items[1].text_size;
        let end = Vec2::new(self.width as f32, start.y + text_size.y as f32); 

        Painter::draw_rect_filled(start, end, Color::new(0.1, 0.1, 0.1, 0.1), 0.0);

        for menu_item in &self.items {
            Cursor::set_pos(menu_item.icon_pos);
            Ui::image(menu_item.icon);
            Cursor::set_pos(menu_item.pos);
            Text::show(menu_item.text);
        }

        Window::end();
    }

    pub fn update(&mut self, fonts: &Fonts, width: i32, height: i32) -> bool {
        Font::push(fonts.default);

        let mut show_state = false;

        match self.state {
            State::CalculatingTextSizes => self.calculate_text_size(),
            State::WatingForAssets => self.wait_for_assets(),
            State::CalculateLayout => self.calculate_layout(width, height).unwrap(),
            State::Ready => {
                self.draw();
                show_state = true;
            }
        }

        Font::pop();

        show_state
    }
}
