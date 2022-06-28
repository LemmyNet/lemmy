use crate::{
  settings::SETTINGS,
  utils::{
    is_valid_actor_name,
    is_valid_display_name,
    is_valid_matrix_id,
    is_valid_post_title,
    remove_slurs,
    scrape_text_for_mentions,
    slur_check,
    slurs_vec_to_str,
  },
};

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
  let actor_name_max_length = SETTINGS.actor_name_max_length;
  assert!(is_valid_actor_name("Hello_98", actor_name_max_length));
  assert!(is_valid_actor_name("ten", actor_name_max_length));
  assert!(!is_valid_actor_name("Hello-98", actor_name_max_length));
  assert!(!is_valid_actor_name("a", actor_name_max_length));
  assert!(!is_valid_actor_name("", actor_name_max_length));
}

#[test]
fn test_valid_display_name() {
  let actor_name_max_length = SETTINGS.actor_name_max_length;
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

#[test]
fn test_slur_filter() {
  let slur_regex = SETTINGS.slur_regex();
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
    assert_eq!(&slurs_vec_to_str(slur_vec), has_slurs_err_str);
  }
}

// These helped with testing
// #[test]
// fn test_send_email() {
//  let result =  send_email("not a subject", "test_email@gmail.com", "ur user", "<h1>HI there</h1>");
//   assert!(result.is_ok());
// }
