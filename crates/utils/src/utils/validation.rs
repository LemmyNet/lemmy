use crate::error::{LemmyError, LemmyResult};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use totp_rs::{Secret, TOTP};
use url::Url;

static VALID_ACTOR_NAME_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_]{3,}$").expect("compile regex"));
static VALID_POST_TITLE_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r".*\S{3,200}.*").expect("compile regex"));
static VALID_MATRIX_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^@[A-Za-z0-9._=-]+:[A-Za-z0-9.-]+\.[A-Za-z]{2,}$").expect("compile regex")
});
// taken from https://en.wikipedia.org/wiki/UTM_parameters
static CLEAN_URL_PARAMS_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^utm_source|utm_medium|utm_campaign|utm_term|utm_content|gclid|gclsrc|dclid|fbclid$")
    .expect("compile regex")
});

const BODY_MAX_LENGTH: usize = 10000;
const POST_BODY_MAX_LENGTH: usize = 50000;
const BIO_MAX_LENGTH: usize = 300;
const SITE_NAME_MAX_LENGTH: usize = 20;
const SITE_NAME_MIN_LENGTH: usize = 1;
const SITE_DESCRIPTION_MAX_LENGTH: usize = 150;

fn has_newline(name: &str) -> bool {
  name.contains('\n')
}

pub fn is_valid_actor_name(name: &str, actor_name_max_length: usize) -> LemmyResult<()> {
  let check = name.chars().count() <= actor_name_max_length
    && VALID_ACTOR_NAME_REGEX.is_match(name)
    && !has_newline(name);
  if !check {
    Err(LemmyError::from_message("invalid_name"))
  } else {
    Ok(())
  }
}

// Can't do a regex here, reverse lookarounds not supported
pub fn is_valid_display_name(name: &str, actor_name_max_length: usize) -> LemmyResult<()> {
  let check = !name.starts_with('@')
    && !name.starts_with('\u{200b}')
    && name.chars().count() >= 3
    && name.chars().count() <= actor_name_max_length
    && !has_newline(name);
  if !check {
    Err(LemmyError::from_message("invalid_username"))
  } else {
    Ok(())
  }
}

pub fn is_valid_matrix_id(matrix_id: &str) -> LemmyResult<()> {
  let check = VALID_MATRIX_ID_REGEX.is_match(matrix_id) && !has_newline(matrix_id);
  if !check {
    Err(LemmyError::from_message("invalid_matrix_id"))
  } else {
    Ok(())
  }
}

pub fn is_valid_post_title(title: &str) -> LemmyResult<()> {
  let check = VALID_POST_TITLE_REGEX.is_match(title) && !has_newline(title);
  if !check {
    Err(LemmyError::from_message("invalid_post_title"))
  } else {
    Ok(())
  }
}

/// This could be post bodies, comments, or any description field
pub fn is_valid_body_field(body: &Option<String>, post: bool) -> LemmyResult<()> {
  if let Some(body) = body {
    let check = if post {
      body.chars().count() <= POST_BODY_MAX_LENGTH
    } else {
      body.chars().count() <= BODY_MAX_LENGTH
    };

    if !check {
      Err(LemmyError::from_message("invalid_body_field"))
    } else {
      Ok(())
    }
  } else {
    Ok(())
  }
}

pub fn is_valid_bio_field(bio: &str) -> LemmyResult<()> {
  max_length_check(bio, BIO_MAX_LENGTH, String::from("bio_length_overflow"))
}

/// Checks the site name length, the limit as defined in the DB.
pub fn site_name_length_check(name: &str) -> LemmyResult<()> {
  min_max_length_check(
    name,
    SITE_NAME_MIN_LENGTH,
    SITE_NAME_MAX_LENGTH,
    String::from("site_name_required"),
    String::from("site_name_length_overflow"),
  )
}

/// Checks the site description length, the limit as defined in the DB.
pub fn site_description_length_check(description: &str) -> LemmyResult<()> {
  max_length_check(
    description,
    SITE_DESCRIPTION_MAX_LENGTH,
    String::from("site_description_length_overflow"),
  )
}

fn max_length_check(item: &str, max_length: usize, msg: String) -> LemmyResult<()> {
  if item.len() > max_length {
    Err(LemmyError::from_message(&msg))
  } else {
    Ok(())
  }
}

fn min_max_length_check(
  item: &str,
  min_length: usize,
  max_length: usize,
  min_msg: String,
  max_msg: String,
) -> LemmyResult<()> {
  if item.len() > max_length {
    Err(LemmyError::from_message(&max_msg))
  } else if item.len() < min_length {
    Err(LemmyError::from_message(&min_msg))
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
        .map_err(|e| LemmyError::from_error_message(e, "invalid_regex"))
        .and_then(|regex| {
          // NOTE: It is difficult to know, in the universe of user-crafted regex, which ones
          // may match against any string text. To keep it simple, we'll match the regex
          // against an innocuous string - a single number - which should help catch a regex
          // that accidentally matches against all strings.
          if regex.is_match("1") {
            return Err(LemmyError::from_message("permissive_regex"));
          }

          Ok(Some(regex))
        })
    },
  )
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

