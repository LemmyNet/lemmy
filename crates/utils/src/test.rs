use crate::utils::{
  is_valid_actor_name,
  is_valid_display_name,
  is_valid_matrix_id,
  is_valid_post_title,
  remove_slurs,
  scrape_text_for_mentions,
  slur_check,
  slurs_vec_to_str,
};
use regex::RegexBuilder;

#[test]
fn test_mentions_regex() {
  let text = "Just read a great blog post by [@tedu@honk.teduangst.com](/u/test). And another by !test_community@fish.teduangst.com . Another [@lemmy@lemmy-alpha:8540](/u/fish)";
  let mentions = scrape_text_for_mentions(text);

  assert_eq!(mentions[0].name, "tedu".to_string());
  assert_eq!(mentions[0].domain, "honk.teduangst.com".to_string());
  assert_eq!(mentions[1].domain, "lemmy-alpha:8540".to_string());
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
