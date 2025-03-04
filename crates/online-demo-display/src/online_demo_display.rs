use crate::data::*;
use log::*;
use flowi_core::content_provider::{ContentProvider, Item, ItemVisibility};
use flowi_core::content_selector::ContentSelector;
/// This file is the main code for the online demo browser for the frontend. It is responsible for
/// displaying a list of items that can be selected. It acts very similar to how movie based
/// selectors for many streaming services works. The user can scroll through a list of items and
/// select one of them. The selected item will be displayed in a larger size than the other items.
/// THe backend uses the Demozoo API to fetch the metadata along with screenshots from it's db.
use flowi_core::{Alignment, Declaration, LayoutAlignmentX, LayoutAlignmentY, LayoutDirection, Padding, Ui, fixed, grow, percent, Color, FontStyle};
use flowi_core::{IoHandle, LoadPriority, LoadState};
use log::error;
//use log::*;
use nanoserde::DeJson;
use std::fmt::Write;
//use std::time::Duration;
use std::collections::HashMap;

const API_URL: &str = "https://demozoo.org/api/v1";

enum QueuedJob {
    Party(IoHandle),
    Production(i32, IoHandle),
}

pub struct OnlineDemoDisplay {
    url_string: String,
    parties: Vec<Box<Party>>,
    productions_loaded: HashMap<i32, Box<ProductionEntry>>,
    production_items: HashMap<i32, Item>,
    jobs: Vec<QueuedJob>,
    selected_item: Option<(u64, u64)>,
}

