use crate::ModlogCombinedViewInternal;
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  IntoSql,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  aliases,
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommentId, CommunityId, PersonId, PostId},
  source::{
    combined::modlog::{modlog_combined_keys as key, ModlogCombined},
    local_user::LocalUser,
  },
  utils::{
    get_conn,
    limit_fetch,
    paginate,
    queries::{filter_is_subscribed, filter_not_unlisted_or_is_subscribed},
    DbPool,
  },
  ModlogActionType,
};
use lemmy_db_schema_file::{
  enums::ListingType,
  schema::{
    admin_allow_instance,
    admin_block_instance,
    admin_purge_comment,
    admin_purge_community,
    admin_purge_person,
    admin_purge_post,
    comment,
    community,
    community_actions,
    instance,
    mod_add,
    mod_add_community,
    mod_ban,
    mod_ban_from_community,
    mod_change_community_visibility,
    mod_feature_post,
    mod_lock_post,
    mod_remove_comment,
    mod_remove_community,
    mod_remove_post,
    mod_transfer_community,
    modlog_combined,
    person,
    post,
  },
};
use lemmy_utils::error::LemmyResult;

impl ModlogCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(
    mod_person_id: Option<PersonId>,
    hide_modlog_names: Option<bool>,
    my_person_id: Option<PersonId>,
  ) -> _ {
    // The modded / other person
    let other_person = aliases::person1.field(person::id);

    let show_mod_names: bool = !(hide_modlog_names.unwrap_or_default());
    let show_mod_names_expr = show_mod_names.into_sql::<diesel::sql_types::Bool>();

    // The query for the admin / mod person
    // It needs an OR condition to every mod table
    // After this you can use person::id to refer to the moderator
    let moderator_names_join = person::table.on(
      show_mod_names_expr
        .or(person::id.nullable().eq(mod_person_id))
        .and(
          admin_allow_instance::admin_person_id
            .eq(person::id)
            .or(admin_block_instance::admin_person_id.eq(person::id))
            .or(admin_purge_comment::admin_person_id.eq(person::id))
            .or(admin_purge_community::admin_person_id.eq(person::id))
            .or(admin_purge_person::admin_person_id.eq(person::id))
            .or(admin_purge_post::admin_person_id.eq(person::id))
            .or(mod_add::mod_person_id.eq(person::id))
            .or(mod_add_community::mod_person_id.eq(person::id))
            .or(mod_ban::mod_person_id.eq(person::id))
            .or(mod_ban_from_community::mod_person_id.eq(person::id))
            .or(mod_feature_post::mod_person_id.eq(person::id))
            .or(mod_change_community_visibility::mod_person_id.eq(person::id))
            .or(mod_lock_post::mod_person_id.eq(person::id))
            .or(mod_remove_comment::mod_person_id.eq(person::id))
            .or(mod_remove_community::mod_person_id.eq(person::id))
            .or(mod_remove_post::mod_person_id.eq(person::id))
            .or(mod_transfer_community::mod_person_id.eq(person::id)),
        ),
    );

    let other_person_join = aliases::person1.on(
      mod_add::other_person_id
        .eq(other_person)
        .or(mod_add_community::other_person_id.eq(other_person))
        .or(mod_ban::other_person_id.eq(other_person))
        .or(mod_ban_from_community::other_person_id.eq(other_person))
        // Some tables don't have the other_person_id directly, so you need to join
        .or(
          mod_feature_post::id
            .is_not_null()
            .and(post::creator_id.eq(other_person)),
        )
        .or(
          mod_lock_post::id
            .is_not_null()
            .and(post::creator_id.eq(other_person)),
        )
        .or(
          mod_remove_comment::id
            .is_not_null()
            .and(comment::creator_id.eq(other_person)),
        )
        .or(
          mod_remove_post::id
            .is_not_null()
            .and(post::creator_id.eq(other_person)),
        )
        .or(mod_transfer_community::other_person_id.eq(other_person)),
    );

    let comment_join = comment::table.on(mod_remove_comment::comment_id.eq(comment::id));

    let post_join = post::table.on(
      admin_purge_comment::post_id
        .eq(post::id)
        .or(mod_feature_post::post_id.eq(post::id))
        .or(mod_lock_post::post_id.eq(post::id))
        .or(
          mod_remove_comment::id
            .is_not_null()
            .and(comment::post_id.eq(post::id)),
        )
        .or(mod_remove_post::post_id.eq(post::id)),
    );

    let community_join = community::table.on(
      admin_purge_post::community_id
        .eq(community::id)
        .or(mod_add_community::community_id.eq(community::id))
        .or(mod_ban_from_community::community_id.eq(community::id))
        .or(
          mod_feature_post::id
            .is_not_null()
            .and(post::community_id.eq(community::id)),
        )
        .or(mod_change_community_visibility::community_id.eq(community::id))
        .or(
          mod_lock_post::id
            .is_not_null()
            .and(post::community_id.eq(community::id)),
        )
        .or(
          mod_remove_comment::id
            .is_not_null()
            .and(post::community_id.eq(community::id)),
        )
        .or(mod_remove_community::community_id.eq(community::id))
        .or(
          mod_remove_post::id
            .is_not_null()
            .and(post::community_id.eq(community::id)),
        )
        .or(mod_transfer_community::community_id.eq(community::id)),
    );

    let instance_join = instance::table.on(
      admin_allow_instance::instance_id
        .eq(instance::id)
        .or(admin_block_instance::instance_id.eq(instance::id)),
    );

    let community_actions_join = community_actions::table.on(
      community_actions::community_id
        .eq(community::id)
        .and(community_actions::person_id.nullable().eq(my_person_id)),
    );

    modlog_combined::table
      .left_join(admin_allow_instance::table)
      .left_join(admin_block_instance::table)
      .left_join(admin_purge_comment::table)
      .left_join(admin_purge_community::table)
      .left_join(admin_purge_person::table)
      .left_join(admin_purge_post::table)
      .left_join(mod_add::table)
      .left_join(mod_add_community::table)
      .left_join(mod_ban::table)
      .left_join(mod_ban_from_community::table)
      .left_join(mod_feature_post::table)
      .left_join(mod_change_community_visibility::table)
      .left_join(mod_lock_post::table)
      .left_join(mod_remove_comment::table)
      .left_join(mod_remove_community::table)
      .left_join(mod_remove_post::table)
      .left_join(mod_transfer_community::table)
      .left_join(moderator_names_join)
      .left_join(comment_join)
      .left_join(post_join)
      .left_join(community_join)
      .left_join(instance_join)
      .left_join(other_person_join)
      .left_join(community_actions_join)
  }
}
#[derive(Default)]
/// Querying / filtering the modlog.
pub struct ModlogCombinedQuery<'a> {
  pub type_: Option<ModlogActionType>,
  pub listing_type: Option<ListingType>,
  pub comment_id: Option<CommentId>,
  pub post_id: Option<PostId>,
  pub community_id: Option<CommunityId>,
  pub hide_modlog_names: Option<bool>,
  pub local_user: Option<&'a LocalUser>,
  pub mod_person_id: Option<PersonId>,
  pub other_person_id: Option<PersonId>,
  pub cursor_data: Option<ModlogCombined>,
  pub page_back: Option<bool>,
  pub limit: Option<i64>,
}

