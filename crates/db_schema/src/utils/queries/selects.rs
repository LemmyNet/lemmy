use crate::{Person1AliasAllColumnsTuple, Person2AliasAllColumnsTuple};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  PgExpressionMethods,
  QueryDsl,
  dsl::{case_when, exists, not},
  expression::SqlLiteral,
  helper_types::Nullable,
  query_source::AliasedField,
  sql_types::{Json, Timestamptz},
};
use lemmy_db_schema_file::{
  aliases::{
    CreatorCommunityInstanceActions,
    CreatorHomeInstanceActions,
    CreatorLocalInstanceActions,
    creator_community_actions,
    creator_community_instance_actions,
    creator_home_instance_actions,
    creator_local_instance_actions,
    creator_local_user,
    person1,
    person2,
  },
  schema::{
    comment,
    community,
    community_actions,
    community_tag,
    instance_actions,
    local_user,
    person,
    post,
    post_community_tag,
  },
};
use lemmy_diesel_utils::utils::functions::{coalesce_2_nullable, coalesce_3_nullable};

/// Checks that the creator_local_user is an admin.
#[diesel::dsl::auto_type]
pub fn creator_is_admin() -> _ {
  creator_local_user
    .field(local_user::admin)
    .nullable()
    .is_not_distinct_from(true)
}

/// Checks that the local_user is an admin.
#[diesel::dsl::auto_type]
pub fn local_user_is_admin() -> _ {
  local_user::admin.nullable().is_not_distinct_from(true)
}

/// Checks to see if the comment creator is an admin.
#[diesel::dsl::auto_type]
pub fn comment_creator_is_admin() -> _ {
  exists(
    creator_local_user.filter(
      comment::creator_id
        .eq(creator_local_user.field(local_user::person_id))
        .and(creator_local_user.field(local_user::admin).eq(true)),
    ),
  )
}

#[diesel::dsl::auto_type]
pub fn post_creator_is_admin() -> _ {
  exists(
    creator_local_user.filter(
      post::creator_id
        .eq(creator_local_user.field(local_user::person_id))
        .and(creator_local_user.field(local_user::admin).eq(true)),
    ),
  )
}

#[diesel::dsl::auto_type]
pub fn creator_is_moderator() -> _ {
  creator_community_actions
    .field(community_actions::became_moderator_at)
    .nullable()
    .is_not_null()
}

#[diesel::dsl::auto_type]
pub fn creator_banned_from_community() -> _ {
  creator_community_actions
    .field(community_actions::received_ban_at)
    .nullable()
    .is_not_null()
}

#[diesel::dsl::auto_type]
pub fn creator_ban_expires_from_community() -> _ {
  creator_community_actions
    .field(community_actions::ban_expires_at)
    .nullable()
}

#[diesel::dsl::auto_type]
/// Checks to see if a creator is banned from the local instance.
fn creator_local_banned() -> _ {
  creator_local_instance_actions
    .field(instance_actions::received_ban_at)
    .nullable()
    .is_not_null()
}

#[diesel::dsl::auto_type]
fn creator_local_ban_expires() -> _ {
  creator_local_instance_actions
    .field(instance_actions::ban_expires_at)
    .nullable()
}

#[diesel::dsl::auto_type]
/// Checks to see if a creator is banned from their community's instance
fn creator_community_instance_banned() -> _ {
  creator_community_instance_actions
    .field(instance_actions::received_ban_at)
    .nullable()
    .is_not_null()
}

#[diesel::dsl::auto_type]
fn creator_community_instance_ban_expires() -> _ {
  creator_community_instance_actions
    .field(instance_actions::ban_expires_at)
    .nullable()
}

#[diesel::dsl::auto_type]
/// Checks to see if a creator is banned from their home instance
pub fn creator_home_banned() -> _ {
  creator_home_instance_actions
    .field(instance_actions::received_ban_at)
    .nullable()
    .is_not_null()
}