impl OnlineDemoDisplay {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            parties: Vec::new(),
            url_string: String::with_capacity(128),
            productions_loaded: HashMap::new(),
            production_items: HashMap::new(),
            selected_item: None,
        }
    }

    /// Fetches a party from the Demozoo API
    pub fn fetch_party(&mut self, ui: &Ui, party_id: u64) {
        self.url_string.clear();
        write!(self.url_string, "{}/parties/{}", API_URL, party_id).unwrap();

        let handle = ui.load_with_callback(
            &self.url_string,
            LoadPriority::Normal,
            Box::new(|data| {
                let json_data = std::str::from_utf8(data).expect("Failed to parse string");
                let party: Party =
                    DeJson::deserialize_json(json_data).expect("Failed to parse JSON");

                Box::new(party)
            }),
        );

        self.jobs.push(QueuedJob::Party(handle));
    }

    /// Queues the screenshots for loading. If there are no screenshots, it will return a pair of
    /// (0, 0) IoHandles.
    /// TODO: We should have a default image here instead of null handles
    fn queue_screenshots(entry: &ProductionEntry, ui: &Ui) -> (IoHandle, IoHandle) {
        if entry.screenshots.is_empty() {
            (IoHandle(0), IoHandle(0))
        } else if entry.screenshots.len() == 1 {
            let handle = ui.load_image(&entry.screenshots[0].thumbnail_url, None);
            let bgi = ui.load_background_image(&entry.screenshots[0].original_url);
            (handle, bgi)
        } else {
            let h0 = ui.load_image(&entry.screenshots[0].thumbnail_url, None);
            let bgi = ui.load_background_image(&entry.screenshots[1].original_url);
            (h0, bgi)
        }
    }

    pub fn update(&mut self, ui: &Ui) {
        for job in self.jobs.iter_mut() {
            match job {
                QueuedJob::Party(handle) => match ui.return_loaded(*handle, LoadPriority::Normal) {
                    LoadState::Loaded(data) => {
                        match data.downcast::<Party>() {
                            Ok(party) => self.parties.push(party),
                            Err(_) => error!("Failed to downcast to Party"),
                        }
                    }
                    _ => {}
                },

                QueuedJob::Production(id, handle) => match ui.return_loaded(*handle, LoadPriority::Normal) {
                    LoadState::Loaded(data) => {
                        match data.downcast::<ProductionEntry>() {
                            Ok(production) => {
                                let screenshots = Self::queue_screenshots(&production, ui);
                                self.productions_loaded.insert(*id, production);
                                self.production_items.insert(*id, Item {
                                    image: screenshots.0,
                                    background_image: screenshots.1,
                                    id: *id as _,
                                });
                            }
                            Err(_) => error!("Failed to downcast to ProductionEntry"),
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}

impl ContentProvider for OnlineDemoDisplay {
    fn get_item_id(&mut self, row: u64, col: u64) -> u64 {
        if self.parties.is_empty() {
            return 0;
        }

        let party = &self.parties[0];
        let release = &party.competitions[row as usize].results[col as usize].production;
        // in order to make the ids unique wi include row / column in the idea
        let id = release.id as u64;
        id ^ ((row as u64) << 16) ^ (row as u64)
    }
    fn get_item(&mut self, ui: &Ui, visibility: ItemVisibility, row: u64, col: u64) -> Item {
        if self.parties.is_empty() {
            return Item {
                image: IoHandle(0),
                background_image: IoHandle(0),
                id: u64::MAX / 2,
            };
        }

        let id = self.get_item_id(row, col) as i32;
        let party = &self.parties[0];
        let release = &party.competitions[row as usize].results[col as usize].production;

        // First we check if we have loaded the production entry
        if let Some(entry) = self.production_items.get(&id) {
            if visibility == ItemVisibility::Selected {
                self.selected_item = Some((row, col));
            }
            return *entry;
        }

        let priority = match visibility {
            ItemVisibility::Hidden => LoadPriority::Low,
            ItemVisibility::Visible => LoadPriority::High,
            ItemVisibility::Selected => LoadPriority::Highest,
        };

        // find if we have the job in the queue already
        let found = self.jobs.iter().find(|job| match job {
            QueuedJob::Production(job_id, _) => *job_id == id,
            _ => false,
        });

        match found {
            Some(QueuedJob::Production(_, handle)) => {
                debug!("Hinting production {} with priority {:?}", id, priority);
                ui.hint_load_priority(*handle, priority);
            },
            None => {
                debug!("Loading production {} with priority {:?}", id, priority);

                let handle = ui.load_with_callback(
                    &release.url,
                    priority,
                    Box::new(|data| {
                        let json_data = std::str::from_utf8(data).expect("Failed to parse string");
                        let production: ProductionEntry =
                            DeJson::deserialize_json(json_data).expect("Failed to parse JSON");
                        Box::new(production)
                    }),
                );

                self.jobs.push(QueuedJob::Production(id, handle));
            }

            _ => (),
        }

        Item {
            image: IoHandle(0),
            background_image: IoHandle(0),
            id: self.get_item_id(row, col) as _,
        }
    }

    fn get_column_count(&mut self, _ui: &Ui, row: u64) -> u64 {
        if self.parties.is_empty() {
            return 0;
        }

        // TODO: Filtering and stuff goes here
        let party = &self.parties[0];
        party.competitions[row as usize].results.len() as u64
    }

    fn get_row_name(&mut self, _ui: &Ui, row: u64) -> &str {
        if self.parties.is_empty() || row as usize >= self.parties[0].competitions.len() {
            return "";
        }

        let party = &self.parties[0];
        &party.competitions[row as usize].name
    }
}

pub struct OnlineDemoSelector {
    pub content_selector: ContentSelector,
    pub content_provider: OnlineDemoDisplay,
}

fn update(ui: &Ui, selector: &mut ContentSelector, content: &mut OnlineDemoDisplay) {
    selector.update(ui, content);
}

impl OnlineDemoSelector {
    pub fn new() -> OnlineDemoSelector {
        let mut content_provider = OnlineDemoDisplay::new();

        OnlineDemoSelector {
            content_selector: ContentSelector::new(&mut content_provider),
            content_provider,
        }
    }

    pub fn update(&mut self, ui: &Ui) {
        ui.with_layout(
            &Declaration::new()
                .id(ui.id("foo"))
                .layout()
                .width(grow!())
                .height(fixed!(400.0))
                .direction(LayoutDirection::TopToBottom)
                .child_alignment(Alignment::new(
                    LayoutAlignmentX::Left,
                    LayoutAlignmentY::Center,
                ))
                .child_gap(10)
                .end(), |ui| {

                let selected_id = if let Some((row, col)) = self.content_provider.selected_item {
                    self.content_provider.get_item_id(row, col) as _
                } else {
                    0
                };

                if let Some((row, col)) = self.content_provider.selected_item {
                    let party = &self.content_provider.parties[0];
                    let release = &party.competitions[row as usize].results[col as usize].production;
                    if let Some(entry) = self.content_provider.productions_loaded.get(&selected_id) {
                        display_entry(ui, release, entry);
                    }
                }

                // TODO: Fill out entry info here
            },
        );

        self.content_provider.update(ui);
        update(ui, &mut self.content_selector, &mut self.content_provider);
    }
}

#[rustfmt::skip]
fn display_entry(ui: &Ui, release: &Release, entry: &ProductionEntry) {
    ui.with_layout(&Declaration::new()
        .id(ui.id("entry_info"))
        .layout()
            .width(grow!())
            .height(fixed!(200.0))
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
            ui.select_font(FontStyle::Thin);

            let text_size = ui.text_size(&entry.title, 78);

            ui.text_with_layout(&entry.title, 78, (255.0, 255.0, 255.0, 255.0).into(),
                &Declaration::new()
                    .layout()
                        .width(fixed!(text_size.width))
                        .height(fixed!(text_size.height))
                        .padding(Padding::horizontal(32))
                        .end());

            ui.text_with_layout("1992", 78, (128.0, 128.0, 128.0, 255.0).into(),
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
            ui.select_font(FontStyle::Default);

            if release.types.len() > 0 {
                ui.button(&release.types[0].name);
            }

            if entry.platforms.len() > 0 {
                ui.button(&entry.platforms[0].name);
            }

            ui.text_with_layout("by", 36, (255.0, 255.0, 255.0, 255.0).into(),
                &Declaration::new()
                    .layout()
                        .width(fixed!(44.0))
                        .end());

            ui.select_font(FontStyle::Bold);

            ui.text_with_layout(&entry.author_nicks[0].name, 36, (201.0, 22.0, 38.0, 255.0).into(),
                &Declaration::new()
                    .layout()
                        .width(grow!())
                        .end());
        });
    });
}
