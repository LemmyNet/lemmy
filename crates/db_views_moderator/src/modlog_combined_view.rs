use crate::structs::{
  AdminAllowInstanceView,
  AdminBlockInstanceView,
  AdminPurgeCommentView,
  AdminPurgeCommunityView,
  AdminPurgePersonView,
  AdminPurgePostView,
  ModAddCommunityView,
  ModAddView,
  ModBanFromCommunityView,
  ModBanView,
  ModFeaturePostView,
  ModHideCommunityView,
  ModLockPostView,
  ModRemoveCommentView,
  ModRemoveCommunityView,
  ModRemovePostView,
  ModTransferCommunityView,
  ModlogCombinedPaginationCursor,
  ModlogCombinedView,
  ModlogCombinedViewInternal,
};
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  IntoSql,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::PaginatedQueryBuilder;
use lemmy_db_schema::{
  aliases,
  newtypes::{CommentId, CommunityId, PersonId, PostId},
  schema::{
    admin_allow_instance,
    admin_block_instance,
    admin_purge_comment,
    admin_purge_community,
    admin_purge_person,
    admin_purge_post,
    comment,
    community,
    instance,
    mod_add,
    mod_add_community,
    mod_ban,
    mod_ban_from_community,
    mod_feature_post,
    mod_hide_community,
    mod_lock_post,
    mod_remove_comment,
    mod_remove_community,
    mod_remove_post,
    mod_transfer_community,
    modlog_combined,
    person,
    post,
  },
  source::combined::modlog::{modlog_combined_keys as key, ModlogCombined},
  utils::{get_conn, DbPool},
  InternalToCombinedView,
  ModlogActionType,
};
use lemmy_utils::error::LemmyResult;

impl ModlogCombinedPaginationCursor {
  // get cursor for page that starts immediately after the given post
  pub fn after_post(view: &ModlogCombinedView) -> ModlogCombinedPaginationCursor {
    let (prefix, id) = match view {
      ModlogCombinedView::AdminAllowInstance(v) => {
        ("AdminAllowInstance", v.admin_allow_instance.id.0)
      }
      ModlogCombinedView::AdminBlockInstance(v) => {
        ("AdminBlockInstance", v.admin_block_instance.id.0)
      }
      ModlogCombinedView::AdminPurgeComment(v) => ("AdminPurgeComment", v.admin_purge_comment.id.0),
      ModlogCombinedView::AdminPurgeCommunity(v) => {
        ("AdminPurgeCommunity", v.admin_purge_community.id.0)
      }
      ModlogCombinedView::AdminPurgePerson(v) => ("AdminPurgePerson", v.admin_purge_person.id.0),
      ModlogCombinedView::AdminPurgePost(v) => ("AdminPurgePost", v.admin_purge_post.id.0),
      ModlogCombinedView::ModAdd(v) => ("ModAdd", v.mod_add.id.0),
      ModlogCombinedView::ModAddCommunity(v) => ("ModAddCommunity", v.mod_add_community.id.0),
      ModlogCombinedView::ModBan(v) => ("ModBan", v.mod_ban.id.0),
      ModlogCombinedView::ModBanFromCommunity(v) => {
        ("ModBanFromCommunity", v.mod_ban_from_community.id.0)
      }
      ModlogCombinedView::ModFeaturePost(v) => ("ModFeaturePost", v.mod_feature_post.id.0),
      ModlogCombinedView::ModHideCommunity(v) => ("ModHideCommunity", v.mod_hide_community.id.0),
      ModlogCombinedView::ModLockPost(v) => ("ModLockPost", v.mod_lock_post.id.0),
      ModlogCombinedView::ModRemoveComment(v) => ("ModRemoveComment", v.mod_remove_comment.id.0),
      ModlogCombinedView::ModRemoveCommunity(v) => {
        ("ModRemoveCommunity", v.mod_remove_community.id.0)
      }
      ModlogCombinedView::ModRemovePost(v) => ("ModRemovePost", v.mod_remove_post.id.0),
      ModlogCombinedView::ModTransferCommunity(v) => {
        ("ModTransferCommunity", v.mod_transfer_community.id.0)
      }
    };
    // hex encoding to prevent ossification
    ModlogCombinedPaginationCursor(format!("{prefix}{id:x}"))
  }

