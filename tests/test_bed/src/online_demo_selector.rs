/// This file is the main code for the online demo browser for the frontend. It is responsible for
/// displaying a list of items that can be selected. It acts very similar to how movie based
/// selectors for many streaming services works. The user can scroll through a list of items and
/// select one of them. The selected item will be displayed in a larger size than the other items.
/// THe backend uses the Demozoo API to fetch the metadata along with screenshots from it's db.
use flowi::{
    fixed,
    grow,
    percent, Alignment, ClayColor, Declaration, LayoutAlignmentX, LayoutAlignmentY, LayoutDirection, Padding, Ui,
    job_system::{BoxAnySend, JobHandle, JobResult, JobSystem},
};
use crate::content_provider::{ContentProvider, Item};
use crate::content_selector::ContentSelector;
use log::*;
use std::{fs, io};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use nanoserde::DeJson;
//use std::io::Write;
use std::fmt::Write;
use std::hash::Hasher;
use std::time::Duration;

/// The JSON “author_nicks” array is an array of objects. We only care about the name.
#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct AuthorNick {
    pub name: String,
    // Other fields (like abbreviation or releaser) are ignored.
}

/// The JSON “credits” array holds more detailed information about a production’s credits.
/// We use this struct to capture the “nick” (an AuthorNick) plus a category and role.
#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct Credit {
    pub nick: AuthorNick,
    pub category: String,
    pub role: String,
}

/// The JSON “download_links” array contains objects with a link class and a URL.
#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct DownloadLink {
    pub link_class: String,
    pub url: String,
}

/// The JSON “platforms” array gives details for each platform. (We only really care about the name.)
#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct Platform {
    pub url: String,
    pub id: u32,
    pub name: String,
}

/// The JSON “screenshots” array has keys like “original_url” which we map into our struct.
/// We use Serde’s rename attribute to change the JSON key into our field name.
#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct Screenshot {
    pub original_url: String,
    pub original_width: u32,
    pub original_height: u32,
    pub standard_url: String,
    pub standard_width: u32,
    pub standard_height: u32,
    pub thumbnail_url: String,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
}

/// The main ProductionEntry struct gathers the fields we care about from the JSON.
#[derive(DeJson, Debug)]
pub struct ProductionEntry {
    pub title: String,
    pub release_date: String,
    pub author_nicks: Vec<AuthorNick>,
    pub credits: Vec<Credit>,
    pub download_links: Vec<DownloadLink>,
    pub platforms: Vec<Platform>,
    pub screenshots: Vec<Screenshot>,
    pub tags: Vec<String>,
}
#[derive(DeJson, Debug)]
pub struct Invitation {
    pub url: String,
    pub demozoo_url: String,
    pub id: u32,
    pub title: String,
    pub author_nicks: Vec<AuthorNick>,
    pub author_affiliation_nicks: Vec<String>, // Assuming it's an empty array, using String
    pub release_date: String,
    pub supertype: String,
    pub platforms: Vec<Platform>,
    pub types: Vec<ProductionType>,
    pub tags: Vec<String>,
}

#[derive(DeJson, Debug)]
pub struct Party {
    pub url: String,
    pub demozoo_url: String,
    pub id: i32,
    pub name: String,
    pub tagline: String,
    pub party_series: PartySeries,
    pub start_date: String,
    pub end_date: String,
    pub location: String,
    pub is_online: bool,
    pub country_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub website: String,
    pub invitations: Vec<Invitation>, // Assuming empty array means Vec<String>
    pub releases: Vec<Release>,
    pub competitions: Vec<Competition>,
}

#[derive(DeJson, Debug)]
pub struct PartySeries {
    pub url: String,
    pub demozoo_url: String,
    pub id: i32,
    pub name: String,
    pub website: String,
}

