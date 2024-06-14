use crate::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder, RegexSet};
use url::{ParseError, Url};

// From here: https://github.com/vector-im/element-android/blob/develop/matrix-sdk-android/src/main/java/org/matrix/android/sdk/api/MatrixPatterns.kt#L35
static VALID_MATRIX_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^@[A-Za-z0-9\x21-\x39\x3B-\x7F]+:[A-Za-z0-9.-]+(:[0-9]{2,5})?$")
    .expect("compile regex")
});
// taken from https://en.wikipedia.org/wiki/UTM_parameters
static CLEAN_URL_PARAMS_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(
    r"^(utm_source|utm_medium|utm_campaign|utm_term|utm_content|gclid|gclsrc|dclid|fbclid)=",
  )
  .expect("compile regex")
});
const ALLOWED_POST_URL_SCHEMES: [&str; 3] = ["http", "https", "magnet"];

const BODY_MAX_LENGTH: usize = 10000;
const POST_BODY_MAX_LENGTH: usize = 50000;
const BIO_MAX_LENGTH: usize = 300;
const ALT_TEXT_MAX_LENGTH: usize = 1500;
const SITE_NAME_MAX_LENGTH: usize = 20;
const SITE_NAME_MIN_LENGTH: usize = 1;
const SITE_DESCRIPTION_MAX_LENGTH: usize = 150;
//Invisible unicode characters, taken from https://invisible-characters.com/
const FORBIDDEN_DISPLAY_CHARS: [char; 53] = [
  '\u{0009}',
  '\u{00a0}',
  '\u{00ad}',
  '\u{034f}',
  '\u{061c}',
  '\u{115f}',
  '\u{1160}',
  '\u{17b4}',
  '\u{17b5}',
  '\u{180e}',
  '\u{2000}',
  '\u{2001}',
  '\u{2002}',
  '\u{2003}',
  '\u{2004}',
  '\u{2005}',
  '\u{2006}',
  '\u{2007}',
  '\u{2008}',
  '\u{2009}',
  '\u{200a}',
  '\u{200b}',
  '\u{200c}',
  '\u{200d}',
  '\u{200e}',
  '\u{200f}',
  '\u{202f}',
  '\u{205f}',
  '\u{2060}',
  '\u{2061}',
  '\u{2062}',
  '\u{2063}',
  '\u{2064}',
  '\u{206a}',
  '\u{206b}',
  '\u{206c}',
  '\u{206d}',
  '\u{206e}',
  '\u{206f}',
  '\u{3000}',
  '\u{2800}',
  '\u{3164}',
  '\u{feff}',
  '\u{ffa0}',
  '\u{1d159}',
  '\u{1d173}',
  '\u{1d174}',
  '\u{1d175}',
  '\u{1d176}',
  '\u{1d177}',
  '\u{1d178}',
  '\u{1d179}',
  '\u{1d17a}',
];

fn has_newline(name: &str) -> bool {
  name.contains('\n')
}

pub fn is_valid_actor_name(name: &str, actor_name_max_length: usize) -> LemmyResult<()> {
  static VALID_ACTOR_NAME_REGEX_EN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_]{3,}$").expect("compile regex"));
  static VALID_ACTOR_NAME_REGEX_AR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\p{Arabic}0-9_]{3,}$").expect("compile regex"));
  static VALID_ACTOR_NAME_REGEX_RU: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\p{Cyrillic}0-9_]{3,}$").expect("compile regex"));

  let check = name.chars().count() <= actor_name_max_length && !has_newline(name);

  // Only allow characters from a single alphabet per username. This avoids problems with lookalike
  // characters like `o` which looks identical in Latin and Cyrillic, and can be used to imitate
  // other users. Checks for additional alphabets can be added in the same way.
  let lang_check = VALID_ACTOR_NAME_REGEX_EN.is_match(name)
    || VALID_ACTOR_NAME_REGEX_AR.is_match(name)
    || VALID_ACTOR_NAME_REGEX_RU.is_match(name);

  if !check || !lang_check {
    Err(LemmyErrorType::InvalidName.into())
  } else {
    Ok(())
  }
}

fn has_3_permitted_display_chars(name: &str) -> bool {
  let mut num_non_fdc: i8 = 0;
  for c in name.chars() {
    if !FORBIDDEN_DISPLAY_CHARS.contains(&c) {
      num_non_fdc += 1;
      if num_non_fdc >= 3 {
        break;
      }
    }
  }
  if num_non_fdc >= 3 {
    return true;
  }
  false
}