  pub async fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
    let err_msg = || Error::QueryBuilderError("Could not parse pagination token".into());
    let mut query = modlog_combined::table
      .select(ModlogCombined::as_select())
      .into_boxed();
    let (prefix, id_str) = self.0.split_at_checked(1).ok_or_else(err_msg)?;
    let id = i32::from_str_radix(id_str, 16).map_err(|_err| err_msg())?;
    query = match prefix {
      "AdminAllowInstance" => query.filter(modlog_combined::admin_allow_instance_id.eq(id)),
      "AdminBlockInstance" => query.filter(modlog_combined::admin_block_instance_id.eq(id)),
      "AdminPurgeComment" => query.filter(modlog_combined::admin_purge_comment_id.eq(id)),
      "AdminPurgeCommunity" => query.filter(modlog_combined::admin_purge_community_id.eq(id)),
      "AdminPurgePerson" => query.filter(modlog_combined::admin_purge_person_id.eq(id)),
      "AdminPurgePost" => query.filter(modlog_combined::admin_purge_post_id.eq(id)),
      "ModAdd" => query.filter(modlog_combined::mod_add_id.eq(id)),
      "ModAddCommunity" => query.filter(modlog_combined::mod_add_community_id.eq(id)),
      "ModBan" => query.filter(modlog_combined::mod_ban_id.eq(id)),
      "ModBanFromCommunity" => query.filter(modlog_combined::mod_ban_from_community_id.eq(id)),
      "ModFeaturePost" => query.filter(modlog_combined::mod_feature_post_id.eq(id)),
      "ModHideCommunity" => query.filter(modlog_combined::mod_hide_community_id.eq(id)),
      "ModLockPost" => query.filter(modlog_combined::mod_lock_post_id.eq(id)),
      "ModRemoveComment" => query.filter(modlog_combined::mod_remove_comment_id.eq(id)),
      "ModRemoveCommunity" => query.filter(modlog_combined::mod_remove_community_id.eq(id)),
      "ModRemovePost" => query.filter(modlog_combined::mod_remove_post_id.eq(id)),
      "ModTransferCommunity" => query.filter(modlog_combined::mod_transfer_community_id.eq(id)),

      _ => return Err(err_msg()),
    };
    let token = query.first(&mut get_conn(pool).await?).await?;

    Ok(PaginationCursorData(token))
  }
}

#[derive(Clone)]
pub struct PaginationCursorData(ModlogCombined);

#[derive(Default)]
/// Querying / filtering the modlog.
pub struct ModlogCombinedQuery {
  pub type_: Option<ModlogActionType>,
  pub comment_id: Option<CommentId>,
  pub post_id: Option<PostId>,
  pub community_id: Option<CommunityId>,
  pub hide_modlog_names: bool,
  pub mod_person_id: Option<PersonId>,
  pub modded_person_id: Option<PersonId>,
  pub page_after: Option<PaginationCursorData>,
  pub page_back: Option<bool>,
}

impl ModlogCombinedQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<Vec<ModlogCombinedView>> {
    let conn = &mut get_conn(pool).await?;

    let mod_person = self.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !self.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let modded_person = aliases::person1.field(person::id);

    // The query for the admin / mod person
    // It needs an OR condition to every mod table
    // After this you can use person::id to refer to the moderator
    let moderator_names_join = show_mod_names_expr.or(person::id.eq(mod_person)).and(
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
        .or(mod_hide_community::mod_person_id.eq(person::id))
        .or(mod_lock_post::mod_person_id.eq(person::id))
        .or(mod_remove_comment::mod_person_id.eq(person::id))
        .or(mod_remove_community::mod_person_id.eq(person::id))
        .or(mod_remove_post::mod_person_id.eq(person::id))
        .or(mod_transfer_community::mod_person_id.eq(person::id)),
    );

