use lemmy_db_schema::source::{
  comment::Comment,
  community::Community,
  instance::Instance,
  mod_log::{
    admin::{
      AdminAdd,
      AdminAllowInstance,
      AdminBan,
      AdminBlockInstance,
      AdminPurgeComment,
      AdminPurgeCommunity,
      AdminPurgePerson,
      AdminPurgePost,
      AdminRemoveCommunity,
    },
    moderator::{
      ModAddToCommunity,
      ModBanFromCommunity,
      ModChangeCommunityVisibility,
      ModFeaturePost,
      ModLockComment,
      ModLockPost,
      ModRemoveComment,
      ModRemovePost,
      ModTransferCommunity,
    },
  },
  person::Person,
  post::Post,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use {
  diesel::{dsl::Nullable, NullableExpressionMethods, Queryable, Selectable},
  lemmy_db_schema::{utils::queries::selects::person1_select, Person1AliasAllColumnsTuple},
};

pub mod api;
#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is added as a community moderator.
pub struct ModAddToCommunityView {
  pub mod_add_to_community: ModAddToCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is added as a site moderator.
pub struct AdminAddView {
  pub admin_add: AdminAdd,
  pub moderator: Option<Person>,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is banned from a community.
pub struct ModBanFromCommunityView {
  pub mod_ban_from_community: ModBanFromCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When someone is banned from the site.
pub struct AdminBanView {
  pub admin_ban: AdminBan,
  pub moderator: Option<Person>,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When the visibility of a community is changed
pub struct ModChangeCommunityVisibilityView {
  pub mod_change_community_visibility: ModChangeCommunityVisibility,
  pub moderator: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator locks a post (prevents new comments being made).
pub struct ModLockPostView {
  pub mod_lock_post: ModLockPost,
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator locks a comment (prevents replies to it or its children).
pub struct ModLockCommentView {
  pub mod_lock_comment: ModLockComment,
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub comment: Comment,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a comment.
pub struct ModRemoveCommentView {
  pub mod_remove_comment: ModRemoveComment,
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub comment: Comment,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin removes a community.
pub struct AdminRemoveCommunityView {
  pub admin_remove_community: AdminRemoveCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator removes a post.
pub struct ModRemovePostView {
  pub mod_remove_post: ModRemovePost,
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator features a post on a community (pins it to the top).
pub struct ModFeaturePostView {
  pub mod_feature_post: ModFeaturePost,
  pub moderator: Option<Person>,
  pub other_person: Person,
  pub post: Post,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When a moderator transfers a community to a new owner.
pub struct ModTransferCommunityView {
  pub mod_transfer_community: ModTransferCommunity,
  pub moderator: Option<Person>,
  pub community: Community,
  pub other_person: Person,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a comment.
pub struct AdminPurgeCommentView {
  pub admin_purge_comment: AdminPurgeComment,
  pub admin: Option<Person>,
  pub post: Post,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a community.
pub struct AdminPurgeCommunityView {
  pub admin_purge_community: AdminPurgeCommunity,
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a person.
pub struct AdminPurgePersonView {
  pub admin_purge_person: AdminPurgePerson,
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a post.
pub struct AdminPurgePostView {
  pub admin_purge_post: AdminPurgePost,
  pub admin: Option<Person>,
  pub community: Community,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a post.
pub struct AdminBlockInstanceView {
  pub admin_block_instance: AdminBlockInstance,
  pub instance: Instance,
  pub admin: Option<Person>,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// When an admin purges a post.
pub struct AdminAllowInstanceView {
  pub admin_allow_instance: AdminAllowInstance,
  pub instance: Instance,
  pub admin: Option<Person>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined modlog view
pub(crate) struct ModlogCombinedViewInternal {
  // Specific
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_allow_instance: Option<AdminAllowInstance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_block_instance: Option<AdminBlockInstance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_comment: Option<AdminPurgeComment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_community: Option<AdminPurgeCommunity>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_person: Option<AdminPurgePerson>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_purge_post: Option<AdminPurgePost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_add: Option<AdminAdd>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_add_to_community: Option<ModAddToCommunity>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_ban: Option<AdminBan>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_ban_from_community: Option<ModBanFromCommunity>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_feature_post: Option<ModFeaturePost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_change_community_visibility: Option<ModChangeCommunityVisibility>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_lock_post: Option<ModLockPost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_remove_comment: Option<ModRemoveComment>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub admin_remove_community: Option<AdminRemoveCommunity>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_remove_post: Option<ModRemovePost>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_transfer_community: Option<ModTransferCommunity>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub mod_lock_comment: Option<ModLockComment>,
  // Specific fields

  // Shared
  #[cfg_attr(feature = "full", diesel(embed))]
  pub moderator: Option<Person>,
  #[cfg_attr(feature = "full",
    diesel(
      select_expression_type = Nullable<Person1AliasAllColumnsTuple>,
      select_expression = person1_select().nullable()
    )
  )]
  pub other_person: Option<Person>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub instance: Option<Instance>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub community: Option<Community>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub post: Option<Post>,
  #[cfg_attr(feature = "full", diesel(embed))]
  pub comment: Option<Comment>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
// Use serde's internal tagging, to work easier with javascript libraries
#[serde(tag = "type_")]
pub enum ModlogCombinedView {
  AdminAllowInstance(AdminAllowInstanceView),
  AdminBlockInstance(AdminBlockInstanceView),
  AdminPurgeComment(AdminPurgeCommentView),
  AdminPurgeCommunity(AdminPurgeCommunityView),
  AdminPurgePerson(AdminPurgePersonView),
  AdminPurgePost(AdminPurgePostView),
  AdminAdd(AdminAddView),
  ModAddToCommunity(ModAddToCommunityView),
  AdminBan(AdminBanView),
  ModBanFromCommunity(ModBanFromCommunityView),
  ModFeaturePost(ModFeaturePostView),
  ModChangeCommunityVisibility(ModChangeCommunityVisibilityView),
  ModLockPost(ModLockPostView),
  ModRemoveComment(ModRemoveCommentView),
  AdminRemoveCommunity(AdminRemoveCommunityView),
  ModRemovePost(ModRemovePostView),
  ModTransferCommunity(ModTransferCommunityView),
  ModLockComment(ModLockCommentView),
}
