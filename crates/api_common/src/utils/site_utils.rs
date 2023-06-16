/// Helper functions for manipulating parts of a site.
use lemmy_utils::error::LemmyError;
use regex::{Regex, RegexBuilder};

const SITE_NAME_MAX_LENGTH: usize = 20;
const SITE_DESCRIPTION_MAX_LENGTH: usize = 150;

/// Attempts to build a regex and check it for common errors before inserting into the DB.
pub fn build_and_check_regex(regex_str_opt: &Option<&str>) -> Result<Option<Regex>, LemmyError> {
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

/// Checks the site name length, the limit as defined in the DB.
pub fn site_name_length_check(name: &str) -> Result<(), LemmyError> {
  length_check(
    name,
    SITE_NAME_MAX_LENGTH,
    String::from("site_name_length_overflow"),
  )
}

/// Checks the site description length, the limit as defined in the DB.
pub fn site_description_length_check(description: &str) -> Result<(), LemmyError> {
  length_check(
    description,
    SITE_DESCRIPTION_MAX_LENGTH,
    String::from("site_description_length_overflow"),
  )
}

fn length_check(item: &str, max_length: usize, msg: String) -> Result<(), LemmyError> {
  if item.len() > max_length {
    Err(LemmyError::from_message(&msg))
  } else {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::utils::site_utils::{
    build_and_check_regex,
    site_description_length_check,
    site_name_length_check,
  };

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

        assert!(
          result
            .err()
            .is_some_and(|error| error.message.is_some_and(|msg| msg == expected_err)),
          "Testing regex: {:?}",
          regex_str
        );
      });
  }

  #[test]
  fn test_test_valid_site_name() {
    let result = site_name_length_check("awesome.comm");

    assert!(result.is_ok())
  }

  #[test]
  fn test_test_invalid_site_name() {
    let result = site_name_length_check("too long community name");

    assert!(result.err().is_some_and(|error| error
      .message
      .is_some_and(|msg| msg == "site_name_length_overflow")));
  }

  #[test]
  fn test_test_valid_site_description() {
    let result = site_description_length_check("cool cats");

    assert!(result.is_ok())
  }

  #[test]
  fn test_test_invalid_site_description() {
    let result = site_description_length_check(&(0..151).map(|_| 'A').collect::<String>());

    assert!(result.err().is_some_and(|error| error
      .message
      .is_some_and(|msg| msg == "site_description_length_overflow")));
  }
}
