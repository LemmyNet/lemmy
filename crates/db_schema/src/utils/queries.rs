use crate::{
  aliases::{
    creator_community_actions,
    creator_community_instance_actions,
    creator_home_instance_actions,
    creator_local_instance_actions,
    creator_local_user,
    my_instance_persons_actions,
    person1,
    person2,
  },
  newtypes::{InstanceId, PersonId},
  utils::functions::{get_controversy_rank, get_hot_rank, get_score},
  MyInstancePersonsActionsAllColumnsTuple,
  Person1AliasAllColumnsTuple,
  Person2AliasAllColumnsTuple,
};
use diesel::{
  dsl::{case_when, exists, not},
  expression::SqlLiteral,
  helper_types::{Eq, NotEq, Nullable},
  sql_types,
  sql_types::Json,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  PgExpressionMethods,
  QueryDsl,
};
use lemmy_db_schema_file::{
  enums::{CommunityFollowerState, CommunityVisibility},
  schema::{
    comment,
    comment_actions,
    community,
    community_actions,
    image_details,
    instance_actions,
    local_site,
    local_user,
    multi_community,
    multi_community_entry,
    person,
    person_actions,
    post,
    post_actions,
    post_tag,
    tag,
  },
};

/// Hide all content from blocked communities and persons. Content from blocked instances is also
/// hidden, unless the user followed the community explicitly.
#[diesel::dsl::auto_type]
pub fn filter_blocked() -> _ {
  instance_actions::blocked_communities_at
    .is_null()
    .or(community_actions::followed_at.is_not_null())
    .and(community_actions::blocked_at.is_null())
    .and(person_actions::blocked_at.is_null())
    .and(
      my_instance_persons_actions
        .field(instance_actions::blocked_persons_at)
        .is_null(),
    )
}

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
pub fn creator_home_banned() -> _ {
  creator_home_instance_actions
    .field(instance_actions::received_ban_at)
    .nullable()
    .is_not_null()
}

#[diesel::dsl::auto_type]
/// Checks to see if a user is site banned from any of these places:
/// - Their own instance
/// - The local instance
pub fn creator_banned() -> _ {
  let local_ban = creator_local_instance_actions
    .field(instance_actions::received_ban_at)
    .nullable()
    .is_not_null();
  local_ban.or(creator_home_banned())
}

/// Similar to creator_banned(), but also checks if creator was banned from instance where the
/// community is hosted.
#[diesel::dsl::auto_type]
pub fn creator_banned_within_community() -> _ {
  let community_ban = creator_community_instance_actions
    .field(instance_actions::received_ban_at)
    .nullable()
    .is_not_null();
  creator_banned().or(community_ban)
}