#[derive(DeJson, Debug)]
pub struct Release {
    pub url: String,
    pub demozoo_url: String,
    pub id: i32,
    pub title: String,
    pub author_nicks: Vec<AuthorNick>,
    pub author_affiliation_nicks: Vec<AuthorNick>, // Empty in the example, but could be similar to author_nicks
    pub release_date: String,
    pub supertype: String,
    pub platforms: Vec<Platform>,
    pub types: Vec<ProductionType>,
    pub tags: Vec<String>,
}

#[derive(DeJson, Debug)]
pub struct Releaser {
    pub url: String,
    pub id: i32,
    pub name: String,
    pub is_group: bool,
}

#[derive(DeJson, Debug)]
pub struct ProductionType {
    pub url: String,
    pub id: i32,
    pub name: String,
    pub supertype: String,
}

#[derive(DeJson, Debug)]
pub struct Competition {
    pub id: i32,
    pub demozoo_url: String,
    pub name: String,
    pub shown_date: String,
    pub platform: Option<Platform>,
    pub production_type: ProductionType,
    pub results: Vec<Result>,
}

#[derive(DeJson, Debug)]
pub struct Result {
    pub position: i32,
    pub ranking: String,
    pub score: String,
    pub production: Release,
}

fn parse_production_entry(json_data: &str) -> ProductionEntry {
    DeJson::deserialize_json(json_data).expect("Failed to parse JSON")
}

fn parse_party(json_data: &str) -> Party {
    DeJson::deserialize_json(json_data).expect("Failed to parse JSON")
}

/// Directory to store cached images
#[allow(dead_code)]
const CACHE_DIR: &str = "target/cache";

/// Computes a SHA256 hash of the URL to use as a unique filename

/// Extracts the file extension from the URL or defaults to "bin" if not found
#[allow(dead_code)]
fn get_extension(url: &str) -> &str {
    Path::new(url)
        .extension()
        .and_then(|ext| ext.to_str())
        .filter(|ext| !ext.is_empty()) // Ensure we don't return an empty extension
        .unwrap_or("bin") // Default to "bin" if no valid extension found
}

/// Checks if the image is already cached; otherwise, downloads it
#[allow(dead_code)]
pub fn get_from_remote(url: &str) -> io::Result<Vec<u8>> {
    // Fetch the image from the URL
    println!("Downloading: {}", url);
    let resp = ureq::get(url)
        .call()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut reader = resp
        .into_body()
        .into_with_config()
        .reader();

    // Read binary data from response using `response.into_reader()`
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;  // Changed to use `reader()`

    let mut hasher = fxhash::FxHasher64::default();
    hasher.write(url.as_bytes());
    let hash = hasher.finish(); // Returns u64

    let mut output = String::with_capacity(32);

    // Format the u64 as hexadecimal
    use std::fmt::Write;
    write!(output, "{}/{:x}", CACHE_DIR, hash).unwrap(); // Write hex without "0x" prefix

    // Write the image to the cache directory
    {
        use std::io::Write;
        let mut file = File::create(&output)?;
        file.write_all(&bytes)?;
    }

    println!("Saved to cache: {}", output);
    Ok(bytes)
}

fn load_cached_file(filename: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}


fn hash_url_to_string(url: &str) -> String {
    let mut hasher = fxhash::FxHasher64::default();
    hasher.write(url.as_bytes());
    let hash = hasher.finish(); // Returns u64

    let mut output = String::with_capacity(32);

    // Format the u64 as hexadecimal
    use std::fmt::Write; // For write! macro
    write!(output, "{}/{:x}", CACHE_DIR, hash).unwrap();
    output
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    Idle,
    FetchParty,
    WaitFetchingParty,
    ShowParty,
}

fn load_party_from_file_job(data: BoxAnySend) -> JobResult<BoxAnySend> {
    let filename = data.downcast::<String>().unwrap();
    let data = load_cached_file(&filename).unwrap();
    let string = std::str::from_utf8(&data).expect("Failed to convert to string");
    let party = parse_party(&string);
    Ok(Box::new(party) as BoxAnySend)
}

