use crate::newtypes::{PersonId, PostId, SiteId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use {
  crate::schema::{post_actions, site_aggregates},
  diesel::{dsl, expression_methods::NullableExpressionMethods},
  ts_rs::TS,
};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
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
  #[cfg_attr(feature = "full", diesel(select_expression = post_actions::read_comments_amount.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<post_actions::read_comments_amount>))]
  pub read_comments: i64,
  #[cfg_attr(feature = "full", diesel(select_expression = post_actions::read_comments.assume_not_null()))]
  #[cfg_attr(feature = "full", diesel(select_expression_type = dsl::AssumeNotNull<post_actions::read_comments>))]
  pub published: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_actions))]
pub struct PersonPostAggregatesForm {
  pub person_id: PersonId,
  pub post_id: PostId,
  #[cfg_attr(feature = "full", diesel(column_name = read_comments_amount))]
  pub read_comments: i64,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy, Hash)]
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
