<<<<<<< HEAD
// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::newtypes::{CommentId, DbUrl, PersonId, PostId};
=======
use crate::newtypes::{CommentId, DbUrl, LtreeDef, PersonId, PostId};
use diesel_ltree::Ltree;
>>>>>>> 2f9d8776acdf326fdb6c57c8dec9ee4f75386017
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::{comment, comment_like, comment_saved};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", belongs_to(crate::source::post::Post))]
#[cfg_attr(feature = "full", table_name = "comment")]
pub struct Comment {
  pub id: CommentId,
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub content: String,
  pub removed: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub ap_id: DbUrl,
  pub local: bool,
  #[serde(with = "LtreeDef")]
  pub path: Ltree,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "comment")]
pub struct CommentForm {
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub content: String,
  pub removed: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
}

#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", belongs_to(Comment))]
#[cfg_attr(feature = "full", table_name = "comment_like")]
pub struct CommentLike {
  pub id: i32,
  pub person_id: PersonId,
  pub comment_id: CommentId,
  pub post_id: PostId, // TODO this is redundant
  pub score: i16,
  pub published: chrono::NaiveDateTime,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "comment_like")]
pub struct CommentLikeForm {
  pub person_id: PersonId,
  pub comment_id: CommentId,
  pub post_id: PostId, // TODO this is redundant
  pub score: i16,
}

#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", belongs_to(Comment))]
#[cfg_attr(feature = "full", table_name = "comment_saved")]
pub struct CommentSaved {
  pub id: i32,
  pub comment_id: CommentId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "comment_saved")]
pub struct CommentSavedForm {
  pub comment_id: CommentId,
  pub person_id: PersonId,
}
