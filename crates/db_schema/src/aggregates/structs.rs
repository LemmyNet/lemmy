use crate::newtypes::{CommentId, CommunityId, InstanceId, PersonId, PostId, SiteId};
#[cfg(feature = "full")]
use crate::schema::{
  comment_aggregates,
  community_aggregates,
  person_aggregates,
  person_post_aggregates,
  post_aggregates,
  site_aggregates,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(feature = "full", diesel(table_name = comment_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(primary_key(comment_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// Aggregate data for a comment.
pub struct CommentAggregates {
  pub comment_id: CommentId,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub published: DateTime<Utc>,
  /// The total number of children in this comment branch.
  pub child_count: i32,
  #[serde(skip)]
  pub hot_rank: f64,
  #[serde(skip)]
  pub controversy_rank: f64,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(feature = "full", diesel(table_name = community_aggregates))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(primary_key(community_id)))]
#[cfg_attr(feature = "full", ts(export))]
/// Aggregate data for a community.
pub struct CommunityAggregates {
  pub community_id: CommunityId,
  pub subscribers: i64,
  pub posts: i64,
  pub comments: i64,
  pub published: DateTime<Utc>,
  /// The number of users with any activity in the last day.
  pub users_active_day: i64,
  /// The number of users with any activity in the last week.
  pub users_active_week: i64,
  /// The number of users with any activity in the last month.
  pub users_active_month: i64,
  /// The number of users with any activity in the last year.
  pub users_active_half_year: i64,
  #[serde(skip)]
  pub hot_rank: f64,
  pub local_subscribers: i64,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(feature = "full", diesel(table_name = person_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// Aggregate data for a person.
pub struct PersonAggregates {
  pub person_id: PersonId,
  pub post_count: i64,
  #[serde(skip)]
  pub post_score: i64,
  pub comment_count: i64,
  #[serde(skip)]
  pub comment_score: i64,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(feature = "full", diesel(table_name = post_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(primary_key(post_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// Aggregate data for a post.
pub struct PostAggregates {
  pub post_id: PostId,
  pub comments: i64,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub published: DateTime<Utc>,
  #[serde(skip)]
  /// A newest comment time, limited to 2 days, to prevent necrobumping
  pub newest_comment_time_necro: DateTime<Utc>,
  /// The time of the newest comment in the post.
  #[serde(skip)]
  pub newest_comment_time: DateTime<Utc>,
  /// If the post is featured on the community.
  #[serde(skip)]
  pub featured_community: bool,
  /// If the post is featured on the site / to local.
  #[serde(skip)]
  pub featured_local: bool,
  #[serde(skip)]
  pub hot_rank: f64,
  #[serde(skip)]
  pub hot_rank_active: f64,
  #[serde(skip)]
  pub community_id: CommunityId,
  #[serde(skip)]
  pub creator_id: PersonId,
  #[serde(skip)]
  pub controversy_rank: f64,
  #[serde(skip)]
  pub instance_id: InstanceId,
  /// A rank that amplifies smaller communities
  #[serde(skip)]
  pub scaled_rank: f64,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(table_name = person_post_aggregates))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, post_id)))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// Aggregate data for a person's post.
pub struct PersonPostAggregates {
  pub person_id: PersonId,
  pub post_id: PostId,
  /// The number of comments they've read on that post.
  ///
  /// This is updated to the current post comment count every time they view a post.
  pub read_comments: i64,
  pub published: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_post_aggregates))]
pub struct PersonPostAggregatesForm {
  pub person_id: PersonId,
  pub post_id: PostId,
  pub read_comments: i64,
  pub published: Option<DateTime<Utc>>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(feature = "full", diesel(table_name = site_aggregates))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::site::Site)))]
#[cfg_attr(feature = "full", diesel(primary_key(site_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// Aggregate data for a site.
pub struct SiteAggregates {
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