#[diesel::dsl::auto_type]
/// Checks to see if a creator is banned from their home instance
pub fn creator_home_ban_expires() -> _ {
  creator_home_instance_actions
    .field(instance_actions::ban_expires_at)
    .nullable()
}

#[diesel::dsl::auto_type]
/// Checks to see if a user is site banned from any of these places:
/// - Their own instance
/// - The local instance
pub fn creator_local_home_banned() -> _ {
  creator_local_banned().or(creator_home_banned())
}

pub type CreatorLocalHomeBanExpiresType = coalesce_2_nullable<
  Timestamptz,
  Nullable<AliasedField<CreatorLocalInstanceActions, instance_actions::ban_expires_at>>,
  Nullable<AliasedField<CreatorHomeInstanceActions, instance_actions::ban_expires_at>>,
>;

pub fn creator_local_home_ban_expires() -> CreatorLocalHomeBanExpiresType {
  coalesce_2_nullable(creator_local_ban_expires(), creator_home_ban_expires())
}

/// Checks to see if a user is site banned from any of these places:
/// - The local instance
/// - Their own instance
/// - The community instance.
#[diesel::dsl::auto_type]
pub fn creator_local_home_community_banned() -> _ {
  creator_local_banned()
    .or(creator_home_banned())
    .or(creator_community_instance_banned())
}

pub type CreatorLocalHomeCommunityBanExpiresType = coalesce_3_nullable<
  Timestamptz,
  Nullable<AliasedField<CreatorLocalInstanceActions, instance_actions::ban_expires_at>>,
  Nullable<AliasedField<CreatorHomeInstanceActions, instance_actions::ban_expires_at>>,
  Nullable<AliasedField<CreatorCommunityInstanceActions, instance_actions::ban_expires_at>>,
>;

pub fn creator_local_home_community_ban_expires() -> CreatorLocalHomeCommunityBanExpiresType {
  coalesce_3_nullable(
    creator_local_ban_expires(),
    creator_home_ban_expires(),
    creator_community_instance_ban_expires(),
  )
}

#[diesel::dsl::auto_type]
fn am_higher_mod() -> _ {
  let i_became_moderator = community_actions::became_moderator_at.nullable();

  let creator_became_moderator = creator_community_actions
    .field(community_actions::became_moderator_at)
    .nullable();

  i_became_moderator.is_not_null().and(
    creator_became_moderator
      .ge(i_became_moderator)
      .is_distinct_from(false),
  )
}

/// Checks to see if you can mod an item.
///
/// Caveat: Since admin status isn't federated or ordered, it can't know whether
/// item creator is a federated admin, or a higher admin.
/// The back-end will reject an action for admin that is higher via
/// LocalUser::is_higher_mod_or_admin_check
#[diesel::dsl::auto_type]
pub fn local_user_can_mod() -> _ {
  local_user_is_admin().or(not(creator_is_admin()).and(am_higher_mod()))
}

/// Checks to see if you can mod a post.
#[diesel::dsl::auto_type]
pub fn local_user_can_mod_post() -> _ {
  local_user_is_admin().or(not(post_creator_is_admin()).and(am_higher_mod()))
}

/// Checks to see if you can mod a comment.
#[diesel::dsl::auto_type]
pub fn local_user_can_mod_comment() -> _ {
  local_user_is_admin().or(not(comment_creator_is_admin()).and(am_higher_mod()))
}

/// A special type of can_mod for communities, which dont have creators.
#[diesel::dsl::auto_type]
pub fn local_user_community_can_mod() -> _ {
  let am_admin = local_user::admin.nullable();
  let am_moderator = community_actions::became_moderator_at
    .nullable()
    .is_not_null();
  am_admin.or(am_moderator).is_not_distinct_from(true)
}