impl ModlogCombinedQuery<'_> {
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<Vec<ModlogCombinedViewInternal>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(self.limit)?;

    let other_person = aliases::person1.field(person::id);
    let my_person_id = self.local_user.person_id();

    let mut query =
      ModlogCombinedViewInternal::joins(self.mod_person_id, self.hide_modlog_names, my_person_id)
        .select(ModlogCombinedViewInternal::as_select())
        .limit(limit)
        .into_boxed();

    if let Some(mod_person_id) = self.mod_person_id {
      query = query.filter(person::id.eq(mod_person_id));
    };

    if let Some(other_person_id) = self.other_person_id {
      query = query.filter(other_person.eq(other_person_id));
    };

    if let Some(community_id) = self.community_id {
      query = query.filter(community::id.eq(community_id))
    }

    if let Some(post_id) = self.post_id {
      query = query.filter(post::id.eq(post_id))
    }

    if let Some(comment_id) = self.comment_id {
      query = query.filter(comment::id.eq(comment_id))
    }

    if let Some(type_) = self.type_ {
      use lemmy_db_schema::ModlogActionType::*;
      query = match type_ {
        All => query,
        ModRemovePost => query.filter(modlog_combined::mod_remove_post_id.is_not_null()),
        ModLockPost => query.filter(modlog_combined::mod_lock_post_id.is_not_null()),
        ModFeaturePost => query.filter(modlog_combined::mod_feature_post_id.is_not_null()),
        ModRemoveComment => query.filter(modlog_combined::mod_remove_comment_id.is_not_null()),
        ModRemoveCommunity => query.filter(modlog_combined::mod_remove_community_id.is_not_null()),
        ModBanFromCommunity => {
          query.filter(modlog_combined::mod_ban_from_community_id.is_not_null())
        }
        ModAddCommunity => query.filter(modlog_combined::mod_add_community_id.is_not_null()),
        ModTransferCommunity => {
          query.filter(modlog_combined::mod_transfer_community_id.is_not_null())
        }
        ModAdd => query.filter(modlog_combined::mod_add_id.is_not_null()),
        ModBan => query.filter(modlog_combined::mod_ban_id.is_not_null()),
        ModChangeCommunityVisibility => {
          query.filter(modlog_combined::mod_change_community_visibility_id.is_not_null())
        }
        AdminPurgePerson => query.filter(modlog_combined::admin_purge_person_id.is_not_null()),
        AdminPurgeCommunity => {
          query.filter(modlog_combined::admin_purge_community_id.is_not_null())
        }
        AdminPurgePost => query.filter(modlog_combined::admin_purge_post_id.is_not_null()),
        AdminPurgeComment => query.filter(modlog_combined::admin_purge_comment_id.is_not_null()),
        AdminBlockInstance => query.filter(modlog_combined::admin_block_instance_id.is_not_null()),
        AdminAllowInstance => query.filter(modlog_combined::admin_allow_instance_id.is_not_null()),
      }
    }

    query = match self.listing_type.unwrap_or(ListingType::All) {
      ListingType::All => query,
      ListingType::Subscribed => query.filter(filter_is_subscribed()),
      ListingType::Local => query
        .filter(community::local.eq(true))
        .filter(filter_not_unlisted_or_is_subscribed()),
      ListingType::ModeratorView => query.filter(community_actions::became_moderator.is_not_null()),
    };

    // Sorting by published
    let paginated_query = paginate(
      query,
      SortDirection::Desc,
      self.cursor_data,
      None,
      self.page_back,
    )
    .then_order_by(key::published)
    // Tie breaker
    .then_order_by(key::id);

    Ok(
      paginated_query
        .load::<ModlogCombinedViewInternal>(conn)
        .await?,
    )
  }
}
