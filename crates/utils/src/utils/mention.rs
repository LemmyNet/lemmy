use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;

static MENTIONS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"@(?P<name>[\w.]+)@(?P<domain>[a-zA-Z0-9._:-]+)").expect("compile regex")
});
// TODO nothing is done with community / group webfingers yet, so just ignore those for now
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MentionData {
    pub name: String,
    pub domain: String,
}

impl MentionData {
    pub fn is_local(&self, hostname: &str) -> bool {
        hostname.eq(&self.domain)
    }
    pub fn full_name(&self) -> String {
        format!("@{}@{}", &self.name, &self.domain)
    }
}

pub fn scrape_text_for_mentions(text: &str) -> Vec<MentionData> {
    let mut out: Vec<MentionData> = Vec::new();
    for caps in MENTIONS_REGEX.captures_iter(text) {
        if let Some(name) = caps.name("name").map(|c| c.as_str().to_string()) {
            if let Some(domain) = caps.name("domain").map(|c| c.as_str().to_string()) {
                out.push(MentionData { name, domain });
            }
        }
    }
    out.into_iter().unique().collect()
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::indexing_slicing)]

    use crate::utils::mention::scrape_text_for_mentions;

    #[test]
    fn test_mentions_regex() {
        let text = "Just read a great blog post by [@tedu@honk.teduangst.com](/u/test). And another by !test_community@fish.teduangst.com . Another [@lemmy@lemmy-alpha:8540](/u/fish)";
        let mentions = scrape_text_for_mentions(text);

        assert_eq!(mentions[0].name, "tedu".to_string());
        assert_eq!(mentions[0].domain, "honk.teduangst.com".to_string());
        assert_eq!(mentions[1].domain, "lemmy-alpha:8540".to_string());
    }
}