#[diesel::dsl::auto_type]
pub fn creator_local_user_admin_join() -> _ {
  creator_local_user.on(
    person::id
      .eq(creator_local_user.field(local_user::person_id))
      .and(creator_local_user.field(local_user::admin).eq(true)),
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

// TODO: move this
pub trait NullableExpression {
  type InnerSqlType;
}
impl<ST, T: diesel::expression::Expression<SqlType = sql_types::Nullable<ST>>> NullableExpression
  for T
{
  type InnerSqlType = ST;
}
pub fn coalesce<
  X: NullableExpression
    + diesel::expression::Expression<SqlType = sql_types::Nullable<X::InnerSqlType>>,
  Y: diesel::expression::AsExpression<X::InnerSqlType>,
>(
  x: X,
  y: Y,
) -> crate::utils::functions::coalesce<X::InnerSqlType, X, Y>
where
  X::InnerSqlType: diesel::sql_types::SqlType + diesel::sql_types::SingleValue,
{
  crate::utils::functions::coalesce(x, y)
}
#[expect(non_camel_case_types)]
pub type coalesce<X, Y> =
  crate::utils::functions::coalesce<<X as NullableExpression>::InnerSqlType, X, Y>;

/// Selects the comment columns, but gives an empty string for content when
/// deleted or removed, and you're not a mod/admin.
#[diesel::dsl::auto_type]
pub fn comment_select_remove_deletes() -> _ {
  let deleted_or_removed = comment::deleted.or(comment::removed);

  // You can only view the content if it hasn't been removed, or you can mod.
  let can_view_content = not(deleted_or_removed).or(local_user_can_mod_comment());
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
    get_score(comment::non_1_upvotes, comment::non_0_downvotes),
    coalesce(comment::non_1_upvotes, 1i32),
    coalesce(comment::non_0_downvotes, 0i32),
    coalesce(comment::non_0_child_count, 0i32),
    get_hot_rank(
      comment::non_1_upvotes,
      comment::non_0_downvotes,
      comment::age,
    ),
    get_controversy_rank(comment::non_1_upvotes, comment::non_0_downvotes),
    comment::age,
    coalesce(comment::non_0_report_count, 0i16),
    coalesce(comment::non_0_unresolved_report_count, 0i16),
    comment::federation_pending,
  )
}

#[diesel::dsl::auto_type]
// Gets the post tags set on a specific post
pub fn post_tags_fragment() -> _ {
  let sel: SqlLiteral<Json> = diesel::dsl::sql::<diesel::sql_types::Json>("json_agg(tag.*)");
  post_tag::table
    .inner_join(tag::table)
    .select(sel)
    .filter(post_tag::post_id.eq(post::id))
    .filter(tag::deleted.eq(false))
    .single_value()
}

#[diesel::dsl::auto_type]
/// Gets the post tags available within a specific community
pub fn community_post_tags_fragment() -> _ {
  let sel: SqlLiteral<Json> = diesel::dsl::sql::<diesel::sql_types::Json>("json_agg(tag.*)");
  tag::table
    .select(sel)
    .filter(tag::community_id.eq(community::id))
    .filter(tag::deleted.eq(false))
    .single_value()
}

#[diesel::dsl::auto_type]
// `Clone` is required by the `impl<S, C>
// diesel::internal::table_macro::FieldAliasMapperAssociatedTypesDisjointnessTrick<table, S, C> for
// table` block proudly found in the implementation of the `table` macro. TODO: maybe open diesel
// issue
pub fn person_alias_as_select<
  S: diesel::query_source::AliasSource<Target = person::table> + Clone,
>(
  alias: diesel::query_source::Alias<S>,
) -> _ {
  (
    alias.field(person::id),
    alias.field(person::name),
    alias.field(person::display_name),
    alias.field(person::avatar),
    alias.field(person::published_at),
    alias.field(person::updated_at),
    alias.field(person::ap_id),
    alias.field(person::bio),
    alias.field(person::local),
    alias.field(person::private_key),
    alias.field(person::public_key),
    alias.field(person::last_refreshed_at),
    alias.field(person::banner),
    alias.field(person::deleted),
    alias.field(person::inbox_url),
    alias.field(person::matrix_user_id),
    alias.field(person::bot_account),
    alias.field(person::instance_id),
    coalesce(alias.field(person::non_0_post_count), 0i32),
    coalesce(alias.field(person::non_0_post_score), 0i32),
    coalesce(alias.field(person::non_0_comment_count), 0i32),
    coalesce(alias.field(person::non_0_comment_score), 0i32),
  )
}

/// The select for the person1 alias.
pub fn person1_select() -> Person1AliasAllColumnsTuple {
  person_alias_as_select(person1)
}

/// The select for the person2 alias.
pub fn person2_select() -> Person2AliasAllColumnsTuple {
  person_alias_as_select(person2)
}

type IsSubscribedType =
  Eq<lemmy_db_schema_file::schema::community_actions::follow_state, Option<CommunityFollowerState>>;

pub fn filter_is_subscribed() -> IsSubscribedType {
  community_actions::follow_state.eq(Some(CommunityFollowerState::Accepted))
}

type IsNotUnlistedType =
  NotEq<lemmy_db_schema_file::schema::community::visibility, CommunityVisibility>;

#[diesel::dsl::auto_type]
pub fn filter_not_unlisted_or_is_subscribed() -> _ {
  let not_unlisted: IsNotUnlistedType = community::visibility.ne(CommunityVisibility::Unlisted);
  let is_subscribed: IsSubscribedType = filter_is_subscribed();
  not_unlisted.or(is_subscribed)
}

#[diesel::dsl::auto_type]
pub fn community_join() -> _ {
  community::table.on(post::community_id.eq(community::id))
}

#[diesel::dsl::auto_type]
pub fn creator_home_instance_actions_join() -> _ {
  creator_home_instance_actions.on(
    creator_home_instance_actions
      .field(instance_actions::instance_id)
      .eq(person::instance_id)
      .and(
        creator_home_instance_actions
          .field(instance_actions::person_id)
          .eq(person::id),
      ),
  )
}
#[diesel::dsl::auto_type]
pub fn creator_community_instance_actions_join() -> _ {
  creator_community_instance_actions.on(
    creator_home_instance_actions
      .field(instance_actions::instance_id)
      .eq(community::instance_id)
      .and(
        creator_community_instance_actions
          .field(instance_actions::person_id)
          .eq(person::id),
      ),
  )
}

/// join with instance actions for local instance
///
/// Requires annotation for return type, see https://docs.diesel.rs/2.2.x/diesel/dsl/attr.auto_type.html#annotating-types
#[diesel::dsl::auto_type]
pub fn creator_local_instance_actions_join(local_instance_id: InstanceId) -> _ {
  creator_local_instance_actions.on(
    creator_local_instance_actions
      .field(instance_actions::instance_id)
      .eq(local_instance_id)
      .and(
        creator_local_instance_actions
          .field(instance_actions::person_id)
          .eq(person::id),
      ),
  )
}

/// Your instance actions for the community's instance.
#[diesel::dsl::auto_type]
pub fn my_instance_communities_actions_join(my_person_id: Option<PersonId>) -> _ {
  instance_actions::table.on(
    instance_actions::instance_id
      .eq(community::instance_id)
      .and(instance_actions::person_id.nullable().eq(my_person_id)),
  )
}

/// Your instance actions for the person's instance.
#[diesel::dsl::auto_type]
pub fn my_instance_persons_actions_join(my_person_id: Option<PersonId>) -> _ {
  instance_actions::table.on(
    instance_actions::instance_id
      .eq(person::instance_id)
      .and(instance_actions::person_id.nullable().eq(my_person_id)),
  )
}

/// The select for the my_instance_persons_actions alias
pub fn my_instance_persons_actions_select() -> Nullable<MyInstancePersonsActionsAllColumnsTuple> {
  my_instance_persons_actions
    .fields(instance_actions::all_columns)
    .nullable()
}

/// Your instance actions for the person's instance.
/// A dupe of the above function, but aliased
#[diesel::dsl::auto_type]
pub fn my_instance_persons_actions_join_1(my_person_id: Option<PersonId>) -> _ {
  my_instance_persons_actions.on(
    my_instance_persons_actions
      .field(instance_actions::instance_id)
      .eq(person::instance_id)
      .and(
        my_instance_persons_actions
          .field(instance_actions::person_id)
          .nullable()
          .eq(my_person_id),
      ),
  )
}
#[diesel::dsl::auto_type]
pub fn image_details_join() -> _ {
  image_details::table.on(post::thumbnail_url.eq(image_details::link.nullable()))
}

#[diesel::dsl::auto_type]
pub fn my_community_actions_join(my_person_id: Option<PersonId>) -> _ {
  community_actions::table.on(
    community_actions::community_id
      .eq(community::id)
      .and(community_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_post_actions_join(my_person_id: Option<PersonId>) -> _ {
  post_actions::table.on(
    post_actions::post_id
      .eq(post::id)
      .and(post_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_comment_actions_join(my_person_id: Option<PersonId>) -> _ {
  comment_actions::table.on(
    comment_actions::comment_id
      .eq(comment::id)
      .and(comment_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_person_actions_join(my_person_id: Option<PersonId>) -> _ {
  person_actions::table.on(
    person_actions::target_id
      .eq(person::id)
      .and(person_actions::person_id.nullable().eq(my_person_id)),
  )
}

#[diesel::dsl::auto_type]
pub fn my_local_user_admin_join(my_person_id: Option<PersonId>) -> _ {
  local_user::table.on(
    local_user::person_id
      .nullable()
      .eq(my_person_id)
      .and(local_user::admin.eq(true)),
  )
}

#[diesel::dsl::auto_type]
pub fn creator_community_actions_join() -> _ {
  creator_community_actions.on(
    creator_community_actions
      .field(community_actions::community_id)
      .eq(community::id)
      .and(
        creator_community_actions
          .field(community_actions::person_id)
          .eq(person::id),
      ),
  )
}

#[diesel::dsl::auto_type]
pub fn suggested_communities() -> _ {
  community::id.eq_any(
    local_site::table
      .left_join(multi_community::table.inner_join(multi_community_entry::table))
      .filter(multi_community_entry::community_id.is_not_null())
      .select(multi_community_entry::community_id.assume_not_null()),
  )
}
