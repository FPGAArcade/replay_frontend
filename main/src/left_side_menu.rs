use flowi::ClayColor as Color;
use flowi::Ui;
use flowi::{
    fixed, grow, Alignment, Layout, LayoutAlignmentX, LayoutAlignmentY, LayoutDirection, Padding,
    Rectangle,
};

#[derive(Copy, Clone)]
#[allow(dead_code)]
enum MenuSelection {
    Search,
    Systems,
    Games,
    Demos,
    Settings,
    Debug,
}

#[allow(dead_code)]
struct MenuItemInfo {
    selection_id: MenuSelection,
    text: &'static str,
    path: &'static str,
}

#[allow(dead_code)]
static MENU_ITEMS: &[MenuItemInfo] = &[
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

#[allow(dead_code)]
struct MenuItem {
    selection_id: MenuSelection,
    text: &'static str,
    icon_path: &'static str,
    //icon_pos: Vec2,
    //icon_size: Vec2,
    //icon: Image,
    //color: Color,
}

#[allow(dead_code)]
pub struct LeftSideMenu {
    // TODO: Arena
    items: Vec<MenuItem>,
    offset: f32,
}

#[allow(dead_code)]
impl LeftSideMenu {
    pub fn new(_flowi: &Ui) -> Self {
        let items = MENU_ITEMS
            .iter()
            .map(|item| MenuItem {
                selection_id: item.selection_id,
                text: item.text,
                icon_path: item.path,
            })
            .collect::<Vec<_>>();

        Self { items, offset: 0.0 }
    }

    #[rustfmt::skip]
    pub fn update(&mut self, ui: &Ui) {
        self.offset += 0.01;
        ui.with_layout(Some("launcher_left_side"), [
            Layout::new()
                .height(grow!())
                .width(fixed!(180.0))
                .child_gap(2)
                .direction(LayoutDirection::TopToBottom)
                .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
                .end(),
            Rectangle::new()
                .color(Color::rgba(30.0, 30.0, 30.0, 255.0))
                .end()], |ui|
        {
            for (index, menu_item) in self.items.iter().enumerate() {
                let state = ui.button_with_layout(menu_item.text, [
                    Layout::new()
                        .width(fixed!(180.0))
                        .height(fixed!(76.0))
                        .child_alignment(Alignment::new(LayoutAlignmentX::Center, LayoutAlignmentY::Center))
                        .end(),
                    Rectangle::new()
                        .color(Color::rgba(44.0, 40.0, 40.0, 255.0))
                        .end()]);

                if state.hovering() {
                    println!("Hovering over {}", menu_item.text);
                }

                if state.clicked() {
                    println!("Clicked on {}", menu_item.text);
                }
            }
        });
    }
}
