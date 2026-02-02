use chrono::{DateTime, Utc};
use lemmy_db_schema::{
  SearchSortType,
  SearchType,
  newtypes::CommunityId,
  source::{
    combined::search::SearchCombined,
    comment::{Comment, CommentActions},
    community::{Community, CommunityActions},
    community_tag::CommunityTagsView,
    images::ImageDetails,
    multi_community::MultiCommunity,
    person::{Person, PersonActions},
    post::{Post, PostActions},
  },
};
use lemmy_db_schema_file::{PersonId, enums::ListingType};
use lemmy_db_views_comment::CommentView;
use lemmy_db_views_community::{CommunityView, MultiCommunityView};
use lemmy_db_views_person::PersonView;
use lemmy_db_views_post::PostView;
use lemmy_diesel_utils::pagination::PaginationCursor;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{Queryable, Selectable},
  lemmy_db_schema::utils::queries::selects::{
    CreatorLocalHomeBanExpiresType,
    community_tags_fragment,
    creator_ban_expires_from_community,
    creator_banned_from_community,
    creator_is_admin,
    creator_is_moderator,
    creator_local_home_ban_expires,
    creator_local_home_banned,
    local_user_can_mod,
    post_community_tags_fragment,
  },
  lemmy_db_views_local_user::LocalUserView,
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[cfg(feature = "full")]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
/// A combined search view
pub(crate) struct SearchCombinedViewInternal {
  #[diesel(embed)]
  pub search_combined: SearchCombined,
  #[diesel(embed)]
  pub comment: Option<Comment>,
  #[diesel(embed)]
  pub post: Option<Post>,
  #[diesel(embed)]
  pub item_creator: Option<Person>,
  #[diesel(embed)]
  pub community: Option<Community>,
  #[diesel(embed)]
  pub multi_community: Option<MultiCommunity>,
  #[diesel(embed)]
  pub community_actions: Option<CommunityActions>,
  #[diesel(embed)]
  pub post_actions: Option<PostActions>,
  #[diesel(embed)]
  pub person_actions: Option<PersonActions>,
  #[diesel(embed)]
  pub comment_actions: Option<CommentActions>,
  #[diesel(embed)]
  pub image_details: Option<ImageDetails>,
  #[diesel(select_expression = creator_is_admin())]
  pub item_creator_is_admin: bool,
  #[diesel(select_expression = post_community_tags_fragment())]
  /// tags for this post
  pub tags: CommunityTagsView,
  #[diesel(select_expression = community_tags_fragment())]
  /// available tags in this community
  pub community_tags: CommunityTagsView,
  #[diesel(select_expression = local_user_can_mod())]
  pub can_mod: bool,
  #[diesel(select_expression = creator_local_home_banned())]
  pub creator_banned: bool,
  #[diesel(
    select_expression_type = CreatorLocalHomeBanExpiresType,
    select_expression = creator_local_home_ban_expires()
  )]
  pub creator_ban_expires_at: Option<DateTime<Utc>>,
  #[diesel(select_expression = creator_is_moderator())]
  pub creator_is_moderator: bool,
  #[diesel(select_expression = creator_banned_from_community())]
  pub creator_banned_from_community: bool,
  #[diesel(select_expression = creator_ban_expires_from_community())]
  pub creator_community_ban_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(tag = "type_", rename_all = "snake_case")]
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
  /// The search query. Can be a plain text, or an object ID which will be resolved
  /// (eg `https://lemmy.world/comment/1` or `!fediverse@lemmy.ml`).
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
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// The search response, containing lists of the return type possibilities
pub struct SearchResponse {
  /// If `Search.q` contains an ActivityPub ID (eg `https://lemmy.world/comment/1`) or an
  /// identifier (eg `!fediverse@lemmy.ml`) then this field contains the resolved object.
  /// It should always be shown above other search results.
  pub resolve: Option<SearchCombinedView>,
  /// Items which contain the search string in post body, comment text, community sidebar etc.
  /// This is always empty when calling `/api/v4/resolve_object`
  pub search: Vec<SearchCombinedView>,
  /// the pagination cursor to use to fetch the next page
  pub next_page: Option<PaginationCursor>,
  pub prev_page: Option<PaginationCursor>,
}