// Can't do a regex here, reverse lookarounds not supported
pub fn is_valid_display_name(name: &str, actor_name_max_length: usize) -> LemmyResult<()> {
  let check = !name.starts_with('@')
    && !name.starts_with(FORBIDDEN_DISPLAY_CHARS)
    && name.chars().count() <= actor_name_max_length
    && !has_newline(name)
    && has_3_permitted_display_chars(name);
  if !check {
    Err(LemmyErrorType::InvalidDisplayName.into())
  } else {
    Ok(())
  }
}

pub fn is_valid_matrix_id(matrix_id: &str) -> LemmyResult<()> {
  let check = VALID_MATRIX_ID_REGEX.is_match(matrix_id) && !has_newline(matrix_id);
  if !check {
    Err(LemmyErrorType::InvalidMatrixId.into())
  } else {
    Ok(())
  }
}

pub fn is_valid_post_title(title: &str) -> LemmyResult<()> {
  let length = title.trim().chars().count();
  let check = (3..=200).contains(&length) && !has_newline(title);
  if !check {
    Err(LemmyErrorType::InvalidPostTitle.into())
  } else {
    Ok(())
  }
}

/// This could be post bodies, comments, or any description field
pub fn is_valid_body_field(body: &str, post: bool) -> LemmyResult<()> {
  if post {
    max_length_check(body, POST_BODY_MAX_LENGTH, LemmyErrorType::InvalidBodyField)?;
  } else {
    max_length_check(body, BODY_MAX_LENGTH, LemmyErrorType::InvalidBodyField)?;
  };
  Ok(())
}

pub fn is_valid_bio_field(bio: &str) -> LemmyResult<()> {
  max_length_check(bio, BIO_MAX_LENGTH, LemmyErrorType::BioLengthOverflow)
}

pub fn is_valid_alt_text_field(alt_text: &str) -> LemmyResult<()> {
  max_length_check(
    alt_text,
    ALT_TEXT_MAX_LENGTH,
    LemmyErrorType::AltTextLengthOverflow,
  )?;

  Ok(())
}

/// Checks the site name length, the limit as defined in the DB.
pub fn site_name_length_check(name: &str) -> LemmyResult<()> {
  min_length_check(name, SITE_NAME_MIN_LENGTH, LemmyErrorType::SiteNameRequired)?;
  max_length_check(
    name,
    SITE_NAME_MAX_LENGTH,
    LemmyErrorType::SiteNameLengthOverflow,
  )
}

/// Checks the site description length, the limit as defined in the DB.
pub fn site_description_length_check(description: &str) -> LemmyResult<()> {
  max_length_check(
    description,
    SITE_DESCRIPTION_MAX_LENGTH,
    LemmyErrorType::SiteDescriptionLengthOverflow,
  )
}

/// Check minimum and maximum length of input string. If the string is too short or too long, the
/// corresponding error is returned.
///
/// HTML frontends specify maximum input length using `maxlength` attribute.
/// For consistency we use the same counting method (UTF-16 code units).
/// https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes/maxlength
fn max_length_check(item: &str, max_length: usize, max_msg: LemmyErrorType) -> LemmyResult<()> {
  let len = item.encode_utf16().count();
  if len > max_length {
    Err(max_msg.into())
  } else {
    Ok(())
  }
}

fn min_length_check(item: &str, min_length: usize, min_msg: LemmyErrorType) -> LemmyResult<()> {
  let len = item.encode_utf16().count();
  if len < min_length {
    Err(min_msg.into())
  } else {
    Ok(())
  }
}

/// Attempts to build a regex and check it for common errors before inserting into the DB.
pub fn build_and_check_regex(regex_str_opt: &Option<&str>) -> LemmyResult<Option<Regex>> {
  regex_str_opt.map_or_else(
    || Ok(None::<Regex>),
    |regex_str| {
      if regex_str.is_empty() {
        // If the proposed regex is empty, return as having no regex at all; this is the same
        // behavior that happens downstream before the write to the database.
        return Ok(None::<Regex>);
      }

      RegexBuilder::new(regex_str)
        .case_insensitive(true)
        .build()
        .with_lemmy_type(LemmyErrorType::InvalidRegex)
        .and_then(|regex| {
          // NOTE: It is difficult to know, in the universe of user-crafted regex, which ones
          // may match against any string text. To keep it simple, we'll match the regex
          // against an innocuous string - a single number - which should help catch a regex
          // that accidentally matches against all strings.
          if regex.is_match("1") {
            Err(LemmyErrorType::PermissiveRegex.into())
          } else {
            Ok(Some(regex))
          }
        })
    },
  )
}