    let modded_person_join = mod_add::other_person_id
      .eq(modded_person)
      .or(mod_add_community::other_person_id.eq(modded_person))
      .or(mod_ban::other_person_id.eq(modded_person))
      .or(mod_ban_from_community::other_person_id.eq(modded_person))
      // Some tables don't have the modded_person_id directly, so you need to join
      .or(
        mod_feature_post::id
          .is_not_null()
          .and(post::creator_id.eq(modded_person)),
      )
      .or(
        mod_lock_post::id
          .is_not_null()
          .and(post::creator_id.eq(modded_person)),
      )
      .or(
        mod_remove_comment::id
          .is_not_null()
          .and(comment::creator_id.eq(modded_person)),
      )
      .or(
        mod_remove_post::id
          .is_not_null()
          .and(post::creator_id.eq(modded_person)),
      )
      .or(mod_transfer_community::other_person_id.eq(modded_person));

    let comment_join = mod_remove_comment::comment_id.eq(comment::id);

    let post_join = admin_purge_comment::post_id
      .eq(post::id)
      .or(mod_feature_post::post_id.eq(post::id))
      .or(mod_lock_post::post_id.eq(post::id))
      .or(
        mod_remove_comment::id
          .is_not_null()
          .and(comment::post_id.eq(post::id)),
      )
      .or(mod_remove_post::post_id.eq(post::id));

    let community_join = admin_purge_post::community_id
      .eq(community::id)
      .or(mod_add_community::community_id.eq(community::id))
      .or(mod_ban_from_community::community_id.eq(community::id))
      .or(
        mod_feature_post::id
          .is_not_null()
          .and(post::community_id.eq(community::id)),
      )
      .or(mod_hide_community::community_id.eq(community::id))
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
      .or(mod_transfer_community::community_id.eq(community::id));

    let instance_join = admin_allow_instance::instance_id
      .eq(instance::id)
      .or(admin_block_instance::instance_id.eq(instance::id));

    let mut query = modlog_combined::table
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
      .left_join(mod_hide_community::table)
      .left_join(mod_lock_post::table)
      .left_join(mod_remove_comment::table)
      .left_join(mod_remove_community::table)
      .left_join(mod_remove_post::table)
      .left_join(mod_transfer_community::table)
      // The moderator
      .left_join(person::table.on(moderator_names_join))
      // The comment
      .left_join(comment::table.on(comment_join))
      // The post
      .left_join(post::table.on(post_join))
      // The community
      .left_join(community::table.on(community_join))
      // The instance
      .left_join(instance::table.on(instance_join))
      // The modded person
      .left_join(aliases::person1.on(modded_person_join))
      .select((
        admin_allow_instance::all_columns.nullable(),
        admin_block_instance::all_columns.nullable(),
        admin_purge_comment::all_columns.nullable(),
        admin_purge_community::all_columns.nullable(),
        admin_purge_person::all_columns.nullable(),
        admin_purge_post::all_columns.nullable(),
        mod_add::all_columns.nullable(),
        mod_add_community::all_columns.nullable(),
        mod_ban::all_columns.nullable(),
        mod_ban_from_community::all_columns.nullable(),
        mod_feature_post::all_columns.nullable(),
        mod_hide_community::all_columns.nullable(),
        mod_lock_post::all_columns.nullable(),
        mod_remove_comment::all_columns.nullable(),
        mod_remove_community::all_columns.nullable(),
        mod_remove_post::all_columns.nullable(),
        mod_transfer_community::all_columns.nullable(),
        // Shared
        person::all_columns.nullable(),
        aliases::person1.fields(person::all_columns).nullable(),
        instance::all_columns.nullable(),
        community::all_columns.nullable(),
        post::all_columns.nullable(),
        comment::all_columns.nullable(),
      ))
      .into_boxed();

    if let Some(mod_person_id) = self.mod_person_id {
      query = query.filter(person::id.eq(mod_person_id));
    };