/// Selects the comment columns, but gives an empty string for content when
/// deleted or removed, and you're not a mod/admin.
#[diesel::dsl::auto_type]
pub fn comment_select_remove_deletes() -> _ {
  let deleted_or_removed = comment::deleted.or(comment::removed);

  // You can only view the content if it hasn't been removed, you're a mod or it's your own comment.
  let is_creator = local_user::person_id
    .nullable()
    .eq(comment::creator_id.nullable());
  let can_view_content = not(deleted_or_removed)
    .or(local_user_can_mod_comment())
    .or(is_creator);
  let content = case_when(can_view_content, comment::content).otherwise("");

  (
    comment::id,
    comment::creator_id,
    comment::post_id,
    content,
    comment::removed,
    comment::published_at,
    comment::updated_at,
    comment::deleted,
    comment::ap_id,
    comment::local,
    comment::path,
    comment::distinguished,
    comment::language_id,
    comment::score,
    comment::upvotes,
    comment::downvotes,
    comment::child_count,
    comment::hot_rank,
    comment::controversy_rank,
    comment::report_count,
    comment::unresolved_report_count,
    comment::federation_pending,
    comment::locked,
  )
}

/// Selects the post columns, but gives an empty string for content when
/// deleted or removed, and you're not a mod/admin.
#[diesel::dsl::auto_type]
pub fn post_select_remove_deletes() -> _ {
  let deleted_or_removed = post::deleted.or(post::removed);

  // You can only view the content if it hasn't been removed, you're a mod or it's your own post.
  let is_creator = local_user::person_id
    .nullable()
    .eq(post::creator_id.nullable());
  let can_view_content = not(deleted_or_removed)
    .or(local_user_can_mod_post())
    .or(is_creator);
  let body = case_when(can_view_content, post::body).otherwise("");

  (
    post::id,
    post::name,
    post::url,
    body,
    post::creator_id,
    post::community_id,
    post::removed,
    post::locked,
    post::published_at,
    post::updated_at,
    post::deleted,
    post::nsfw,
    post::embed_title,
    post::embed_description,
    post::thumbnail_url,
    post::ap_id,
    post::local,
    post::embed_video_url,
    post::language_id,
    post::featured_community,
    post::featured_local,
    post::url_content_type,
    post::alt_text,
    post::scheduled_publish_time_at,
    post::newest_comment_time_necro_at,
    post::newest_comment_time_at,
    post::comments,
    post::score,
    post::upvotes,
    post::downvotes,
    post::hot_rank,
    post::hot_rank_active,
    post::controversy_rank,
    post::scaled_rank,
    post::report_count,
    post::unresolved_report_count,
    post::federation_pending,
    post::embed_video_width,
    post::embed_video_height,
  )
}

#[diesel::dsl::auto_type]
// Gets the post community tags set on a specific post.
pub fn post_community_tags_fragment() -> _ {
  let sel: SqlLiteral<Json> =
    diesel::dsl::sql::<diesel::sql_types::Json>("json_agg(community_tag.*)");
  post_community_tag::table
    .inner_join(community_tag::table)
    .select(sel)
    .filter(post_community_tag::post_id.eq(post::id))
    .filter(community_tag::deleted.eq(false))
    .single_value()
}

#[diesel::dsl::auto_type]
/// Gets the tags available within a specific community
pub fn community_tags_fragment() -> _ {
  let sel: SqlLiteral<Json> =
    diesel::dsl::sql::<diesel::sql_types::Json>("json_agg(community_tag.*)");
  community_tag::table
    .select(sel)
    .filter(community_tag::community_id.eq(community::id))
    .filter(
      community_tag::deleted
        .eq(false)
        // Show deleted tags for admins and mods
        .or(local_user_community_can_mod()),
    )
    .single_value()
}

/// The select for the person1 alias.
pub fn person1_select() -> Person1AliasAllColumnsTuple {
  person1.fields(person::all_columns)
}

/// The select for the person2 alias.
pub fn person2_select() -> Person2AliasAllColumnsTuple {
  person2.fields(person::all_columns)
}
