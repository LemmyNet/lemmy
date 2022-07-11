// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::newtypes::{CommentId, CommunityId, PersonId, PostId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::{
  comment_aggregates,
  community_aggregates,
  person_aggregates,
  post_aggregates,
  site_aggregates,
};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", table_name = "comment_aggregates")]
pub struct CommentAggregates {
  pub id: i32,
  pub comment_id: CommentId,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub published: chrono::NaiveDateTime,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", table_name = "community_aggregates")]
pub struct CommunityAggregates {
  pub id: i32,
  pub community_id: CommunityId,
  pub subscribers: i64,
  pub posts: i64,
  pub comments: i64,
  pub published: chrono::NaiveDateTime,
  pub users_active_day: i64,
  pub users_active_week: i64,
  pub users_active_month: i64,
  pub users_active_half_year: i64,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", table_name = "person_aggregates")]
pub struct PersonAggregates {
  pub id: i32,
  pub person_id: PersonId,
  pub post_count: i64,
  pub post_score: i64,
  pub comment_count: i64,
  pub comment_score: i64,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", table_name = "post_aggregates")]
pub struct PostAggregates {
  pub id: i32,
  pub post_id: PostId,
  pub comments: i64,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub stickied: bool,
  pub published: chrono::NaiveDateTime,
  pub newest_comment_time_necro: chrono::NaiveDateTime, // A newest comment time, limited to 2 days, to prevent necrobumping
  pub newest_comment_time: chrono::NaiveDateTime,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", table_name = "site_aggregates")]
pub struct SiteAggregates {
  pub id: i32,
  pub site_id: i32,
  pub users: i64,
  pub posts: i64,
  pub comments: i64,
  pub communities: i64,
  pub users_active_day: i64,
  pub users_active_week: i64,
  pub users_active_month: i64,
  pub users_active_half_year: i64,
}
