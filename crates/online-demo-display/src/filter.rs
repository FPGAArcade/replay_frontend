use crate::data::{Party, PartySeries, Platform, ProductionType, Release};
use chrono::{Datelike, NaiveDate};
use std::collections::HashSet;

/// Filter criteria for demo content
#[derive(Debug, Default, Clone)]
pub struct DemoFilter {
    pub platforms: Option<HashSet<String>>, // Platform names
    pub production_types: Option<HashSet<String>>, // Production type names
    pub party_names: Option<HashSet<String>>, // Party names
    pub years: Option<std::ops::RangeInclusive<i32>>,
    pub groups: Option<HashSet<String>>, // Author affiliation names
    pub authors: Option<HashSet<String>>, // Author names
    pub tags: Option<HashSet<String>>,
    pub random_selection: Option<usize>,
}

impl DemoFilter {
    /// Creates a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a platform name to filter by
    pub fn with_platform(mut self, platform: String) -> Self {
        self.platforms
            .get_or_insert_with(HashSet::new)
            .insert(platform);
        self
    }

    /// Add a production type to filter by
    pub fn with_production_type(mut self, prod_type: String) -> Self {
        self.production_types
            .get_or_insert_with(HashSet::new)
            .insert(prod_type);
        self
    }

    /// Add a party name to filter by
    pub fn with_party(mut self, party: String) -> Self {
        self.party_names
            .get_or_insert_with(HashSet::new)
            .insert(party);
        self
    }

    /// Set a year range to filter by
    pub fn with_year_range(mut self, start: i32, end: i32) -> Self {
        self.years = Some(start..=end);
        self
    }

    /// Add a group name to filter by
    pub fn with_group(mut self, group: String) -> Self {
        self.groups.get_or_insert_with(HashSet::new).insert(group);
        self
    }

    /// Add an author name to filter by
    pub fn with_author(mut self, author: String) -> Self {
        self.authors.get_or_insert_with(HashSet::new).insert(author);
        self
    }

