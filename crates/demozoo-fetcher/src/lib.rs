use serde::Deserialize;

/// The JSON “author_nicks” array is an array of objects. We only care about the name.
#[derive(Deserialize, Debug)]
struct AuthorNick {
    name: String,
    // Other fields (like abbreviation or releaser) are ignored.
}

/// The JSON “credits” array holds more detailed information about a production’s credits.
/// We use this struct to capture the “nick” (an AuthorNick) plus a category and role.
#[derive(Deserialize, Debug)]
struct Credit {
    nick: AuthorNick,
    category: String,
    role: String,
}

/// The JSON “download_links” array contains objects with a link class and a URL.
#[derive(Deserialize, Debug)]
struct DownloadLink {
    link_class: String,
    url: String,
}

/// The JSON “platforms” array gives details for each platform. (We only really care about the name.)
#[derive(Deserialize, Debug)]
struct Platform {
    url: String,
    id: u32,
    name: String,
}

/// The JSON “screenshots” array has keys like “original_url” which we map into our struct.
/// We use Serde’s rename attribute to change the JSON key into our field name.
#[derive(Deserialize, Debug)]
struct Screenshot {
    #[serde(rename = "original_url")]
    original_url: String,
    original_width: u32,
    original_height: u32,
    #[serde(rename = "standard_url")]
    standard_url: String,
    standard_width: u32,
    standard_height: u32,
    #[serde(rename = "thumbnail_url")]
    thumbnail_url: String,
    thumbnail_width: u32,
    thumbnail_height: u32,
}

/// The main ProductionEntry struct gathers the fields we care about from the JSON.
/// We use `#[serde(default)]` for arrays so that missing fields (if any) don’t cause an error.
#[derive(Deserialize, Debug)]
struct ProductionEntry {
    title: String,
    release_date: String,
    #[serde(rename = "author_nicks")]
    author_nicks: Vec<AuthorNick>,
    #[serde(default)]
    credits: Vec<Credit>,
    #[serde(default)]
    download_links: Vec<DownloadLink>,
    #[serde(default)]
    platforms: Vec<Platform>,
    #[serde(default)]
    screenshots: Vec<Screenshot>,
    #[serde(default)]
    tags: Vec<String>,
}

pub fn parse_json(json_data: &str) -> ProductionEntry {
    serde_json::from_str(json_data).expect("Failed to parse JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_production_entry() {
        let data = std::fs::read_to_string("../../crates/demozoo-fetcher/test-data/2.json")
            .expect("Unable to read file");

        let entry: ProductionEntry =
            serde_json::from_str(&data).expect("Failed to deserialize JSON");

        // Verify that the basic fields were deserialized correctly.
        assert_eq!(entry.title, "State of the Art");
        assert_eq!(entry.release_date, "1992-12-29");

        // The author_nicks array should contain one entry with the name "Spaceballs".
        assert_eq!(entry.author_nicks.len(), 1);
        assert_eq!(entry.author_nicks[0].name, "Spaceballs");

        // There should be 4 credit entries; verify that one of them is for Music ("Travolta").
        assert_eq!(entry.credits.len(), 4);
        let music_credit = entry
            .credits
            .iter()
            .find(|credit| credit.category == "Music")
            .expect("Missing music credit");
        assert_eq!(music_credit.nick.name, "Travolta");

        // There should be 4 download links.
        assert_eq!(entry.download_links.len(), 4);

        // The platforms array should have one platform with name "Amiga OCS/ECS".
        assert_eq!(entry.platforms.len(), 1);
        assert_eq!(entry.platforms[0].name, "Amiga OCS/ECS");

        // According to the sample JSON, there should be 23 screenshots.
        assert_eq!(entry.screenshots.len(), 23);

        // And there should be 3 tags.
        assert_eq!(entry.tags.len(), 3);
    }
}