pub fn clean_url_params(url: &Url) -> Url {
  let mut url_out = url.clone();
  if let Some(query) = url.query() {
    let new_query = query
      .split_inclusive('&')
      .filter(|q| !CLEAN_URL_PARAMS_REGEX.is_match(q))
      .collect::<String>();
    url_out.set_query(Some(&new_query));
  }
  url_out
}

pub fn check_site_visibility_valid(
  current_private_instance: bool,
  current_federation_enabled: bool,
  new_private_instance: &Option<bool>,
  new_federation_enabled: &Option<bool>,
) -> LemmyResult<()> {
  let private_instance = new_private_instance.unwrap_or(current_private_instance);
  let federation_enabled = new_federation_enabled.unwrap_or(current_federation_enabled);

  if private_instance && federation_enabled {
    Err(LemmyErrorType::CantEnablePrivateInstanceAndFederationTogether.into())
  } else {
    Ok(())
  }
}

pub fn check_url_scheme(url: &Url) -> LemmyResult<()> {
  if !ALLOWED_POST_URL_SCHEMES.contains(&url.scheme()) {
    Err(LemmyErrorType::InvalidUrlScheme)?
  }

  Ok(())
}

pub fn is_url_blocked(url: &Url, blocklist: &RegexSet) -> LemmyResult<()> {
  if blocklist.is_match(url.as_str()) {
    Err(LemmyErrorType::BlockedUrl)?
  }

  Ok(())
}

/// Check that urls are valid, and also remove the scheme, and uniques
pub fn check_urls_are_valid(urls: &Vec<String>) -> LemmyResult<Vec<String>> {
  let mut parsed_urls = vec![];
  for url in urls {
    parsed_urls.push(build_url_str_without_scheme(url)?);
  }

  let unique_urls = parsed_urls.into_iter().unique().collect();
  Ok(unique_urls)
}

