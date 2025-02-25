use nanoserde::DeJson;

#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct AuthorNick {
    pub name: String,
}

#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct Credit {
    pub nick: AuthorNick,
    pub category: String,
    pub role: String,
}

#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct DownloadLink {
    pub link_class: String,
    pub url: String,
}

#[derive(DeJson, Debug)]
#[allow(dead_code)]
pub struct Platform {
    pub url: String,
    pub id: u32,
    pub name: String,
}

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
    pub shown_date: Option<String>,
    pub platform: Option<Platform>,
    pub production_type: Option<ProductionType>,
    pub results: Vec<Result>,
}

#[derive(DeJson, Debug)]
pub struct Result {
    pub position: i32,
    pub ranking: String,
    pub score: String,
    pub production: Release,
}
