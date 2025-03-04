use crate::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use regex::Regex;

pub fn remove_slurs(test: &str, slur_regex: &Regex) -> String {
  slur_regex.replace_all(test, "*removed*").to_string()
}

pub(crate) fn slur_check<'a>(test: &'a str, slur_regex: &'a Regex) -> Result<(), Vec<&'a str>> {
  let mut matches: Vec<&str> = slur_regex.find_iter(test).map(|mat| mat.as_str()).collect();

  // Unique
  matches.sort_unstable();
  matches.dedup();

  if matches.is_empty() {
    Ok(())
  } else {
    Err(matches)
  }
}

pub fn check_slurs(text: &str, slur_regex: &Regex) -> LemmyResult<()> {
  if let Err(slurs) = slur_check(text, slur_regex) {
    Err(anyhow::anyhow!("{}", slurs_vec_to_str(&slurs))).with_lemmy_type(LemmyErrorType::Slurs)
  } else {
    Ok(())
  }
}

pub fn check_slurs_opt(text: &Option<String>, slur_regex: &Regex) -> LemmyResult<()> {
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

#[cfg(test)]
mod test {

  use crate::{
    error::LemmyResult,
    utils::slurs::{remove_slurs, slur_check, slurs_vec_to_str},
  };
  use pretty_assertions::assert_eq;
  use regex::RegexBuilder;

  #[test]
  fn test_slur_filter() -> LemmyResult<()> {
    let slur_regex = RegexBuilder::new(r"(fag(g|got|tard)?\b|cock\s?sucker(s|ing)?|ni((g{2,}|q)+|[gq]{2,})[e3r]+(s|z)?|mudslime?s?|kikes?|\bspi(c|k)s?\b|\bchinks?|gooks?|bitch(es|ing|y)?|whor(es?|ing)|\btr(a|@)nn?(y|ies?)|\b(b|re|r)tard(ed)?s?)").case_insensitive(true).build()?;
    let test =
      "faggot test kike tranny cocksucker retardeds. Capitalized Niggerz. This is a bunch of other safe text.";
    let slur_free = "No slurs here";
    assert_eq!(
      remove_slurs(test, &slur_regex),
      "*removed* test *removed* *removed* *removed* *removed*. Capitalized *removed*. This is a bunch of other safe text."
        .to_string()
    );

    let has_slurs_vec = vec![
      "Niggerz",
      "cocksucker",
      "faggot",
      "kike",
      "retardeds",
      "tranny",
    ];
    let has_slurs_err_str = "No slurs - Niggerz, cocksucker, faggot, kike, retardeds, tranny";

    assert_eq!(slur_check(test, &slur_regex), Err(has_slurs_vec));
    assert_eq!(slur_check(slur_free, &slur_regex), Ok(()));
    if let Err(slur_vec) = slur_check(test, &slur_regex) {
      assert_eq!(&slurs_vec_to_str(&slur_vec), has_slurs_err_str);
    }

    Ok(())
  }

  // These helped with testing
  // #[test]
  // fn test_send_email() {
  //  let result =  send_email("not a subject", "test_email@gmail.com", "ur user", "<h1>HI
  // there</h1>");   assert!(result.is_ok());
  // }
}
