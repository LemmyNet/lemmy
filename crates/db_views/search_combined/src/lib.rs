use lemmy_db_schema::{
  newtypes::{CommunityId, PaginationCursor, PersonId},
  source::{
    combined::search::SearchCombined,
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions},
    images::ImageDetails,
    instance::InstanceActions,
    person::{Person, PersonActions},
    post::{Post, PostActions},
    tag::TagsView,
  },
  SearchSortType,
  SearchType,
};
use lemmy_db_schema_file::enums::ListingType;
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{dsl::Nullable, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::{
    utils::queries::{
      community_post_tags_fragment,
      creator_banned,
      creator_community_actions_select,
      creator_home_instance_actions_select,
      creator_is_admin,
      creator_local_instance_actions_select,
      local_user_can_mod,
      post_tags_fragment,
    },
    CreatorCommunityActionsAllColumnsTuple,
    CreatorHomeInstanceActionsAllColumnsTuple,
    CreatorLocalInstanceActionsAllColumnsTuple,
  },
  lemmy_db_views_local_user::LocalUserView,
  ts_rs::TS,
};

#[cfg(feature = "full")]
pub mod impls;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined search view
pub(crate) struct SearchCombinedViewInternal {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub search_combined: SearchCombined,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub item_creator: Option<Person>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Option<Community>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<CreatorCommunityActionsAllColumnsTuple>,
      select_expression = creator_community_actions_select().nullable()
    )
  )]
  pub creator_community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorHomeInstanceActionsAllColumnsTuple>,
      select_expression = creator_home_instance_actions_select()))]
  pub creator_home_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(
      select_expression_type = Nullable<CreatorLocalInstanceActionsAllColumnsTuple>,
      select_expression = creator_local_instance_actions_select()))]
  pub creator_local_instance_actions: Option<InstanceActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post_actions: Option<PostActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub person_actions: Option<PersonActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment_actions: Option<CommentActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub image_details: Option<ImageDetails>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_admin()
    )
  )]
  pub item_creator_is_admin: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = post_tags_fragment()
    )
  )]
  /// tags of this post
  pub post_tags: TagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = community_post_tags_fragment()
    )
  )]
  /// available tags in this community
  pub community_post_tags: TagsView,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = local_user_can_mod()
    )
  )]
  pub can_mod: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned()
    )
  )]
  pub creator_banned: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum SearchCombinedView {
  Post(PostView),
  Comment(CommentView),
  Community(CommunityView),
  Person(PersonView),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Searches the site, given a search term, and some optional filters.
pub struct Search {
  pub q: String,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_name: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub creator_id: Option<PersonId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub type_: Option<SearchType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub sort: Option<SearchSortType>,
  #[cfg_attr(feature = "full", ts(optional))]
  /// Filter to within a given time range, in seconds.
  /// IE 60 would give results for the past minute.
  pub time_range_seconds: Option<i32>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub listing_type: Option<ListingType>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub title_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_url_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub liked_only: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub disliked_only: Option<bool>,
  /// If true, then show the nsfw posts (even if your user setting is to hide them)
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_nsfw: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The search response, containing lists of the return type possibilities
pub struct SearchResponse {
  pub results: Vec<SearchCombinedView>,
  /// the pagination cursor to use to fetch the next page
  #[cfg_attr(feature = "full", ts(optional))]
  pub next_page: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub prev_page: Option<PaginationCursor>,
}
