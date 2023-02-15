use crate::error::LemmyError;
use regex::{Regex, RegexBuilder};

pub fn remove_slurs(test: &str, slur_regex: &Option<Regex>) -> String {
  if let Some(slur_regex) = slur_regex {
    slur_regex.replace_all(test, "*removed*").to_string()
  } else {
    test.to_string()
  }
}

pub(crate) fn slur_check<'a>(
  test: &'a str,
  slur_regex: &'a Option<Regex>,
) -> Result<(), Vec<&'a str>> {
  if let Some(slur_regex) = slur_regex {
    let mut matches: Vec<&str> = slur_regex.find_iter(test).map(|mat| mat.as_str()).collect();

    // Unique
    matches.sort_unstable();
    matches.dedup();

    if matches.is_empty() {
      Ok(())
    } else {
      Err(matches)
    }
  } else {
    Ok(())
  }
}

pub fn build_slur_regex(regex_str: Option<&str>) -> Option<Regex> {
  regex_str.map(|slurs| {
    RegexBuilder::new(slurs)
      .case_insensitive(true)
      .build()
      .expect("compile regex")
  })
}

pub fn check_slurs(text: &str, slur_regex: &Option<Regex>) -> Result<(), LemmyError> {
  if let Err(slurs) = slur_check(text, slur_regex) {
    Err(LemmyError::from_error_message(
      anyhow::anyhow!("{}", slurs_vec_to_str(&slurs)),
      "slurs",
    ))
  } else {
    Ok(())
  }
}

pub fn check_slurs_opt(
  text: &Option<String>,
  slur_regex: &Option<Regex>,
) -> Result<(), LemmyError> {
  match text {
    Some(t) => check_slurs(t, slur_regex),
    None => Ok(()),
  }
}

pub(crate) fn slurs_vec_to_str(slurs: &[&str]) -> String {
  let start = "No slurs - ";
  let combined = &slurs.join(", ");
  [start, combined].concat()
}