pub fn check_totp_2fa_valid(
  totp_secret: &Option<String>,
  totp_token: &Option<String>,
  site_name: &str,
  username: &str,
) -> LemmyResult<()> {
  // Check only if they have a totp_secret in the DB
  if let Some(totp_secret) = totp_secret {
    // Throw an error if their token is missing
    let token = totp_token
      .as_deref()
      .ok_or_else(|| LemmyError::from_message("missing_totp_token"))?;

    let totp = build_totp_2fa(site_name, username, totp_secret)?;

    let check_passed = totp.check_current(token)?;
    if !check_passed {
      return Err(LemmyError::from_message("incorrect_totp token"));
    }
  }

  Ok(())
}

pub fn generate_totp_2fa_secret() -> String {
  Secret::generate_secret().to_string()
}

pub fn build_totp_2fa(site_name: &str, username: &str, secret: &str) -> Result<TOTP, LemmyError> {
  let sec = Secret::Raw(secret.as_bytes().to_vec());
  let sec_bytes = sec
    .to_bytes()
    .map_err(|_| LemmyError::from_message("Couldnt parse totp secret"))?;

  TOTP::new(
    totp_rs::Algorithm::SHA256,
    6,
    1,
    30,
    sec_bytes,
    Some(site_name.to_string()),
    username.to_string(),
  )
  .map_err(|e| LemmyError::from_error_message(e, "Couldnt generate TOTP"))
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
    return Err(LemmyError::from_message(
      "cant_enable_private_instance_and_federation_together",
    ));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::build_totp_2fa;
  use crate::utils::validation::{
    build_and_check_regex,
    check_site_visibility_valid,
    clean_url_params,
    generate_totp_2fa_secret,
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
    assert!(is_valid_post_title("hi").is_err());
    assert!(is_valid_post_title("him").is_ok());
    assert!(is_valid_post_title("n\n\n\n\nanother").is_err());
    assert!(is_valid_post_title("hello there!\n this is a test.").is_err());
    assert!(is_valid_post_title("hello there! this is a test.").is_ok());
  }

  #[test]
  fn test_valid_actor_name() {
    let actor_name_max_length = 20;
    assert!(is_valid_actor_name("Hello_98", actor_name_max_length).is_ok());
    assert!(is_valid_actor_name("ten", actor_name_max_length).is_ok());
    assert!(is_valid_actor_name("Hello-98", actor_name_max_length).is_err());
    assert!(is_valid_actor_name("a", actor_name_max_length).is_err());
    assert!(is_valid_actor_name("", actor_name_max_length).is_err());
  }

  #[test]
  fn test_valid_display_name() {
    let actor_name_max_length = 20;
    assert!(is_valid_display_name("hello @there", actor_name_max_length).is_ok());
    assert!(is_valid_display_name("@hello there", actor_name_max_length).is_err());

    // Make sure zero-space with an @ doesn't work
    assert!(
      is_valid_display_name(&format!("{}@my name is", '\u{200b}'), actor_name_max_length).is_err()
    );
  }

  #[test]
  fn test_valid_post_title() {
    assert!(is_valid_post_title("Post Title").is_ok());
    assert!(is_valid_post_title("   POST TITLE 😃😃😃😃😃").is_ok());
    assert!(is_valid_post_title("\n \n \n \n    		").is_err()); // tabs/spaces/newlines
  }

  #[test]
  fn test_valid_matrix_id() {
    assert!(is_valid_matrix_id("@dess:matrix.org").is_ok());
    assert!(is_valid_matrix_id("dess:matrix.org").is_err());
    assert!(is_valid_matrix_id(" @dess:matrix.org").is_err());
    assert!(is_valid_matrix_id("@dess:matrix.org t").is_err());
  }

  #[test]
  fn test_build_totp() {
    let generated_secret = generate_totp_2fa_secret();
    let totp = build_totp_2fa("lemmy", "my_name", &generated_secret);
    assert!(totp.is_ok());
  }

  #[test]
  fn test_valid_site_name() {
    let valid_names = [
      (0..SITE_NAME_MAX_LENGTH).map(|_| 'A').collect::<String>(),
      String::from("A"),
    ];
    let invalid_names = [
      (
        &(0..SITE_NAME_MAX_LENGTH + 1)
          .map(|_| 'A')
          .collect::<String>(),
        "site_name_length_overflow",
      ),
      (&String::new(), "site_name_required"),
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
      .for_each(|&(invalid_name, expected_err)| {
        let result = site_name_length_check(invalid_name);

        assert!(result.is_err());
        assert!(
          result
            .unwrap_err()
            .message
            .eq(&Some(String::from(expected_err))),
          "Testing {}, expected error {}",
          invalid_name,
          expected_err
        );
      });
  }

  #[test]
  fn test_valid_bio() {
    assert!(is_valid_bio_field(&(0..BIO_MAX_LENGTH).map(|_| 'A').collect::<String>()).is_ok());

    let invalid_result =
      is_valid_bio_field(&(0..BIO_MAX_LENGTH + 1).map(|_| 'A').collect::<String>());

    assert!(
      invalid_result.is_err()
        && invalid_result
          .unwrap_err()
          .message
          .eq(&Some(String::from("bio_length_overflow")))
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
        && invalid_result
          .unwrap_err()
          .message
          .eq(&Some(String::from("site_description_length_overflow")))
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
      (&Some("["), "invalid_regex"),
      (&Some("(foo|bar|)"), "permissive_regex"),
      (&Some(".*"), "permissive_regex"),
    ];

    match_everything_regexes
      .iter()
      .for_each(|&(regex_str, expected_err)| {
        let result = build_and_check_regex(regex_str);

        assert!(result.is_err());
        assert!(
          result
            .unwrap_err()
            .message
            .eq(&Some(String::from(expected_err))),
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
}
