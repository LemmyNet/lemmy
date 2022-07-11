// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

pub mod comment;
pub mod post;
pub mod private_message;

#[cfg(test)]
mod tests {
  use crate::protocol::{
    activities::create_or_update::{
      comment::CreateOrUpdateComment,
      post::CreateOrUpdatePost,
      private_message::CreateOrUpdatePrivateMessage,
    },
    tests::test_parse_lemmy_item,
  };

  #[test]
  fn test_parse_lemmy_create_or_update() {
    test_parse_lemmy_item::<CreateOrUpdatePost>(
      "assets/lemmy/activities/create_or_update/create_page.json",
    )
    .unwrap();
    test_parse_lemmy_item::<CreateOrUpdatePost>(
      "assets/lemmy/activities/create_or_update/update_page.json",
    )
    .unwrap();
    test_parse_lemmy_item::<CreateOrUpdateComment>(
      "assets/lemmy/activities/create_or_update/create_note.json",
    )
    .unwrap();
    test_parse_lemmy_item::<CreateOrUpdatePrivateMessage>(
      "assets/lemmy/activities/create_or_update/create_private_message.json",
    )
    .unwrap();
  }
}
