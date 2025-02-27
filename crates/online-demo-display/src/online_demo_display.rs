use crate::data::*;
use flowi_core::content_provider::{ContentProvider, Item};
use flowi_core::content_selector::ContentSelector;
/// This file is the main code for the online demo browser for the frontend. It is responsible for
/// displaying a list of items that can be selected. It acts very similar to how movie based
/// selectors for many streaming services works. The user can scroll through a list of items and
/// select one of them. The selected item will be displayed in a larger size than the other items.
/// THe backend uses the Demozoo API to fetch the metadata along with screenshots from it's db.
use flowi_core::{
    Alignment, ClayColor, Declaration, LayoutAlignmentX, LayoutAlignmentY, LayoutDirection,
    //Padding,
    Ui, fixed, grow,
    //job_system::{BoxAnySend, JobHandle, JobResult, JobSystem},
    //percent,
};
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
}

impl OnlineDemoDisplay {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            parties: Vec::new(),
            url_string: String::with_capacity(128),
            productions_loaded: HashMap::new(),
            production_items: HashMap::new(),
        }
    }

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

    fn queue_screenshots(entry: &ProductionEntry, ui: &Ui) -> (IoHandle, IoHandle){
        if entry.screenshots.is_empty() {
            (IoHandle(0), IoHandle(0))
        } else if entry.screenshots.len() == 1 {
            let handle = ui.load_image(&entry.screenshots[0].thumbnail_url, None);
            (handle, handle)
        } else {
            let h0 = ui.load_image(&entry.screenshots[0].thumbnail_url, None);
            let h1 = ui.load_image(&entry.screenshots[1].thumbnail_url, None);
            (h0, h1)
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
                                    unselected_image: screenshots.0,
                                    selected_image: screenshots.1,
                                    id: *id as u64,
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
    fn get_item(&mut self, ui: &Ui, row: u64, col: u64) -> Item {
        if self.parties.is_empty() {
            return Item {
                unselected_image: IoHandle(0),
                selected_image: IoHandle(0),
                id: u64::MAX / 2,
            };
        }

        let party = &self.parties[0];
        let release = &party.competitions[row as usize].results[col as usize].production;
        let id = release.id;

        // First we check if we have loaded the production entry
        if let Some(entry) = self.production_items.get(&id) {
            return *entry;
        }

        // find if we have the job in the queue already
        let found = self.jobs.iter().find(|job| match job {
            QueuedJob::Production(job_id, _) => *job_id == id,
            _ => false,
        });

        if found.is_none() {
            let handle = ui.load_with_callback(
                &release.url,
                LoadPriority::Normal,
                Box::new(|data| {
                    let json_data = std::str::from_utf8(data).expect("Failed to parse string");
                    let production: ProductionEntry =
                        DeJson::deserialize_json(json_data).expect("Failed to parse JSON");
                    Box::new(production)
                }),
            );

            self.jobs.push(QueuedJob::Production(id, handle));
        }

        Item {
            unselected_image: IoHandle(0),
            selected_image: IoHandle(0),
            id: id as _,
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
        if self.parties.is_empty() {
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
                .end()
                .background_color(ClayColor::rgba(255.0, 0.0, 0.0, 255.0)),
            |_ui| {
                // TODO: Fill out entry info here
            },
        );

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
        self.content_provider.update(ui);
        update(ui, &mut self.content_selector, &mut self.content_provider);
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