    /// Add a tag to filter by
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.get_or_insert_with(HashSet::new).insert(tag);
        self
    }

    /// Set the number of random entries to select
    pub fn with_random_selection(mut self, count: usize) -> Self {
        self.random_selection = Some(count);
        self
    }

    /// Check if a release matches this filter's criteria
    pub fn matches_release(&self, release: &Release, party: Option<&Party>) -> bool {
        // Check platforms
        if let Some(platforms) = &self.platforms {
            if !release
                .platforms
                .iter()
                .any(|p| platforms.contains(&p.name))
            {
                return false;
            }
        }

        // Check production types
        if let Some(prod_types) = &self.production_types {
            if !release.types.iter().any(|t| prod_types.contains(&t.name)) {
                return false;
            }
        }

        // Check party name if a party context is provided
        if let Some(party_names) = &self.party_names {
            match party {
                Some(party) => {
                    if !party_names.contains(&party.name) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Check year
        if let Some(years) = &self.years {
            if let Ok(date) = NaiveDate::parse_from_str(&release.release_date, "%Y-%m-%d") {
                if !years.contains(&date.year()) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check groups (author affiliations)
        if let Some(groups) = &self.groups {
            if !release
                .author_affiliation_nicks
                .iter()
                .any(|n| groups.contains(&n.name))
            {
                return false;
            }
        }

        // Check authors
        if let Some(authors) = &self.authors {
            if !release
                .author_nicks
                .iter()
                .any(|n| authors.contains(&n.name))
            {
                return false;
            }
        }

        // Check tags
        if let Some(tags) = &self.tags {
            if !tags.iter().all(|tag| release.tags.contains(tag)) {
                return false;
            }
        }

        true
    }
}

/// Functions for filtering collections of releases
pub fn filter_releases<'a>(
    releases: impl IntoIterator<Item = &'a Release>,
    filter: &DemoFilter,
    party: Option<&'a Party>,
) -> Vec<&'a Release> {
    let mut filtered: Vec<&Release> = releases
        .into_iter()
        .filter(|release| filter.matches_release(release, party))
        .collect();

    /*
    // Handle random selection if specified
    if let Some(count) = filter.random_selection {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        filtered.shuffle(&mut rng);
        filtered.truncate(count);
    }

     */

    filtered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AuthorNick, ProductionType};

    fn create_test_release() -> Release {
        Release {
            url: "https://example.com".to_string(),
            demozoo_url: "https://demozoo.org/123".to_string(),
            id: 123,
            title: "Test Demo".to_string(),
            author_nicks: vec![AuthorNick {
                name: "Purple Motion".to_string(),
            }],
            author_affiliation_nicks: vec![AuthorNick {
                name: "Future Crew".to_string(),
            }],
            release_date: "1993-12-27".to_string(),
            supertype: "production".to_string(),
            platforms: vec![Platform {
                url: "https://example.com".to_string(),
                id: 1,
                name: "Amiga".to_string(),
            }],
            types: vec![ProductionType {
                url: "https://example.com".to_string(),
                id: 1,
                name: "Demo".to_string(),
                supertype: "production".to_string(),
            }],
            tags: vec!["demo".to_string(), "amiga".to_string()],
        }
    }

    fn create_test_party() -> Party {
        Party {
            url: "https://example.com".to_string(),
            demozoo_url: "https://demozoo.org/123".to_string(),
            id: 123,
            name: "The Party".to_string(),
            tagline: "The Original Party".to_string(),
            party_series: PartySeries {
                url: "https://example.com".to_string(),
                demozoo_url: "https://demozoo.org/123".to_string(),
                id: 1,
                name: "The Party".to_string(),
                website: "https://example.com".to_string(),
            },
            start_date: "1993-12-27".to_string(),
            end_date: "1993-12-29".to_string(),
            location: "Denmark".to_string(),
            is_online: false,
            country_code: "DK".to_string(),
            latitude: 55.676098,
            longitude: 12.568337,
            website: "https://example.com".to_string(),
            invitations: vec![],
            releases: vec![],
            competitions: vec![],
        }
    }

    #[test]
    fn test_platform_filter() {
        let release = create_test_release();

        let filter = DemoFilter::new().with_platform("Amiga".to_string());
        assert!(filter.matches_release(&release, None));

        let filter = DemoFilter::new().with_platform("Atari ST".to_string());
        assert!(!filter.matches_release(&release, None));
    }

    #[test]
    fn test_production_type_filter() {
        let release = create_test_release();

        let filter = DemoFilter::new().with_production_type("Demo".to_string());
        assert!(filter.matches_release(&release, None));

        let filter = DemoFilter::new().with_production_type("Music".to_string());
        assert!(!filter.matches_release(&release, None));
    }

    #[test]
    fn test_party_filter() {
        let release = create_test_release();
        let party = create_test_party();

        let filter = DemoFilter::new().with_party("The Party".to_string());
        assert!(filter.matches_release(&release, Some(&party)));

        let filter = DemoFilter::new().with_party("Assembly".to_string());
        assert!(!filter.matches_release(&release, Some(&party)));
    }

    #[test]
    fn test_year_filter() {
        let release = create_test_release();

        let filter = DemoFilter::new().with_year_range(1993, 1994);
        assert!(filter.matches_release(&release, None));

        let filter = DemoFilter::new().with_year_range(1995, 1996);
        assert!(!filter.matches_release(&release, None));
    }

    #[test]
    fn test_combined_filters() {
        let release = create_test_release();
        let party = create_test_party();

        let filter = DemoFilter::new()
            .with_platform("Amiga".to_string())
            .with_production_type("Demo".to_string())
            .with_party("The Party".to_string())
            .with_year_range(1993, 1993)
            .with_tag("demo".to_string());

        assert!(filter.matches_release(&release, Some(&party)));
    }
}
