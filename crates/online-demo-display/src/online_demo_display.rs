/// This file is the main code for the online demo browser for the frontend. It is responsible for
/// displaying a list of items that can be selected. It acts very similar to how movie based
/// selectors for many streaming services works. The user can scroll through a list of items and
/// select one of them. The selected item will be displayed in a larger size than the other items.
/// THe backend uses the Demozoo API to fetch the metadata along with screenshots from it's db.
use flowi_core::{
    fixed,
    grow,
    percent, Alignment, ClayColor, Declaration, LayoutAlignmentX, LayoutAlignmentY, LayoutDirection, Padding, Ui,
    job_system::{BoxAnySend, JobHandle, JobResult, JobSystem},
};
use flowi_core::content_provider::{ContentProvider, Item};
use flowi_core::content_selector::ContentSelector;
use flowi_core::{IoHandle, LoadState, LoadPriority};
use log::*;
use std::{fs, io};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use nanoserde::DeJson;
use std::fmt::Write;
use std::hash::Hasher;
use std::time::Duration;
use log::error;
use crate::data::*;

const API_URL: &str = "https://demozoo.org/api/v1";

fn parse_party(json_data: &str) -> Party {
    DeJson::deserialize_json(json_data).expect("Failed to parse JSON")
}

/*
fn read_party_from_cache_job(data: BoxAnySend) -> JobResult<BoxAnySend> {
    read_data(
        data,
        DataSource::Cache,
        DataFormat::String(Box::new(parse_party))
    )
}

fn read_party_from_remote(data: BoxAnySend) -> JobResult<BoxAnySend> {
    read_data(
        data,
        DataSource::Remote,
        DataFormat::String(Box::new(parse_party))
    )
}

fn read_production_entry_from_cache(data: BoxAnySend) -> JobResult<BoxAnySend> {
    read_data(
        data,
        DataSource::Cache,
        DataFormat::String(Box::new(parse_production_entry))
    )
}

fn read_production_entry_from_remote(data: BoxAnySend) -> JobResult<BoxAnySend> {
    read_data(
        data,
        DataSource::Remote,
        DataFormat::String(Box::new(parse_production_entry))
    )
}

 */


enum FetchItem {
    Party(u64, String),
    Release((u64, String)),
    Screenshot((u64, String)),
}

enum QueuedJob {
    Party(IoHandle),
    Release(IoHandle),
    Screenshot(IoHandle),
}

pub struct OnlineDemoDisplay {
    url_string: String,
    parties: Vec<Box<Party>>,
    jobs: Vec<QueuedJob>,
}

impl OnlineDemoDisplay {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            parties: Vec::new(),
            url_string: String::with_capacity(128),
        }
    }

    fn fetch_party(&mut self, ui: &Ui, party_id: u64) {
        self.url_string.clear();
        write!(self.url_string, "{}/parties/{}", API_URL, party_id).unwrap();

        let handle = ui.load_with_callback(&self.url_string, Box::new(|data| {
            let json_data = std::str::from_utf8(data).expect("Failed to parse string");
            let party: Party = DeJson::deserialize_json(json_data).expect("Failed to parse JSON");
            party
        }));

        self.jobs.push(QueuedJob::Party(handle));
    }

    pub fn update(&mut self, ui: &Ui) {
        for job in self.jobs.iter_mut() {
            match job {
                QueuedJob::Party(handle) => {
                    match ui.return_loaded(*handle, LoadPriority::Normal) {
                        LoadState::Loaded(data) => {
                            if let Ok(party) = data.downcast::<Party>() {
                                self.parties.push(party);
                            } else {
                                error!("Failed to parse party data");
                            }
                        }
                        _ => { },
                    }
                }

                _ => {}
            }
        }
        /*
        match self.state {
            State::Idle => {}

            State::FetchParty =>
                fetch_party(),
                self.url_string.clear();
                write!(self.url_string, "https://demozoo.org/api/v1/parties/{}", self.party_show_id).unwrap();
                self.load_party_handle = Some(ui.job_system().schedule_job(
                    fetch_party_job,
                    Box::new(self.url_string.clone())).unwrap());

                self.state = State::WaitFetchingParty;
            }

            State::WaitFetchingParty => {
                if let Some(handle) = self.load_party_handle.as_ref() {
                    // TODO: Proper error handling
                    self.active_party = Some(handle.try_get_result::<Party>().unwrap().unwrap());
                    self.state = State::ShowParty;
                }
            }

            _ => {},
        }

         */
    }
}

impl ContentProvider for OnlineDemoDisplay {
    fn get_item(&self, row: u64, col: u64) -> Item {
        /*
        if self.state == State::ShowParty {
            if let Some(party) = self.active_party.as_ref() {
                let id = party.competitions[row as usize].results[col as usize].production.id as u64;
                return Item {
                    unselected_image: id,
                    selected_image: id,
                    id,
                }
            }
        }

         */

        Item {
            unselected_image: IoHandle(0),
            selected_image: IoHandle(0),
            id: u64::MAX / 2,
        }
    }

    fn get_column_count(&self, row: u64) -> u64 {
        /*
        if self.state == State::ShowParty {
            if let Some(party) = self.active_party.as_ref() {
                return party.competitions[row as usize].results.len() as u64;
            }
        }

         */

        0
    }

    fn get_row_name(&self, row: u64) -> &str {
        /*
        if self.state == State::ShowParty {
            if let Some(party) = self.active_party.as_ref() {
                return &party.competitions[row as usize].name;
            }
        }

         */

        ""
    }
}


pub struct OnlineDemoSelector {
    content_selector: ContentSelector,
    content_provider: OnlineDemoDisplay,
}

fn update(ui: &Ui, selector: &mut ContentSelector, content: &OnlineDemoDisplay) {
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
       self.content_provider.update(ui);
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