pub fn build_url_str_without_scheme(url_str: &str) -> LemmyResult<String> {
  // Parse and check for errors
  let mut url = Url::parse(url_str).or_else(|e| {
    if e == ParseError::RelativeUrlWithoutBase {
      Url::parse(&format!("http://{url_str}"))
    } else {
      Err(e)
    }
  })?;

  // Set the scheme to http, then remove the http:// part
  url
    .set_scheme("http")
    .map_err(|_| LemmyErrorType::InvalidUrl)?;

  let mut out = url
    .to_string()
    .get(7..)
    .ok_or(LemmyErrorType::InvalidUrl)?
    .to_string();

  // Remove trailing / if necessary
  if out.ends_with('/') {
    out.pop();
  }

  Ok(out)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{
    error::{LemmyErrorType, LemmyResult},
    utils::validation::{
      build_and_check_regex,
      check_site_visibility_valid,
      check_url_scheme,
      check_urls_are_valid,
      clean_url_params,
      is_url_blocked,
      is_valid_actor_name,
      is_valid_bio_field,
      is_valid_display_name,
      is_valid_matrix_id,
      is_valid_post_title,
      site_description_length_check,
      site_name_length_check,
      BIO_MAX_LENGTH,
      SITE_DESCRIPTION_MAX_LENGTH,
      SITE_NAME_MAX_LENGTH,
    },
  };
  use pretty_assertions::assert_eq;
  use url::Url;

  #[test]
  fn test_clean_url_params() -> LemmyResult<()> {
    let url = Url::parse("https://example.com/path/123?utm_content=buffercf3b2&utm_medium=social&user+name=random+user%20&id=123")?;
    let cleaned = clean_url_params(&url);
    let expected = Url::parse("https://example.com/path/123?user+name=random+user%20&id=123")?;
    assert_eq!(expected.to_string(), cleaned.to_string());

    let url = Url::parse("https://example.com/path/123")?;
    let cleaned = clean_url_params(&url);
    assert_eq!(url.to_string(), cleaned.to_string());

    Ok(())
  }

  #[test]
  fn regex_checks() {
    assert!(is_valid_post_title("hi").is_err());
    assert!(is_valid_post_title("him").is_ok());
    assert!(is_valid_post_title("  him  ").is_ok());
    assert!(is_valid_post_title("n\n\n\n\nanother").is_err());
    assert!(is_valid_post_title("hello there!\n this is a test.").is_err());
    assert!(is_valid_post_title("hello there! this is a test.").is_ok());
    assert!(is_valid_post_title(("12345".repeat(40) + "x").as_str()).is_err());
    assert!(is_valid_post_title("12345".repeat(40).as_str()).is_ok());
    assert!(is_valid_post_title((("12345".repeat(40)) + "  ").as_str()).is_ok());
  }

  #[test]
  fn test_valid_actor_name() {
    let actor_name_max_length = 20;
    assert!(is_valid_actor_name("Hello_98", actor_name_max_length).is_ok());
    assert!(is_valid_actor_name("ten", actor_name_max_length).is_ok());
    assert!(is_valid_actor_name("تجريب", actor_name_max_length).is_ok());
    assert!(is_valid_actor_name("تجريب_123", actor_name_max_length).is_ok());
    assert!(is_valid_actor_name("Владимир", actor_name_max_length).is_ok());

    // mixed scripts
    assert!(is_valid_actor_name("تجريب_abc", actor_name_max_length).is_err());
    assert!(is_valid_actor_name("Влад_abc", actor_name_max_length).is_err());
    // dash
    assert!(is_valid_actor_name("Hello-98", actor_name_max_length).is_err());
    // too short
    assert!(is_valid_actor_name("a", actor_name_max_length).is_err());
    // empty
    assert!(is_valid_actor_name("", actor_name_max_length).is_err());
  }

  #[test]
  fn test_valid_display_name() {
    let actor_name_max_length = 20;
    assert!(is_valid_display_name("hello @there", actor_name_max_length).is_ok());
    assert!(is_valid_display_name("@hello there", actor_name_max_length).is_err());
    assert!(is_valid_display_name("\u{200d}hello", actor_name_max_length).is_err());
    assert!(is_valid_display_name(
      "\u{1f3f3}\u{fe0f}\u{200d}\u{26a7}\u{fe0f}Name",
      actor_name_max_length
    )
    .is_ok());
    assert!(is_valid_display_name("\u{2003}1\u{ffa0}2\u{200d}", actor_name_max_length).is_err());

    // Make sure zero-space with an @ doesn't work
    assert!(
      is_valid_display_name(&format!("{}@my name is", '\u{200b}'), actor_name_max_length).is_err()
    );
  }

  #[test]
  fn test_valid_post_title() {
    assert!(is_valid_post_title("Post Title").is_ok());
    assert!(is_valid_post_title(
      "აშშ ითხოვს ირანს დაუყოვნებლივ გაანთავისუფლოს დაკავებული ნავთობის ტანკერი"
    )
    .is_ok());
    assert!(is_valid_post_title("   POST TITLE 😃😃😃😃😃").is_ok());
    assert!(is_valid_post_title("\n \n \n \n    		").is_err()); // tabs/spaces/newlines
  }

  #[test]
  fn test_valid_matrix_id() {
    assert!(is_valid_matrix_id("@dess:matrix.org").is_ok());
    assert!(is_valid_matrix_id("@dess_:matrix.org").is_ok());
    assert!(is_valid_matrix_id("@dess:matrix.org:443").is_ok());
    assert!(is_valid_matrix_id("dess:matrix.org").is_err());
    assert!(is_valid_matrix_id(" @dess:matrix.org").is_err());
    assert!(is_valid_matrix_id("@dess:matrix.org t").is_err());
    assert!(is_valid_matrix_id("@dess:matrix.org t").is_err());
  }

  #[test]
  fn test_valid_site_name() -> LemmyResult<()> {
    let valid_names = [
      (0..SITE_NAME_MAX_LENGTH).map(|_| 'A').collect::<String>(),
      String::from("A"),
    ];
    let invalid_names = [
      (
        &(0..SITE_NAME_MAX_LENGTH + 1)
          .map(|_| 'A')
          .collect::<String>(),
        LemmyErrorType::SiteNameLengthOverflow,
      ),
      (&String::new(), LemmyErrorType::SiteNameRequired),
    ];

    valid_names.iter().for_each(|valid_name| {
      assert!(
        site_name_length_check(valid_name).is_ok(),
        "Expected {} of length {} to be Ok.",
        valid_name,
        valid_name.len()
      )
    });

    invalid_names
      .iter()
      .for_each(|(invalid_name, expected_err)| {
        let result = site_name_length_check(invalid_name);

        assert!(result.is_err());
        assert!(
          result.is_err_and(|e| e.error_type.eq(&expected_err.clone())),
          "Testing {}, expected error {}",
          invalid_name,
          expected_err
        );
      });
    Ok(())
  }

  #[test]
  fn test_valid_bio() {
    assert!(is_valid_bio_field(&(0..BIO_MAX_LENGTH).map(|_| 'A').collect::<String>()).is_ok());

    let invalid_result =
      is_valid_bio_field(&(0..BIO_MAX_LENGTH + 1).map(|_| 'A').collect::<String>());

    assert!(
      invalid_result.is_err()
        && invalid_result.is_err_and(|e| e.error_type.eq(&LemmyErrorType::BioLengthOverflow))
    );
  }

  #[test]
  fn test_valid_site_description() {
    assert!(site_description_length_check(
      &(0..SITE_DESCRIPTION_MAX_LENGTH)
        .map(|_| 'A')
        .collect::<String>()
    )
    .is_ok());

    let invalid_result = site_description_length_check(
      &(0..SITE_DESCRIPTION_MAX_LENGTH + 1)
        .map(|_| 'A')
        .collect::<String>(),
    );

    assert!(
      invalid_result.is_err()
        && invalid_result.is_err_and(|e| e
          .error_type
          .eq(&LemmyErrorType::SiteDescriptionLengthOverflow))
    );
  }

  #[test]
  fn test_valid_slur_regex() {
    let valid_regexes = [&None, &Some(""), &Some("(foo|bar)")];

    valid_regexes.iter().for_each(|regex| {
      let result = build_and_check_regex(regex);

      assert!(result.is_ok(), "Testing regex: {:?}", regex);
    });
  }

  #[test]
  fn test_too_permissive_slur_regex() {
    let match_everything_regexes = [
      (&Some("["), LemmyErrorType::InvalidRegex),
      (&Some("(foo|bar|)"), LemmyErrorType::PermissiveRegex),
      (&Some(".*"), LemmyErrorType::PermissiveRegex),
    ];

    match_everything_regexes
      .iter()
      .for_each(|(regex_str, expected_err)| {
        let result = build_and_check_regex(regex_str);

        assert!(result.is_err());
        assert!(
          result.is_err_and(|e| e.error_type.eq(&expected_err.clone())),
          "Testing regex {:?}, expected error {}",
          regex_str,
          expected_err
        );
      });
  }

  #[test]
  fn test_check_site_visibility_valid() {
    assert!(check_site_visibility_valid(true, true, &None, &None).is_err());
    assert!(check_site_visibility_valid(true, false, &None, &Some(true)).is_err());
    assert!(check_site_visibility_valid(false, true, &Some(true), &None).is_err());
    assert!(check_site_visibility_valid(false, false, &Some(true), &Some(true)).is_err());
    assert!(check_site_visibility_valid(true, false, &None, &None).is_ok());
    assert!(check_site_visibility_valid(false, true, &None, &None).is_ok());
    assert!(check_site_visibility_valid(false, false, &Some(true), &None).is_ok());
    assert!(check_site_visibility_valid(false, false, &None, &Some(true)).is_ok());
  }

  #[test]
  fn test_check_url_scheme() -> LemmyResult<()> {
    assert!(check_url_scheme(&Url::parse("http://example.com")?).is_ok());
    assert!(check_url_scheme(&Url::parse("https://example.com")?).is_ok());
    assert!(check_url_scheme(&Url::parse("https://example.com")?).is_ok());
    assert!(check_url_scheme(&Url::parse("ftp://example.com")?).is_err());
    assert!(check_url_scheme(&Url::parse("javascript:void")?).is_err());

    let magnet_link="magnet:?xt=urn:btih:4b390af3891e323778959d5abfff4b726510f14c&dn=Ravel%20Complete%20Piano%20Sheet%20Music%20-%20Public%20Domain&tr=udp%3A%2F%2Fopen.tracker.cl%3A1337%2Fannounce";
    assert!(check_url_scheme(&Url::parse(magnet_link)?).is_ok());

    Ok(())
  }

  #[test]
  fn test_url_block() -> LemmyResult<()> {
    let set = regex::RegexSet::new(vec![
      r"(https://)?example\.org/page/to/article",
      r"(https://)?example\.net/?",
      r"(https://)?example\.com/?",
    ])?;

    assert!(is_url_blocked(&Url::parse("https://example.blog")?, &set).is_ok());

    assert!(is_url_blocked(&Url::parse("https://example.org")?, &set).is_ok());

    assert!(is_url_blocked(&Url::parse("https://example.com")?, &set).is_err());

    Ok(())
  }

  #[test]
  fn test_url_parsed() -> LemmyResult<()> {
    // Make sure the scheme is removed, and uniques also
    assert_eq!(
      &check_urls_are_valid(&vec![
        "example.com".to_string(),
        "http://example.com".to_string(),
        "https://example.com".to_string(),
        "https://example.com/test?q=test2&q2=test3#test4".to_string(),
      ])?,
      &vec![
        "example.com".to_string(),
        "example.com/test?q=test2&q2=test3#test4".to_string()
      ],
    );

    assert!(check_urls_are_valid(&vec!["https://example .com".to_string()]).is_err());
    Ok(())
  }
}
