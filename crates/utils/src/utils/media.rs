use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use crate::error::{LemmyError, LemmyResult};

static IMAGE_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(http)?s?:?(\/\/.*\.(?:jpg|jpeg|gif|png|svg|webp))").expect("compile regex")
});

pub fn is_url_image(url: &str) -> LemmyResult<()> {
    if !IMAGE_URL_REGEX.is_match(url) {
        Err(LemmyError::from_message("invalid_post_title"))
    } else {
        Ok(())
    }
}