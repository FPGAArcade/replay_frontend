use flowi::{fixed, grow, percent, Alignment, ClayColor, Declaration, LayoutAlignmentX, LayoutAlignmentY, LayoutDirection, Padding, Ui};
use crate::{App};
use crate::content_provider::{ContentProvider, Item};
use crate::content_selector::ContentSelector;
use flowi_api::ImageHandle;

static DUMMY_STRINGS: [&str; 10] = [
    "Demo list 1",
    "Demo list 2",
    "Demo list 3",
    "Demo list 4",
    "Demo list 5",
    "Demo list 6",
    "Demo list 7",
    "Demo list 8",
    "Demo list 9",
    "Demo list 10",
];

struct OnlineDemoContentProvider {
    items: Vec<Item>,
}

impl OnlineDemoContentProvider {
    pub(crate) fn new() -> OnlineDemoContentProvider {
        let mut items = Vec::new();
        for i in 0..100 {
            items.push(Item {
                unselected_image: 0,
                selected_image: 0,
                id: i + 1,
            });
        }
        OnlineDemoContentProvider { items }
    }
}

impl ContentProvider for OnlineDemoContentProvider {
    fn set_image_sizes(&mut self, _unselected: (f32, f32), _selected: (f32, f32)) {
        // We don't care about the image sizes in this example
    }
    fn get_item(&self, row: u64, col: u64) -> &Item {
        if let Some(t) = self.items.get((row * 10 + col) as usize) {
            t
        } else {
            &Item {
                unselected_image: 0,
                selected_image: 0,
                id: u64::MAX,
            }
        }
    }

    fn get_column_count(&self, row: u64) -> u64 {
        10
    }

    fn get_total_row_count(&self) -> u64 {
        10
    }

    fn get_row_name(&self, row: u64) -> &str {
        DUMMY_STRINGS[row as usize]
    }
}

pub(crate) struct OnlineDemoSelector {
    content_selector: ContentSelector,
    content_provider: OnlineDemoContentProvider,
}

fn update(ui: &Ui, selector: &mut ContentSelector, content: &OnlineDemoContentProvider) {
    selector.update(ui, content);
}

impl OnlineDemoSelector {
    pub(crate) fn new() -> OnlineDemoSelector {
        let mut content_provider = OnlineDemoContentProvider::new();

        OnlineDemoSelector {
            content_selector: ContentSelector::new(&mut content_provider),
            content_provider,
        }
    }

    pub fn update(&mut self, ui: &Ui) {
        ui.with_layout(&Declaration::new()
            .id(ui.id("foo"))
            .layout()
                .width(grow!())
                .height(fixed!(400.0))
                .direction(LayoutDirection::TopToBottom)
                .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
                .child_gap(10)
            .end()
            .background_color(ClayColor::rgba(255.0, 0.0, 0.0, 255.0)), |ui|
        {
            // TODO: Fill out entry info here
        });

        /*
        ui.with_layout(&Declaration::new()
            .id(ui.id("bar"))
            .layout()
                .width(grow!())
                .height(grow!())
                .direction(LayoutDirection::LeftToRight)
                .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
                .child_gap(10)
            .end()
            .background_color(ClayColor::rgba(0.0, 255.0, 0.0, 255.0)), |ui|
        {

         */
       update(ui, &mut self.content_selector, &self.content_provider);
        //});
    }
}

/*
#[rustfmt::skip]
fn display_entry(ui: &Ui, app: &App, entry: &DemoEntry) {
    ui.with_layout(&Declaration::new()
        .id(ui.id("entry_info"))
        .layout()
            .width(grow!())
            .height(percent!(0.5))
            .direction(LayoutDirection::TopToBottom)
            .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
            .child_gap(10)
         .end(), |ui|
    {
        ui.with_layout(&Declaration::new()
            .id(ui.id("tile_info"))
            .layout()
                .width(grow!())
                .height(fixed!(80.0))
                .child_gap(40)
                .direction(LayoutDirection::LeftToRight)
            .end(), |ui|
        {
            ui.set_font(app.fonts.thin);

            let text_size = ui.text_size(&entry.metadata.title, 78);

            ui.text_with_layout(&entry.metadata.title, 78,
                ClayColor::rgba(255.0, 255.0, 255.0, 255.0),
                &Declaration::new()
                    .layout()
                        .width(fixed!(text_size.width))
                        .height(fixed!(text_size.height))
                        .padding(Padding::horizontal(32))
                        .end());

            ui.text_with_layout("1992", 78,
                ClayColor::rgba(128.0, 128.0, 128.0, 255.0),
                &Declaration::new()
                    .layout()
                        .width(grow!())
                        .end());
        });

        ui.with_layout(&Declaration::new()
            .id(ui.id("platform_info"))
            .layout()
                .width(grow!())
                .height(fixed!(40.0))
                .padding(Padding::horizontal(32))
                .child_gap(16)
                .direction(LayoutDirection::LeftToRight)
            .end(), |ui|
        {
            ui.set_font(app.fonts.default);

            ui.button("DEMO");
            ui.button("AMIGA OCS/ECS");

            ui.text_with_layout("by", 36,
                ClayColor::rgba(255.0, 255.0, 255.0, 255.0),
                &Declaration::new()
                    .layout()
                        .width(fixed!(44.0))
                        .end());

            ui.set_font(app.fonts.bold);

            ui.text_with_layout(&entry.metadata.author_nicks[0].name, 36,
                ClayColor::rgba(201.0, 22.0, 38.0, 255.0),
                &Declaration::new()
                    .layout()
                        .width(grow!())
                        .end());

        });
    });
}
*/