    if let Some(modded_person_id) = self.modded_person_id {
      query = query.filter(modded_person.eq(modded_person_id));
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
      query = match type_ {
        ModlogActionType::All => query,
        ModlogActionType::ModRemovePost => {
          query.filter(modlog_combined::mod_remove_post_id.is_not_null())
        }
        ModlogActionType::ModLockPost => {
          query.filter(modlog_combined::mod_lock_post_id.is_not_null())
        }
        ModlogActionType::ModFeaturePost => {
          query.filter(modlog_combined::mod_feature_post_id.is_not_null())
        }
        ModlogActionType::ModRemoveComment => {
          query.filter(modlog_combined::mod_remove_comment_id.is_not_null())
        }
        ModlogActionType::ModRemoveCommunity => {
          query.filter(modlog_combined::mod_remove_community_id.is_not_null())
        }
        ModlogActionType::ModBanFromCommunity => {
          query.filter(modlog_combined::mod_ban_from_community_id.is_not_null())
        }
        ModlogActionType::ModAddCommunity => {
          query.filter(modlog_combined::mod_add_community_id.is_not_null())
        }
        ModlogActionType::ModTransferCommunity => {
          query.filter(modlog_combined::mod_transfer_community_id.is_not_null())
        }
        ModlogActionType::ModAdd => query.filter(modlog_combined::mod_add_id.is_not_null()),
        ModlogActionType::ModBan => query.filter(modlog_combined::mod_ban_id.is_not_null()),
        ModlogActionType::ModHideCommunity => {
          query.filter(modlog_combined::mod_hide_community_id.is_not_null())
        }
        ModlogActionType::AdminPurgePerson => {
          query.filter(modlog_combined::admin_purge_person_id.is_not_null())
        }
        ModlogActionType::AdminPurgeCommunity => {
          query.filter(modlog_combined::admin_purge_community_id.is_not_null())
        }
        ModlogActionType::AdminPurgePost => {
          query.filter(modlog_combined::admin_purge_post_id.is_not_null())
        }
        ModlogActionType::AdminPurgeComment => {
          query.filter(modlog_combined::admin_purge_comment_id.is_not_null())
        }
        ModlogActionType::AdminBlockInstance => {
          query.filter(modlog_combined::admin_block_instance_id.is_not_null())
        }
        ModlogActionType::AdminAllowInstance => {
          query.filter(modlog_combined::admin_allow_instance_id.is_not_null())
        }
      }
    }

    let mut query = PaginatedQueryBuilder::new(query);

    let page_after = self.page_after.map(|c| c.0);

    if self.page_back.unwrap_or_default() {
      query = query.before(page_after).limit_and_offset_from_end();
    } else {
      query = query.after(page_after);
    }

    // Tie breaker
    query = query.then_desc(key::published).then_desc(key::id);

    let res = query.load::<ModlogCombinedViewInternal>(conn).await?;

    // Map the query results to the enum
    let out = res.into_iter().filter_map(|u| u.map_to_enum()).collect();

    Ok(out)
  }
}

impl InternalToCombinedView for ModlogCombinedViewInternal {
  type CombinedView = ModlogCombinedView;

