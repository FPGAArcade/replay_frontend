use flowi::{
    button::Button,
    image::Image,
    image::ImageLoadStatus,
    image::ImageOptions,
    layout::Cursor,
    math_data::Vec2,
    ui::Ui,
    window::{Window, WindowFlags},
    Color,
};

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
        path: "data/svgs/search.svg",
        selection_id: MenuSelection::Search,
    },
    MenuItemInfo {
        text: "Systems",
        path: "data/svgs/home.svg",
        selection_id: MenuSelection::Systems,
    },
    MenuItemInfo {
        text: "Games",
        path: "data/svgs/charts.svg",
        selection_id: MenuSelection::Games,
    },
    MenuItemInfo {
        text: "Demos",
        path: "data/svgs/radio.svg",
        selection_id: MenuSelection::Demos,
    },
    MenuItemInfo {
        text: "Settings",
        path: "data/svgs/playlist.svg",
        selection_id: MenuSelection::Settings,
    },
    MenuItemInfo {
        text: "Debug",
        path: "data/svgs/history.svg",
        selection_id: MenuSelection::Debug,
    },
];

struct MenuItem {
    pos: Vec2,
    text_size: Vec2,
    selection_id: MenuSelection,
    text: &'static str,
    icon_path: &'static str,
    icon_pos: Vec2,
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
    width: u32,
    height: u32,
    // margin to the left screen edge
    icons_left_margin: u32,
    // margin between the icons and the text
    icons_text_margin: u32,
}

impl LeftSideMenu {
    pub fn new(width: u32, height: u32) -> Self {
        let items = MENU_ITEMS
            .iter()
            .map(|item| MenuItem {
                pos: Vec2::default(),
                text_size: Vec2::default(),
                icon_pos: Vec2::default(),
                selection_id: item.selection_id,
                icon_path: item.path,
                text: item.text,
                icon: Image { handle: 0 },
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
            icons_left_margin: 10,
            icons_text_margin: 10,
        }
    }

    // Calculate the text sizes so we know how large the icon images has to be.
    // We assume that the caller has loaded the font already at this point and will set it
    fn calculate_text_size(&mut self) {
        let mut options = ImageOptions::default();

        for menu_item in &mut self.items {
            menu_item.text_size = Ui::calc_text_size(menu_item.text);

            // We only set the height of the image, the width will be calculated automatically to keep the aspect ratio
            options.size = Vec2::new(0.0, menu_item.text_size.y);

            menu_item.icon = Image::load_with_options(menu_item.icon_path, options);
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

    fn calculate_layout(&mut self, width: u32, height: u32) -> flowi::Result<()> {
        let logo_info = Image::get_info(self.logo)?;
        let logo_size = Vec2 {
            x: logo_info.width as f32,
            y: logo_info.height as f32,
        };

        let mut max_width_icons = 0;
        let mut _icons_center_x = 0;
        let mut spacing_between_items = 10u32;
        let mut total_height = 0u32;
        let mut max_text_width = 0u32;

        // Get the size of each icon and also handle if the data is not loaded yet. We return false
        // from this function and it will be called again next frame until we returnt true.
        for (i, menu_item) in self.items.iter().enumerate() {
            let icon = Image::get_info(menu_item.icon)?;
            total_height += (menu_item.text_size.y as u32) + spacing_between_items;

            max_width_icons = max_width_icons.max(icon.width);
            max_text_width = max_text_width.max(menu_item.text_size.x as u32);

            _icons_center_x += icon.width;
        }

        _icons_center_x /= self.items.len() as u32;
        let x_icons_start = self.icons_left_margin;

        let text_start = x_icons_start + max_width_icons + self.icons_text_margin;

        let total_width = x_icons_start + text_start;
        let items_starting_y = (height - total_height) / 2;

        let mut y = items_starting_y;

        for menu_item in &mut self.items {
            menu_item.icon_pos.x = x_icons_start as _;
            menu_item.icon_pos.y = y as _;
            menu_item.pos.x = text_start as _;
            menu_item.pos.y = y as _;
            y += (menu_item.text_size.y as u32) + spacing_between_items;
        }

        self.logo_pos.x = 10.0; //((total_width - 180) / 2) as _;
        self.logo_pos.y = 4.0;
        self.width = 200 + (total_width + 20) as u32;
        self.height = height as u32;

        self.state = State::Ready;

        Ok(())
    }

    fn draw(&mut self) {
        Window::set_pos(Vec2 { x: 0.0, y: 0.0 });
        Window::set_size(Vec2 {
            x: self.width as _,
            y: self.height as _,
        });

        Window::begin("left_side_menu", WindowFlags::NO_DECORATION);

        Cursor::set_pos(self.logo_pos);
        Ui::image(self.logo);

        for menu_item in &self.items {
            Cursor::set_pos(menu_item.icon_pos);
            Ui::image(menu_item.icon);
            Cursor::set_pos(menu_item.pos);
            Button::regular(menu_item.text);
        }

        Window::end();
    }

    pub fn update(&mut self, width: u32, height: u32) {
        match self.state {
            State::CalculatingTextSizes => self.calculate_text_size(),
            State::WatingForAssets => self.wait_for_assets(),
            State::CalculateLayout => self.calculate_layout(width, height).unwrap(),
            State::Ready => self.draw(),
        }
    }
}
