// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::newtypes::{CommunityId, DbUrl, PersonId, PostId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::{post, post_like, post_read, post_saved};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable))]
#[cfg_attr(feature = "full", table_name = "post")]
pub struct Post {
  pub id: PostId,
  pub name: String,
  pub url: Option<DbUrl>,
  pub body: Option<String>,
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  pub removed: bool,
  pub locked: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub nsfw: bool,
  pub stickied: bool,
  pub embed_title: Option<String>,
  pub embed_description: Option<String>,
  pub embed_html: Option<String>,
  pub thumbnail_url: Option<DbUrl>,
  pub ap_id: DbUrl,
  pub local: bool,
}

#[derive(Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "post")]
pub struct PostForm {
  pub name: String,
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  pub nsfw: Option<bool>,
  pub url: Option<DbUrl>,
  pub body: Option<String>,
  pub removed: Option<bool>,
  pub locked: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub stickied: Option<bool>,
  pub embed_title: Option<String>,
  pub embed_description: Option<String>,
  pub embed_html: Option<String>,
  pub thumbnail_url: Option<DbUrl>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
}

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", belongs_to(Post))]
#[cfg_attr(feature = "full", table_name = "post_like")]
pub struct PostLike {
  pub id: i32,
  pub post_id: PostId,
  pub person_id: PersonId,
  pub score: i16,
  pub published: chrono::NaiveDateTime,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "post_like")]
pub struct PostLikeForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  pub score: i16,
}

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", belongs_to(Post))]
#[cfg_attr(feature = "full", table_name = "post_saved")]
pub struct PostSaved {
  pub id: i32,
  pub post_id: PostId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "post_saved")]
pub struct PostSavedForm {
  pub post_id: PostId,
  pub person_id: PersonId,
}

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", belongs_to(Post))]
#[cfg_attr(feature = "full", table_name = "post_read")]
pub struct PostRead {
  pub id: i32,
  pub post_id: PostId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "post_read")]
pub struct PostReadForm {
  pub post_id: PostId,
  pub person_id: PersonId,
}
