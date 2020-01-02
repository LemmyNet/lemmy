pub mod community;
pub mod post;
pub mod user;
use crate::Settings;

use std::fmt::Display;

#[cfg(test)]
mod tests {
  use crate::db::community::Community;
  use crate::db::post::Post;
  use crate::db::user::User_;
  use crate::db::{ListingType, SortType};
  use crate::{naive_now, Settings};

  #[test]
  fn test_person() {
    let user = User_ {
      id: 52,
      name: "thom".into(),
      fedi_name: "rrf".into(),
      preferred_username: None,
      password_encrypted: "here".into(),
      email: None,
      avatar: None,
      published: naive_now(),
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
    };

    let person = user.as_person();
    assert_eq!(
      format!("https://{}/federation/u/thom", Settings::get().hostname),
      person.object_props.id_string().unwrap()
    );
  }

  #[test]
  fn test_community() {
    let community = Community {
      id: 42,
      name: "Test".into(),
      title: "Test Title".into(),
      description: Some("Test community".into()),
      category_id: 32,
      creator_id: 52,
      removed: false,
      published: naive_now(),
      updated: Some(naive_now()),
      deleted: false,
      nsfw: false,
    };

    let group = community.as_group();
    assert_eq!(
      format!("https://{}/federation/c/Test", Settings::get().hostname),
      group.object_props.id_string().unwrap()
    );
  }

  #[test]
  fn test_post() {
    let post = Post {
      id: 62,
      name: "A test post".into(),
      url: None,
      body: None,
      creator_id: 52,
      community_id: 42,
      published: naive_now(),
      removed: false,
      locked: false,
      stickied: false,
      nsfw: false,
      deleted: false,
      updated: None,
    };

    let page = post.as_page();
    assert_eq!(
      format!("https://{}/federation/post/62", Settings::get().hostname),
      page.object_props.id_string().unwrap()
    );
  }
}

pub fn make_apub_endpoint<S: Display, T: Display>(point: S, value: T) -> String {
  format!(
    "https://{}/federation/{}/{}",
    Settings::get().hostname,
    point,
    value
  )
}