  fn map_to_enum(&self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self.clone();

    if let (Some(admin_allow_instance), Some(instance)) =
      (v.admin_allow_instance, v.instance.clone())
    {
      Some(ModlogCombinedView::AdminAllowInstance(
        AdminAllowInstanceView {
          admin_allow_instance,
          instance,
          admin: v.moderator,
        },
      ))
    } else if let (Some(admin_block_instance), Some(instance)) =
      (v.admin_block_instance, v.instance)
    {
      Some(ModlogCombinedView::AdminBlockInstance(
        AdminBlockInstanceView {
          admin_block_instance,
          instance,
          admin: v.moderator,
        },
      ))
    } else if let (Some(admin_purge_comment), Some(post)) = (v.admin_purge_comment, v.post.clone())
    {
      Some(ModlogCombinedView::AdminPurgeComment(
        AdminPurgeCommentView {
          admin_purge_comment,
          post,
          admin: v.moderator,
        },
      ))
    } else if let Some(admin_purge_community) = v.admin_purge_community {
      Some(ModlogCombinedView::AdminPurgeCommunity(
        AdminPurgeCommunityView {
          admin_purge_community,
          admin: v.moderator,
        },
      ))
    } else if let Some(admin_purge_person) = v.admin_purge_person {
      Some(ModlogCombinedView::AdminPurgePerson(AdminPurgePersonView {
        admin_purge_person,
        admin: v.moderator,
      }))
    } else if let (Some(admin_purge_post), Some(community)) =
      (v.admin_purge_post, v.community.clone())
    {
      Some(ModlogCombinedView::AdminPurgePost(AdminPurgePostView {
        admin_purge_post,
        admin: v.moderator,
        community,
      }))
    } else if let (Some(mod_add), Some(modded_person)) = (v.mod_add, v.modded_person.clone()) {
      Some(ModlogCombinedView::ModAdd(ModAddView {
        mod_add,
        moderator: v.moderator,
        modded_person,
      }))
    } else if let (Some(mod_add_community), Some(modded_person), Some(community)) = (
      v.mod_add_community,
      v.modded_person.clone(),
      v.community.clone(),
    ) {
      Some(ModlogCombinedView::ModAddCommunity(ModAddCommunityView {
        mod_add_community,
        moderator: v.moderator,
        modded_person,
        community,
      }))
    } else if let (Some(mod_ban), Some(modded_person)) = (v.mod_ban, v.modded_person.clone()) {
      Some(ModlogCombinedView::ModBan(ModBanView {
        mod_ban,
        moderator: v.moderator,
        modded_person,
      }))
    } else if let (Some(mod_ban_from_community), Some(modded_person), Some(community)) = (
      v.mod_ban_from_community,
      v.modded_person.clone(),
      v.community.clone(),
    ) {
      Some(ModlogCombinedView::ModBanFromCommunity(
        ModBanFromCommunityView {
          mod_ban_from_community,
          moderator: v.moderator,
          modded_person,
          community,
        },
      ))
    } else if let (Some(mod_feature_post), Some(modded_person), Some(community), Some(post)) = (
      v.mod_feature_post,
      v.modded_person.clone(),
      v.community.clone(),
      v.post.clone(),
    ) {
      Some(ModlogCombinedView::ModFeaturePost(ModFeaturePostView {
        mod_feature_post,
        moderator: v.moderator,
        modded_person,
        community,
        post,
      }))
    } else if let (Some(mod_hide_community), Some(community)) =
      (v.mod_hide_community, v.community.clone())
    {
      Some(ModlogCombinedView::ModHideCommunity(ModHideCommunityView {
        mod_hide_community,
        admin: v.moderator,
        community,
      }))
    } else if let (Some(mod_lock_post), Some(modded_person), Some(community), Some(post)) = (
      v.mod_lock_post,
      v.modded_person.clone(),
      v.community.clone(),
      v.post.clone(),
    ) {
      Some(ModlogCombinedView::ModLockPost(ModLockPostView {
        mod_lock_post,
        moderator: v.moderator,
        modded_person,
        community,
        post,
      }))
    } else if let (
      Some(mod_remove_comment),
      Some(modded_person),
      Some(community),
      Some(post),
      Some(comment),
    ) = (
      v.mod_remove_comment,
      v.modded_person.clone(),
      v.community.clone(),
      v.post.clone(),
      v.comment,
    ) {
      Some(ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
        mod_remove_comment,
        moderator: v.moderator,
        modded_person,
        community,
        post,
        comment,
      }))
    } else if let (Some(mod_remove_community), Some(community)) =
      (v.mod_remove_community, v.community.clone())
    {
      Some(ModlogCombinedView::ModRemoveCommunity(
        ModRemoveCommunityView {
          mod_remove_community,
          moderator: v.moderator,
          community,
        },
      ))
    } else if let (Some(mod_remove_post), Some(modded_person), Some(community), Some(post)) = (
      v.mod_remove_post,
      v.modded_person.clone(),
      v.community.clone(),
      v.post.clone(),
    ) {
      Some(ModlogCombinedView::ModRemovePost(ModRemovePostView {
        mod_remove_post,
        moderator: v.moderator,
        modded_person,
        community,
        post,
      }))
    } else if let (Some(mod_transfer_community), Some(modded_person), Some(community)) = (
      v.mod_transfer_community,
      v.modded_person.clone(),
      v.community.clone(),
    ) {
      Some(ModlogCombinedView::ModTransferCommunity(
        ModTransferCommunityView {
          mod_transfer_community,
          moderator: v.moderator,
          modded_person,
          community,
        },
      ))
    } else {
      None
    }
  }
}

// TODO add tests, especially for all the filters
