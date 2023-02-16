use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

static VALID_ACTOR_NAME_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_]{3,}$").expect("compile regex"));
static VALID_POST_TITLE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r".*\S{3,}.*").expect("compile regex"));
static VALID_MATRIX_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^@[A-Za-z0-9._=-]+:[A-Za-z0-9.-]+\.[A-Za-z]{2,}$").expect("compile regex")
});
// taken from https://en.wikipedia.org/wiki/UTM_parameters
static CLEAN_URL_PARAMS_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^utm_source|utm_medium|utm_campaign|utm_term|utm_content|gclid|gclsrc|dclid|fbclid$")
    .expect("compile regex")
});

fn has_newline(name: &str) -> bool {
  name.contains('\n')
}

pub fn is_valid_actor_name(name: &str, actor_name_max_length: usize) -> bool {
  name.chars().count() <= actor_name_max_length
    && VALID_ACTOR_NAME_REGEX.is_match(name)
    && !has_newline(name)
}

// Can't do a regex here, reverse lookarounds not supported
pub fn is_valid_display_name(name: &str, actor_name_max_length: usize) -> bool {
  !name.starts_with('@')
    && !name.starts_with('\u{200b}')
    && name.chars().count() >= 3
    && name.chars().count() <= actor_name_max_length
    && !has_newline(name)
}

pub fn is_valid_matrix_id(matrix_id: &str) -> bool {
  VALID_MATRIX_ID_REGEX.is_match(matrix_id) && !has_newline(matrix_id)
}

pub fn is_valid_post_title(title: &str) -> bool {
  VALID_POST_TITLE_REGEX.is_match(title) && !has_newline(title)
}

pub fn clean_url_params(url: &Url) -> Url {
  let mut url_out = url.clone();
  if url.query().is_some() {
    let new_query = url
      .query_pairs()
      .filter(|q| !CLEAN_URL_PARAMS_REGEX.is_match(&q.0))
      .map(|q| format!("{}={}", q.0, q.1))
      .join("&");
    url_out.set_query(Some(&new_query));
  }
  url_out
}

#[cfg(test)]
mod tests {
  use crate::utils::validation::{
    clean_url_params,
    is_valid_actor_name,
    is_valid_display_name,
    is_valid_matrix_id,
    is_valid_post_title,
  };
  use url::Url;

  #[test]
  fn test_clean_url_params() {
    let url = Url::parse("https://example.com/path/123?utm_content=buffercf3b2&utm_medium=social&username=randomuser&id=123").unwrap();
    let cleaned = clean_url_params(&url);
    let expected = Url::parse("https://example.com/path/123?username=randomuser&id=123").unwrap();
    assert_eq!(expected.to_string(), cleaned.to_string());

    let url = Url::parse("https://example.com/path/123").unwrap();
    let cleaned = clean_url_params(&url);
    assert_eq!(url.to_string(), cleaned.to_string());
  }

  #[test]
  fn regex_checks() {
    assert!(!is_valid_post_title("hi"));
    assert!(is_valid_post_title("him"));
    assert!(!is_valid_post_title("n\n\n\n\nanother"));
    assert!(!is_valid_post_title("hello there!\n this is a test."));
    assert!(is_valid_post_title("hello there! this is a test."));
  }

  #[test]
  fn test_valid_actor_name() {
    let actor_name_max_length = 20;
    assert!(is_valid_actor_name("Hello_98", actor_name_max_length));
    assert!(is_valid_actor_name("ten", actor_name_max_length));
    assert!(!is_valid_actor_name("Hello-98", actor_name_max_length));
    assert!(!is_valid_actor_name("a", actor_name_max_length));
    assert!(!is_valid_actor_name("", actor_name_max_length));
  }

  #[test]
  fn test_valid_display_name() {
    let actor_name_max_length = 20;
    assert!(is_valid_display_name("hello @there", actor_name_max_length));
    assert!(!is_valid_display_name(
      "@hello there",
      actor_name_max_length
    ));

    // Make sure zero-space with an @ doesn't work
    assert!(!is_valid_display_name(
      &format!("{}@my name is", '\u{200b}'),
      actor_name_max_length
    ));
  }

  #[test]
  fn test_valid_post_title() {
    assert!(is_valid_post_title("Post Title"));
    assert!(is_valid_post_title("   POST TITLE 😃😃😃😃😃"));
    assert!(!is_valid_post_title("\n \n \n \n    		")); // tabs/spaces/newlines
  }

  #[test]
  fn test_valid_matrix_id() {
    assert!(is_valid_matrix_id("@dess:matrix.org"));
    assert!(!is_valid_matrix_id("dess:matrix.org"));
    assert!(!is_valid_matrix_id(" @dess:matrix.org"));
    assert!(!is_valid_matrix_id("@dess:matrix.org t"));
  }
}
