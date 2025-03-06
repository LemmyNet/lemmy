// TODO move all of the view custom selects here

use diesel::{
  dsl::{case_when, exists, not, Nullable},
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  PgExpressionMethods,
  QueryDsl,
};
use lemmy_db_schema::{
  aliases::{creator_community_actions, creator_local_user, person1},
  schema::{comment, community_actions, instance_actions, local_user, person, person_actions},
  Person1AliasAllColumnsTuple,
};

/// Hide all content from blocked communities and persons. Content from blocked instances is also
/// hidden, unless the user followed the community explicitly.
#[diesel::dsl::auto_type]
pub(crate) fn filter_blocked() -> _ {
  instance_actions::blocked
    .is_null()
    .or(community_actions::followed.is_not_null())
    .and(community_actions::blocked.is_null())
    .and(person_actions::blocked.is_null())
}

/// Checks to see if the comment creator is an admin.
#[diesel::dsl::auto_type]
pub(crate) fn comment_creator_is_admin() -> _ {
  exists(
    creator_local_user.filter(
      comment::creator_id
        .eq(creator_local_user.field(local_user::person_id))
        .and(creator_local_user.field(local_user::admin).eq(true)),
    ),
  )
}

/// Checks to see if you can mod an item.
///
/// Caveat: Since admin status isn't federated or ordered, it can't know whether
/// item creator is a federated admin, or a higher admin.
/// The back-end will reject an action for admin that is higher via
/// LocalUser::is_higher_mod_or_admin_check
#[diesel::dsl::auto_type]
pub(crate) fn local_user_can_mod() -> _ {
  let am_admin = local_user::admin.nullable();
  let creator_became_moderator = creator_community_actions
    .field(community_actions::became_moderator)
    .nullable();

  let am_higher_mod = community_actions::became_moderator
    .nullable()
    .le(creator_became_moderator);

  am_admin.or(am_higher_mod).is_not_distinct_from(true)
}

/// A special type of can_mod for communities, which dont have creators
#[diesel::dsl::auto_type]
pub(crate) fn local_user_community_can_mod() -> _ {
  local_user::admin
    .nullable()
    .or(community_actions::became_moderator.nullable().is_not_null())
    .is_not_distinct_from(true)
}

/// Selects the comment columns, but gives an empty string for content when
/// deleted or removed, and you're not a mod/admin.
#[diesel::dsl::auto_type]
pub fn comment_select_remove_deletes() -> _ {
  let deleted_or_removed = comment::deleted.or(comment::removed);

  // You can only view the content if it hasn't been removed, or you can mod.
  let can_view_content = not(deleted_or_removed).or(local_user_can_mod());
  let content = case_when(can_view_content, comment::content).otherwise("");

  (
    comment::id,
    comment::creator_id,
    comment::post_id,
    content,
    comment::removed,
    comment::published,
    comment::updated,
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
  )
}

/// The select for the person1 alias
#[diesel::dsl::auto_type]
pub fn person1_select() -> Nullable<Person1AliasAllColumnsTuple> {
  person1.fields(person::all_columns).nullable()
}
