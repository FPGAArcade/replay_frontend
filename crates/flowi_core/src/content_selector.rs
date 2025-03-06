use crate::content_provider::{ContentProvider, Item, ItemVisibility};
use crate::{fixed, grow, Alignment, BackgroundMode, ClayColor, Declaration, LayoutAlignmentX, LayoutAlignmentY, LayoutDirection, LoadPriority, Padding, Ui};
/// This module is responsible for displaying a list of items that can be selected. It acts very
/// similar to how movie based selectors for many streaming services works. The user can scroll
/// through a list of items and select one of them. The selected item will be displayed in a larger
/// size than the other items. Each item has an ID that
/// TODO: This shouldn't really be part of core-flowi, but we will keep it here for now.

#[derive(Debug, Default, Copy, Clone)]
#[allow(dead_code)]
struct RowColumn {
    row: u64,
    col: u64,
    id: u64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
enum State {
    Init,
    Idle,
    ColumnTransition,
    RowTransition,
}

#[allow(dead_code)]
struct ItemState {
    hot: f32,
}

#[allow(dead_code)]
const UNSELECTED_IMAGE_SIZE: (f32, f32) = (250.0, 187.5);
const ENTRY_ID: &str = "selection_entry";

pub struct ContentSelector {
    /// Selected item in row, col format
    selected_item: RowColumn,
    /// If we are about to transition to a new row this is the row we are transitioning to.
    transition_row: u64,
    /// Temporary time when we decide to transition to a new row
    temp_time: f32,
    /// The scroll value of the content selector
    scroll_value: f32,
    /// The scroll value of the content selector
    curve_transition: f32,
    /// Fade out value during transition
    row_transition_fade_out: f32,
    /// Current state of the content selector
    state: State,
    //item_states: HashMap<u64, ItemState>,
}

impl ContentSelector {
    pub fn new(_provider: &mut dyn ContentProvider) -> ContentSelector {
        ContentSelector {
            selected_item: RowColumn::default(),
            transition_row: 0,
            temp_time: 0.0,
            scroll_value: 0.0,
            row_transition_fade_out: 1.0,
            curve_transition: 0.0,
            state: State::Init,
            //item_states: HashMap::new(),
        }
    }

