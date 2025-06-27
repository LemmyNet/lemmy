use lemmy_db_schema::{
  newtypes::{CommunityId, PaginationCursor, PersonId},
  source::{
    combined::search::SearchCombined,
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions},
    images::ImageDetails,
    instance::InstanceActions,
    multi_community::MultiCommunity,
    person::{Person, PersonActions},
    post::{Post, PostActions},
    tag::TagsView,
  },
  SearchSortType,
  SearchType,
};
use lemmy_db_schema_file::enums::ListingType;
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_community::{CommunityView, MultiCommunityView};
use lemmy_db_views_person::PersonView;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::utils::queries::{
    community_post_tags_fragment,
    creator_banned,
    creator_is_admin,
    local_user_can_mod,
    post_tags_fragment,
  },
  lemmy_db_schema::utils::queries::{creator_banned_from_community, creator_is_moderator},
  lemmy_db_views_local_user::LocalUserView,
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
  #[cfg_attr(feature = "full", diesel(embed))]
  pub multi_community: Option<MultiCommunity>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community_actions: Option<CommunityActions>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance_actions: Option<InstanceActions>,
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
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_is_moderator()
    )
  )]
  pub creator_is_moderator: bool,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression = creator_banned_from_community()
    )
  )]
  pub creator_banned_from_community: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum SearchCombinedView {
  Post(PostView),
  Comment(CommentView),
  Community(CommunityView),
  Person(PersonView),
  MultiCommunity(MultiCommunityView),
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Searches the site, given a search term, and some optional filters.
pub struct Search {
  pub q: String,
  pub community_id: Option<CommunityId>,
  pub community_name: Option<String>,
  pub creator_id: Option<PersonId>,
  pub type_: Option<SearchType>,
  pub sort: Option<SearchSortType>,
  /// Filter to within a given time range, in seconds.
  /// IE 60 would give results for the past minute.
  pub time_range_seconds: Option<i32>,
  pub listing_type: Option<ListingType>,
  pub title_only: Option<bool>,
  pub post_url_only: Option<bool>,
  pub liked_only: Option<bool>,
  pub disliked_only: Option<bool>,
  /// If true, then show the nsfw posts (even if your user setting is to hide them)
  pub show_nsfw: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The search response, containing lists of the return type possibilities
pub struct SearchResponse {
  pub results: Vec<SearchCombinedView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
