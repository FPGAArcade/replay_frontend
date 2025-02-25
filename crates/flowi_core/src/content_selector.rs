use crate::content_provider::{ContentProvider, Item};
use crate::{
    fixed, grow, percent, ActionResponse, Alignment, ClayColor, Declaration, Dimensions,
    InputAction, LayoutAlignmentX, LayoutAlignmentY, LayoutDirection, Padding, Ui,
};
/// This module is responsible for displaying a list of items that can be selected. It acts very
/// similar to how movie based selectors for many streaming services works. The user can scroll
/// through a list of items and select one of them. The selected item will be displayed in a larger
/// size than the other items. Each item has an ID that
/// TODO: This shouldn't really be part of core-flowi, but we will keep it here for now.
//use image_old::RenderImage;
use arena_allocator::TypedArena;
use std::collections::HashMap;

#[derive(Debug, Default, Copy, Clone)]
struct RowColumn {
    row: u64,
    col: u64,
    id: u64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    Init,
    Idle,
    ColumnTransition,
    RowTransition,
}

struct ItemState {
    hot: f32,
}

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
    /// TODO: Optimize
    item_states: HashMap<u64, ItemState>,
}

impl ContentSelector {
    pub fn new(provider: &mut dyn ContentProvider) -> ContentSelector {
        ContentSelector {
            selected_item: RowColumn::default(),
            transition_row: 0,
            temp_time: 0.0,
            scroll_value: 0.0,
            row_transition_fade_out: 1.0,
            curve_transition: 0.0,
            state: State::Init,
            item_states: HashMap::new(),
        }
    }

    #[rustfmt::skip]
    fn draw_row(&self, ui: &Ui, provider: &mut dyn ContentProvider, row: u64, opacity: f32) {
        let name = provider.get_row_name(ui, row);

        if name.is_empty() {
            return;
        }

        let id = ui.id_index(name, row as _);

        ui.text_with_layout(name, 36,
            ClayColor::rgba(255.0, 255.0, 255.0, 255.0 * opacity),
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
                let item = provider.get_item(ui, row, col);
                let is_selected = col == 0;
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
            let item = provider.get_item(ui, 0, 0);
            ui.set_focus_id(ui.id_index(ENTRY_ID, item.id as _));
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
            let item = provider.get_item(ui, self.transition_row, 0);
            ui.set_focus_id(ui.id_index(ENTRY_ID, item.id as _));
            self.state = State::RowTransition;
        }
    }
}

#[allow(dead_code)]
#[rustfmt::skip]
fn draw_selection_entry(time: f32, ui: &Ui, item: &Item, is_selected: bool, opacity: f32) {
    // TODO: Get the data from settings structs as this is affected by the screen size
    let mut size = (250.0, 187.5);
    let id = ui.id_index(ENTRY_ID, item.id as _);

    if is_selected {
        // Extra layout here so we can animate the selected item with the fixed border without
        // affecting the size of the parent.
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

            ui.image_with_opts(id, item.selected_image, opacity, size);
        });
    } else {
        ui.image_with_opts(id, item.unselected_image, opacity, size);
    }
}