    #[rustfmt::skip]
    fn draw_row(&self, ui: &Ui, provider: &mut dyn ContentProvider, row: u64, opacity: f32) {
        let name = provider.get_row_name(ui, row);

        if name.is_empty() {
            return;
        }

        let id = ui.id_index(name, row as _);

        ui.set_font_size(36);
        ui.text_with_layout(name, ClayColor::rgba(255.0, 255.0, 255.0, 255.0 * opacity),
            &Declaration::new()
                .layout()
                    .width(grow!())
                    .end());

        ui.with_layout(&Declaration::new()
            .id(id)
            .layout()
                .width(grow!())
                .height(fixed!(360.0))
                .direction(LayoutDirection::LeftToRight)
                .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Center))
                .child_gap(64)
                .padding(Padding::horizontal(20))
            .end()
            .scroll(true, false), |ui|
       {
            // Get the number of columns we have for this row
            let column_count = provider.get_column_count(ui, row);

            // TODO: The layout system will deal with things that are hidden, but we likely should
            //       be a bit smarter with what we add to the layout to reduce requests to the backend
            //       as much as possible.
            // Added all the items to the layout for this row.
            for col in 0..column_count {
                let item_id = provider.get_item_id(row, col);
                let id = ui.id_index(ENTRY_ID, item_id as _);

                // Figure out visibility of the item
                let is_visible = ui.is_visible(id);
                let is_selected = col == self.selected_item.col && row == self.selected_item.row;

                let visibility = if is_selected {
                    ItemVisibility::Selected
                } else if is_visible {
                    ItemVisibility::Visible
                } else {
                    ItemVisibility::Hidden
                };

                let item = provider.get_item(ui, visibility, row, col);

                match visibility {
                    ItemVisibility::Hidden => {
                        ui.hint_load_priority(item.image, LoadPriority::Low);
                        ui.hint_load_priority(item.background_image, LoadPriority::Low);
                    }

                    ItemVisibility::Visible => {
                        ui.hint_load_priority(item.image, LoadPriority::High);
                    },

                    ItemVisibility::Selected => {
                        ui.hint_load_priority(item.background_image, LoadPriority::Highest);
                        ui.hint_load_priority(item.image, LoadPriority::High);
                    }
                }

                draw_selection_entry(self.temp_time, ui, &item, is_selected, opacity);
            }
       });
    }

    #[rustfmt::skip]
    pub fn update(&mut self, ui: &Ui, provider: &mut dyn ContentProvider) {
        let id = ui.id("content_selector");
        ui.update_scroll(id, (0.0, self.scroll_value));

        ui.with_layout(&Declaration::new()
            .id(id)
            .layout()
                .width(grow!())
                .height(grow!())
                .direction(LayoutDirection::TopToBottom)
                .padding(Padding::horizontal(20))
                .child_alignment(Alignment::new(LayoutAlignmentX::Left, LayoutAlignmentY::Top))
            .end()
            .scroll(false, true), |ui|
        {
            self.draw_row(ui, provider, self.selected_item.row, self.row_transition_fade_out);
            self.draw_row(ui, provider, self.selected_item.row + 1, 1.0);
            self.draw_row(ui, provider, self.selected_item.row + 2, 1.0);
        });

        let dt = ui.delta_time();
        self.temp_time += dt;

        //let down = false; //ui.input().is_action_active(ActionResponse::Down);
        // Simulate a transition to a new row ever 5 seconds
        let down = if self.temp_time > 5.0 {
            self.temp_time = 0.0;
            true
        } else {
            false
        };

        if self.state == State::Init {
            let item_id = provider.get_item_id(0, 0);
            provider.get_item(ui, ItemVisibility::Hidden, 0, 0);
            ui.set_focus_id(ui.id_index(ENTRY_ID, item_id as _));
            self.state = State::Idle;
        }

        if self.state == State::RowTransition {
            let anime_rate = 1.0 - 2f32.powf(-8.0 * dt);
            self.curve_transition += anime_rate * (1.0 - self.curve_transition);

            self.row_transition_fade_out -= dt;
            self.scroll_value = -(self.curve_transition * 400.0);

            if self.row_transition_fade_out <= 0.0 {
                self.row_transition_fade_out = 1.0;
                self.state = State::Idle;
                self.scroll_value = 0.0;
                self.curve_transition = 0.0;
                self.selected_item.row = self.transition_row;
            }
        }

        // TODO: Handle the case if we already are in a transition state
        if self.state == State::Idle && down {
            self.transition_row = self.selected_item.row + 1;
            let item_id = provider.get_item_id(self.transition_row, 0);
            let item = provider.get_item(ui, ItemVisibility::Hidden, self.transition_row, 0);
            ui.set_background_image(item.background_image, BackgroundMode::AlignTopRight);
            ui.set_focus_id(ui.id_index(ENTRY_ID, item_id as _));
            self.state = State::RowTransition;
        }
    }
}


#[allow(dead_code)]
#[rustfmt::skip]
fn draw_selection_entry(_time: f32, ui: &Ui, item: &Item, _is_selected: bool, opacity: f32) {
    // TODO: Get the data from settings structs as this is affected by the screen size
    let mut size = (250.0, 187.5);
    let id = ui.id_index(ENTRY_ID, item.id as _);

    ui.with_layout(&Declaration::new()
        .id(ui.id_index(ENTRY_ID, (item.id + 10000) as _))
        .layout()
            .width(fixed!(size.0))
            .height(fixed!(size.1))
            .child_alignment(Alignment::new(LayoutAlignmentX::Center, LayoutAlignmentY::Center))
        .end(), |ui|
    {
        if let Some(item_state) = ui.item_state(id) {
            size = (250.0 + (item_state.active * 40.0),  187.5 + (item_state.active * 40.0));
        }
        ui.image_with_opts(id, item.image, opacity, size);
    });
}
