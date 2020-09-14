use crate::utils::{
  is_valid_community_name,
  is_valid_post_title,
  is_valid_preferred_username,
  is_valid_username,
  remove_slurs,
  scrape_text_for_mentions,
  slur_check,
  slurs_vec_to_str,
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
fn test_valid_register_username() {
  assert!(is_valid_username("Hello_98"));
  assert!(is_valid_username("ten"));
  assert!(!is_valid_username("Hello-98"));
  assert!(!is_valid_username("a"));
  assert!(!is_valid_username(""));
}

#[test]
fn test_valid_preferred_username() {
  assert!(is_valid_preferred_username("hello @there"));
  assert!(!is_valid_preferred_username("@hello there"));
}

#[test]
fn test_valid_community_name() {
  assert!(is_valid_community_name("example"));
  assert!(is_valid_community_name("example_community"));
  assert!(!is_valid_community_name("Example"));
  assert!(!is_valid_community_name("Ex"));
  assert!(!is_valid_community_name(""));
}

#[test]
fn test_valid_post_title() {
  assert!(is_valid_post_title("Post Title"));
  assert!(is_valid_post_title("   POST TITLE 😃😃😃😃😃"));
  assert!(!is_valid_post_title("\n \n \n \n    		")); // tabs/spaces/newlines
}

#[test]
fn test_slur_filter() {
  let test =
      "coons test dindu ladyboy tranny retardeds. Capitalized Niggerz. This is a bunch of other safe text.";
  let slur_free = "No slurs here";
  assert_eq!(
      remove_slurs(&test),
      "*removed* test *removed* *removed* *removed* *removed*. Capitalized *removed*. This is a bunch of other safe text."
        .to_string()
    );

  let has_slurs_vec = vec![
    "Niggerz",
    "coons",
    "dindu",
    "ladyboy",
    "retardeds",
    "tranny",
  ];
  let has_slurs_err_str = "No slurs - Niggerz, coons, dindu, ladyboy, retardeds, tranny";

  assert_eq!(slur_check(test), Err(has_slurs_vec));
  assert_eq!(slur_check(slur_free), Ok(()));
  if let Err(slur_vec) = slur_check(test) {
    assert_eq!(&slurs_vec_to_str(slur_vec), has_slurs_err_str);
  }
}

// These helped with testing
// #[test]
// fn test_send_email() {
//  let result =  send_email("not a subject", "test_email@gmail.com", "ur user", "<h1>HI there</h1>");
//   assert!(result.is_ok());
// }
