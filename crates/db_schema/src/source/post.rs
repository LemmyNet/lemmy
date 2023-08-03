use crate::newtypes::{CommunityId, DbUrl, LanguageId, PersonId, PostId};
#[cfg(feature = "full")]
use crate::schema::{post, post_like, post_read, post_saved};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use typed_builder::TypedBuilder;

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(table_name = post))]
#[cfg_attr(feature = "full", ts(export))]
/// A post.
pub struct Post {
  pub id: PostId,
  pub name: String,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  /// An optional link / url for the post.
  pub url: Option<DbUrl>,
  /// An optional post body, in markdown.
  pub body: Option<String>,
  pub creator_id: PersonId,
  pub community_id: CommunityId,
  /// Whether the post is removed.
  pub removed: bool,
  /// Whether the post is locked.
  pub locked: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  /// Whether the post is deleted.
  pub deleted: bool,
  /// Whether the post is NSFW.
  pub nsfw: bool,
  /// A title for the link.
  pub embed_title: Option<String>,
  /// A description for the link.
  pub embed_description: Option<String>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  /// A thumbnail picture url.
  pub thumbnail_url: Option<DbUrl>,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  /// The federated activity id / ap_id.
  pub ap_id: DbUrl,
  /// Whether the post is local.
  pub local: bool,
  #[cfg_attr(feature = "full", ts(type = "string"))]
  /// A video url for the link.
  pub embed_video_url: Option<DbUrl>,
  pub language_id: LanguageId,
  /// Whether the post is featured to its community.
  pub featured_community: bool,
  /// Whether the post is featured to its site.
  pub featured_local: bool,
}

#[derive(Debug, Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post))]
pub struct PostInsertForm {
  #[builder(!default)]
  pub name: String,
  #[builder(!default)]
  pub creator_id: PersonId,
  #[builder(!default)]
  pub community_id: CommunityId,
  pub nsfw: Option<bool>,
  pub url: Option<DbUrl>,
  pub body: Option<String>,
  pub removed: Option<bool>,
  pub locked: Option<bool>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub published: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub embed_title: Option<String>,
  pub embed_description: Option<String>,
  pub embed_video_url: Option<DbUrl>,
  pub thumbnail_url: Option<DbUrl>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub language_id: Option<LanguageId>,
  pub featured_community: Option<bool>,
  pub featured_local: Option<bool>,
}

#[derive(Debug, Clone, TypedBuilder)]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post))]
pub struct PostUpdateForm {
  pub name: Option<String>,
  pub nsfw: Option<bool>,
  pub url: Option<Option<DbUrl>>,
  pub body: Option<Option<String>>,
  pub removed: Option<bool>,
  pub locked: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<Option<chrono::NaiveDateTime>>,
  pub deleted: Option<bool>,
  pub embed_title: Option<Option<String>>,
  pub embed_description: Option<Option<String>>,
  pub embed_video_url: Option<Option<DbUrl>>,
  pub thumbnail_url: Option<Option<DbUrl>>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
  pub language_id: Option<LanguageId>,
  pub featured_community: Option<bool>,
  pub featured_local: Option<bool>,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = post_like))]
pub struct PostLike {
  pub id: i32,
  pub post_id: PostId,
  pub person_id: PersonId,
  pub score: i16,
  pub published: chrono::NaiveDateTime,
}

#[derive(Clone)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_like))]
pub struct PostLikeForm {
  pub post_id: PostId,
  pub person_id: PersonId,
  pub score: i16,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = post_saved))]
pub struct PostSaved {
  pub id: i32,
  pub post_id: PostId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_saved))]
pub struct PostSavedForm {
  pub post_id: PostId,
  pub person_id: PersonId,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Associations))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = post_read))]
pub struct PostRead {
  pub id: i32,
  pub post_id: PostId,
  pub person_id: PersonId,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_read))]
pub struct PostReadForm {
  pub post_id: PostId,
  pub person_id: PersonId,
}
