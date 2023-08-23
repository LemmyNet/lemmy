use crate::newtypes::{CommentId, CommunityId, PersonId, PostId, SiteId};
#[cfg(feature = "full")]
use crate::schema::{
  comment_aggregates,
  community_aggregates,
  person_aggregates,
  person_post_aggregates,
  post_aggregates,
  site_aggregates,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = comment_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", ts(export))]
/// Aggregate data for a comment.
pub struct CommentAggregates {
  pub id: i32,
  pub comment_id: CommentId,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub published: chrono::NaiveDateTime,
  /// The total number of children in this comment branch.
  pub child_count: i32,
  pub hot_rank: i32,
  pub controversy_rank: f64,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = community_aggregates))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", ts(export))]
/// Aggregate data for a community.
pub struct CommunityAggregates {
  pub id: i32,
  pub community_id: CommunityId,
  pub subscribers: i64,
  pub posts: i64,
  pub comments: i64,
  pub published: chrono::NaiveDateTime,
  /// The number of users with any activity in the last day.
  pub users_active_day: i64,
  /// The number of users with any activity in the last week.
  pub users_active_week: i64,
  /// The number of users with any activity in the last month.
  pub users_active_month: i64,
  /// The number of users with any activity in the last year.
  pub users_active_half_year: i64,
  pub hot_rank: i32,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = person_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", ts(export))]
/// Aggregate data for a person.
pub struct PersonAggregates {
  pub id: i32,
  pub person_id: PersonId,
  pub post_count: i64,
  pub post_score: i64,
  pub comment_count: i64,
  pub comment_score: i64,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = post_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", ts(export))]
/// Aggregate data for a post.
pub struct PostAggregates {
  pub id: i32,
  pub post_id: PostId,
  pub comments: i64,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub published: chrono::NaiveDateTime,
  /// A newest comment time, limited to 2 days, to prevent necrobumping  
  pub newest_comment_time_necro: chrono::NaiveDateTime,
  /// The time of the newest comment in the post.
  pub newest_comment_time: chrono::NaiveDateTime,
  /// If the post is featured on the community.
  pub featured_community: bool,
  /// If the post is featured on the site / to local.
  pub featured_local: bool,
  pub hot_rank: i32,
  pub hot_rank_active: i32,
  pub community_id: CommunityId,
  pub creator_id: PersonId,
  pub controversy_rank: f64,
  /// A rank that amplifies smaller communities
  pub scaled_rank: i32,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(table_name = person_post_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
/// Aggregate data for a person's post.
pub struct PersonPostAggregates {
  pub id: i32,
  pub person_id: PersonId,
  pub post_id: PostId,
  /// The number of comments they've read on that post.
  ///
  /// This is updated to the current post comment count every time they view a post.
  pub read_comments: i64,
  pub published: chrono::NaiveDateTime,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_post_aggregates))]
pub struct PersonPostAggregatesForm {
  pub person_id: PersonId,
  pub post_id: PostId,
  pub read_comments: i64,
  pub published: Option<chrono::NaiveDateTime>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = site_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::site::Site)))]
#[cfg_attr(feature = "full", ts(export))]
/// Aggregate data for a site.
pub struct SiteAggregates {
  pub id: i32,
  pub site_id: SiteId,
  pub users: i64,
  pub posts: i64,
  pub comments: i64,
  pub communities: i64,
  /// The number of users with any activity in the last day.
  pub users_active_day: i64,
  /// The number of users with any activity in the last week.
  pub users_active_week: i64,
  /// The number of users with any activity in the last month.
  pub users_active_month: i64,
  /// The number of users with any activity in the last half year.
  pub users_active_half_year: i64,
}