fn load_party_from_remote_job(data: BoxAnySend) -> JobResult<BoxAnySend> {
    let url = data.downcast::<String>().unwrap();
    let data = get_from_remote(&url).unwrap();
    let string = std::str::from_utf8(&data).expect("Failed to convert to string");
    let party = parse_party(&string);
    Ok(Box::new(party) as BoxAnySend)
}

fn fetch_data_string<T: Sized + Send + 'static, F: FnOnce(&str) -> T>(data: BoxAnySend, callback: F) -> JobResult<BoxAnySend> {
    let url = data.downcast::<String>().unwrap();

    let cached_filename = hash_url_to_string(&url);
    dbg!(&cached_filename);

    if let Ok(data) = load_cached_file(&cached_filename) {
        debug!("Read url {} from cache {}", url, cached_filename);
        let string = std::str::from_utf8(&data).expect("Failed to convert to string");
        Ok(Box::new(callback(&string)) as BoxAnySend)
    } else {
        debug!("Failed to read url {} from cache {}. Fetching from remote.", url, cached_filename);
        dbg!(&url);
        let data = get_from_remote(&url).unwrap();
        let string = std::str::from_utf8(&data).expect("Failed to convert to string");
        Ok(Box::new(callback(&string)) as BoxAnySend)
    }
}

fn fetch_data_bin<T: Sized + Send + 'static, F: FnOnce(&[u8]) -> T>(data: BoxAnySend, callback: F) -> JobResult<BoxAnySend> {
    let url = data.downcast::<String>().unwrap();

    let cached_filename = hash_url_to_string(&url);

    if let Ok(data) = load_cached_file(&cached_filename) {
        Ok(Box::new(callback(&data)) as BoxAnySend)
    } else {
        let data = get_from_remote(&url).unwrap();
        Ok(Box::new(callback(&data)) as BoxAnySend)
    }
}

fn fetch_party_job(data: BoxAnySend) -> JobResult<BoxAnySend> {
    fetch_data_string(data, |string| parse_party(string))
}

fn fetch_production_entry_job(data: BoxAnySend) -> JobResult<BoxAnySend> {
    fetch_data_string(data, |string| parse_production_entry(string))
}

enum FetchItem {
    Release((u64, String)),
    Screenshot((u64, String)),
}

struct OnlineDemoContentProvider {
    active_party: Option<Party>,
    state: State,
    party_show_id: u64,
    url_string: String,
    time: std::time::Instant,
    load_party_handle: Option<JobHandle>,
    fetch_queue: Vec<FetchItem>,
}

impl OnlineDemoContentProvider {
    pub(crate) fn new() -> Self {
        fs::create_dir_all(CACHE_DIR).expect("Failed to create cache directory");
        Self {
            active_party: None,
            state: State::FetchParty,
            party_show_id: 92,
            url_string: String::with_capacity(128),
            load_party_handle: None,
            time: std::time::Instant::now(),
            fetch_queue: Vec::new(),
        }
    }

    pub fn select_party(&mut self, party_id: u64) {
        self.party_show_id = party_id;
    }

    pub fn update(&mut self, ui: &Ui) {
        match self.state {
            State::Idle => {}

            State::FetchParty => {
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
    }
}

impl ContentProvider for OnlineDemoContentProvider {
    fn get_item(&self, row: u64, col: u64) -> Item {
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

        Item {
            unselected_image: 0,
            selected_image: 0,
            id: u64::MAX / 2,
        }
    }

    fn get_column_count(&self, row: u64) -> u64 {
        if self.state == State::ShowParty {
            if let Some(party) = self.active_party.as_ref() {
                return party.competitions[row as usize].results.len() as u64;
            }
        }

        0
    }

    fn get_total_row_count(&self) -> u64 {
        if self.state == State::ShowParty {
            if let Some(party) = self.active_party.as_ref() {
                return party.competitions.len() as u64;
            }
        }

        0
    }

    fn get_row_name(&self, row: u64) -> &str {
        if self.state == State::ShowParty {
            if let Some(party) = self.active_party.as_ref() {
                return &party.competitions[row as usize].name;
            }
        }

        ""
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
