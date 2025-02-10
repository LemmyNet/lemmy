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
  impls::local_user::LocalUserOptionHelper,
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
    community_actions,
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
  source::{
    combined::modlog::{modlog_combined_keys as key, ModlogCombined},
    local_user::LocalUser,
  },
  traits::InternalToCombinedView,
  utils::{get_conn, DbPool},
  ListingType,
  ModlogActionType,
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
            .or(mod_hide_community::mod_person_id.eq(person::id))
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
      .left_join(mod_hide_community::table)
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
    ModlogCombinedPaginationCursor(format!("{prefix}-{id:x}"))
  }

  pub async fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
    let err_msg = || Error::QueryBuilderError("Could not parse pagination token".into());
    let mut query = modlog_combined::table
      .select(ModlogCombined::as_select())
      .into_boxed();
    let (prefix, id_str) = self.0.split_once('-').ok_or_else(err_msg)?;
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
  pub page_after: Option<PaginationCursorData>,
  pub page_back: Option<bool>,
}

impl ModlogCombinedQuery<'_> {
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<Vec<ModlogCombinedView>> {
    let conn = &mut get_conn(pool).await?;
    let other_person = aliases::person1.field(person::id);
    let my_person_id = self.local_user.person_id();

    let mut query =
      ModlogCombinedViewInternal::joins(self.mod_person_id, self.hide_modlog_names, my_person_id)
        .select(ModlogCombinedViewInternal::as_select())
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
        ModHideCommunity => query.filter(modlog_combined::mod_hide_community_id.is_not_null()),
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

    let is_subscribed = community_actions::followed.is_not_null();
    query = match self.listing_type.unwrap_or(ListingType::All) {
      ListingType::All => query,
      ListingType::Subscribed => query.filter(is_subscribed),
      ListingType::Local => query
        .filter(community::local.eq(true))
        .filter(community::hidden.eq(false).or(is_subscribed)),
      ListingType::ModeratorView => query.filter(community_actions::became_moderator.is_not_null()),
    };

    let mut query = PaginatedQueryBuilder::new(query);

    let page_after = self.page_after.map(|c| c.0);

    if self.page_back.unwrap_or_default() {
      query = query.before(page_after).limit_and_offset_from_end();
    } else {
      query = query.after(page_after);
    }

    query = query
      .then_desc(key::published)
      // Tie breaker
      .then_desc(key::id);

    let res = query.load::<ModlogCombinedViewInternal>(conn).await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

impl InternalToCombinedView for ModlogCombinedViewInternal {
  type CombinedView = ModlogCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

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
    } else if let (Some(mod_add), Some(other_person)) = (v.mod_add, v.other_person.clone()) {
      Some(ModlogCombinedView::ModAdd(ModAddView {
        mod_add,
        moderator: v.moderator,
        other_person,
      }))
    } else if let (Some(mod_add_community), Some(other_person), Some(community)) = (
      v.mod_add_community,
      v.other_person.clone(),
      v.community.clone(),
    ) {
      Some(ModlogCombinedView::ModAddCommunity(ModAddCommunityView {
        mod_add_community,
        moderator: v.moderator,
        other_person,
        community,
      }))
    } else if let (Some(mod_ban), Some(other_person)) = (v.mod_ban, v.other_person.clone()) {
      Some(ModlogCombinedView::ModBan(ModBanView {
        mod_ban,
        moderator: v.moderator,
        other_person,
      }))
    } else if let (Some(mod_ban_from_community), Some(other_person), Some(community)) = (
      v.mod_ban_from_community,
      v.other_person.clone(),
      v.community.clone(),
    ) {
      Some(ModlogCombinedView::ModBanFromCommunity(
        ModBanFromCommunityView {
          mod_ban_from_community,
          moderator: v.moderator,
          other_person,
          community,
        },
      ))
    } else if let (Some(mod_feature_post), Some(other_person), Some(community), Some(post)) = (
      v.mod_feature_post,
      v.other_person.clone(),
      v.community.clone(),
      v.post.clone(),
    ) {
      Some(ModlogCombinedView::ModFeaturePost(ModFeaturePostView {
        mod_feature_post,
        moderator: v.moderator,
        other_person,
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
    } else if let (Some(mod_lock_post), Some(other_person), Some(community), Some(post)) = (
      v.mod_lock_post,
      v.other_person.clone(),
      v.community.clone(),
      v.post.clone(),
    ) {
      Some(ModlogCombinedView::ModLockPost(ModLockPostView {
        mod_lock_post,
        moderator: v.moderator,
        other_person,
        community,
        post,
      }))
    } else if let (
      Some(mod_remove_comment),
      Some(other_person),
      Some(community),
      Some(post),
      Some(comment),
    ) = (
      v.mod_remove_comment,
      v.other_person.clone(),
      v.community.clone(),
      v.post.clone(),
      v.comment,
    ) {
      Some(ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
        mod_remove_comment,
        moderator: v.moderator,
        other_person,
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
    } else if let (Some(mod_remove_post), Some(other_person), Some(community), Some(post)) = (
      v.mod_remove_post,
      v.other_person.clone(),
      v.community.clone(),
      v.post.clone(),
    ) {
      Some(ModlogCombinedView::ModRemovePost(ModRemovePostView {
        mod_remove_post,
        moderator: v.moderator,
        other_person,
        community,
        post,
      }))
    } else if let (Some(mod_transfer_community), Some(other_person), Some(community)) = (
      v.mod_transfer_community,
      v.other_person.clone(),
      v.community.clone(),
    ) {
      Some(ModlogCombinedView::ModTransferCommunity(
        ModTransferCommunityView {
          mod_transfer_community,
          moderator: v.moderator,
          other_person,
          community,
        },
      ))
    } else {
      None
    }
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::{combined::modlog_combined_view::ModlogCombinedQuery, structs::ModlogCombinedView};
  use lemmy_db_schema::{
    newtypes::PersonId,
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      mod_log::{
        admin::{
          AdminAllowInstance,
          AdminAllowInstanceForm,
          AdminBlockInstance,
          AdminBlockInstanceForm,
          AdminPurgeComment,
          AdminPurgeCommentForm,
          AdminPurgeCommunity,
          AdminPurgeCommunityForm,
          AdminPurgePerson,
          AdminPurgePersonForm,
          AdminPurgePost,
          AdminPurgePostForm,
        },
        moderator::{
          ModAdd,
          ModAddCommunity,
          ModAddCommunityForm,
          ModAddForm,
          ModBan,
          ModBanForm,
          ModBanFromCommunity,
          ModBanFromCommunityForm,
          ModFeaturePost,
          ModFeaturePostForm,
          ModHideCommunity,
          ModHideCommunityForm,
          ModLockPost,
          ModLockPostForm,
          ModRemoveComment,
          ModRemoveCommentForm,
          ModRemoveCommunity,
          ModRemoveCommunityForm,
          ModRemovePost,
          ModRemovePostForm,
          ModTransferCommunity,
          ModTransferCommunityForm,
        },
      },
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::Crud,
    utils::{build_db_pool_for_tests, DbPool},
    ModlogActionType,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    instance: Instance,
    timmy: Person,
    sara: Person,
    jessica: Person,
    community: Community,
    community_2: Community,
    post: Post,
    post_2: Post,
    comment: Comment,
    comment_2: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy_form = PersonInsertForm::test_form(instance.id, "timmy_rcv");
    let timmy = Person::create(pool, &timmy_form).await?;

    let sara_form = PersonInsertForm::test_form(instance.id, "sara_rcv");
    let sara = Person::create(pool, &sara_form).await?;

    let jessica_form = PersonInsertForm::test_form(instance.id, "jessica_mrv");
    let jessica = Person::create(pool, &jessica_form).await?;

    let community_form = CommunityInsertForm::new(
      instance.id,
      "test community crv".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;

    let community_form_2 = CommunityInsertForm::new(
      instance.id,
      "test community crv 2".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community_2 = Community::create(pool, &community_form_2).await?;

    let post_form = PostInsertForm::new("A test post crv".into(), timmy.id, community.id);
    let post = Post::create(pool, &post_form).await?;

    let new_post_2 = PostInsertForm::new("A test post crv 2".into(), sara.id, community_2.id);
    let post_2 = Post::create(pool, &new_post_2).await?;

    // Timmy creates a comment
    let comment_form = CommentInsertForm::new(timmy.id, post.id, "A test comment rv".into());
    let comment = Comment::create(pool, &comment_form, None).await?;

    // jessica creates a comment
    let comment_form_2 =
      CommentInsertForm::new(jessica.id, post_2.id, "A test comment rv 2".into());
    let comment_2 = Comment::create(pool, &comment_form_2, None).await?;

    Ok(Data {
      instance,
      timmy,
      sara,
      jessica,
      community,
      community_2,
      post,
      post_2,
      comment,
      comment_2,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn admin_types() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = AdminAllowInstanceForm {
      instance_id: data.instance.id,
      admin_person_id: data.timmy.id,
      allowed: true,
      reason: None,
    };
    AdminAllowInstance::create(pool, &form).await?;

    let form = AdminBlockInstanceForm {
      instance_id: data.instance.id,
      admin_person_id: data.timmy.id,
      blocked: true,
      reason: None,
    };
    AdminBlockInstance::create(pool, &form).await?;

    let form = AdminPurgeCommentForm {
      admin_person_id: data.timmy.id,
      post_id: data.post.id,
      reason: None,
    };
    AdminPurgeComment::create(pool, &form).await?;

    let form = AdminPurgeCommunityForm {
      admin_person_id: data.timmy.id,
      reason: None,
    };
    AdminPurgeCommunity::create(pool, &form).await?;

    let form = AdminPurgePersonForm {
      admin_person_id: data.timmy.id,
      reason: None,
    };
    AdminPurgePerson::create(pool, &form).await?;

    let form = AdminPurgePostForm {
      admin_person_id: data.timmy.id,
      community_id: data.community.id,
      reason: None,
    };
    AdminPurgePost::create(pool, &form).await?;

    let form = ModHideCommunityForm {
      mod_person_id: data.timmy.id,
      community_id: data.community.id,
      hidden: Some(true),
      reason: None,
    };
    ModHideCommunity::create(pool, &form).await?;

    // A 2nd mod hide community, but to a different community, and with jessica
    let form = ModHideCommunityForm {
      mod_person_id: data.jessica.id,
      community_id: data.community_2.id,
      hidden: Some(true),
      reason: None,
    };
    ModHideCommunity::create(pool, &form).await?;

    let modlog = ModlogCombinedQuery::default().list(pool).await?;
    assert_eq!(8, modlog.len());

    if let ModlogCombinedView::ModHideCommunity(v) = &modlog[0] {
      assert_eq!(data.community_2.id, v.mod_hide_community.community_id);
      assert_eq!(data.community_2.id, v.community.id);
      assert_eq!(
        data.jessica.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModHideCommunity(v) = &modlog[1] {
      assert_eq!(data.community.id, v.mod_hide_community.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::AdminPurgePost(v) = &modlog[2] {
      assert_eq!(data.community.id, v.admin_purge_post.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::AdminPurgePerson(v) = &modlog[3] {
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::AdminPurgeCommunity(v) = &modlog[4] {
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::AdminPurgeComment(v) = &modlog[5] {
      assert_eq!(data.post.id, v.admin_purge_comment.post_id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    // Make sure the report types are correct
    if let ModlogCombinedView::AdminBlockInstance(v) = &modlog[6] {
      assert_eq!(data.instance.id, v.admin_block_instance.instance_id);
      assert_eq!(data.instance.id, v.instance.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::AdminAllowInstance(v) = &modlog[7] {
      assert_eq!(data.instance.id, v.admin_allow_instance.instance_id);
      assert_eq!(data.instance.id, v.instance.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    // Filter by admin
    let modlog_admin_filter = ModlogCombinedQuery {
      mod_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    // Only one is jessica
    assert_eq!(7, modlog_admin_filter.len());

    // Filter by community
    let modlog_community_filter = ModlogCombinedQuery {
      community_id: Some(data.community.id),
      ..Default::default()
    }
    .list(pool)
    .await?;

    // Should be 2, and not jessicas
    assert_eq!(2, modlog_community_filter.len());

    // Filter by type
    let modlog_type_filter = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModHideCommunity),
      ..Default::default()
    }
    .list(pool)
    .await?;

    // 2 of these, one is jessicas
    assert_eq!(2, modlog_type_filter.len());

    if let ModlogCombinedView::ModHideCommunity(v) = &modlog_type_filter[0] {
      assert_eq!(data.community_2.id, v.mod_hide_community.community_id);
      assert_eq!(data.community_2.id, v.community.id);
      assert_eq!(
        data.jessica.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModHideCommunity(v) = &modlog_type_filter[1] {
      assert_eq!(data.community.id, v.mod_hide_community.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn mod_types() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = ModAddForm {
      mod_person_id: data.timmy.id,
      other_person_id: data.jessica.id,
      removed: Some(false),
    };
    ModAdd::create(pool, &form).await?;

    let form = ModAddCommunityForm {
      mod_person_id: data.timmy.id,
      other_person_id: data.jessica.id,
      community_id: data.community.id,
      removed: Some(false),
    };
    ModAddCommunity::create(pool, &form).await?;

    let form = ModBanForm {
      mod_person_id: data.timmy.id,
      other_person_id: data.jessica.id,
      banned: Some(true),
      reason: None,
      expires: None,
    };
    ModBan::create(pool, &form).await?;

    let form = ModBanFromCommunityForm {
      mod_person_id: data.timmy.id,
      other_person_id: data.jessica.id,
      community_id: data.community.id,
      banned: Some(true),
      reason: None,
      expires: None,
    };
    ModBanFromCommunity::create(pool, &form).await?;

    let form = ModFeaturePostForm {
      mod_person_id: data.timmy.id,
      post_id: data.post.id,
      featured: Some(true),
      is_featured_community: None,
    };
    ModFeaturePost::create(pool, &form).await?;

    let form = ModLockPostForm {
      mod_person_id: data.timmy.id,
      post_id: data.post.id,
      locked: Some(true),
    };
    ModLockPost::create(pool, &form).await?;

    let form = ModRemoveCommentForm {
      mod_person_id: data.timmy.id,
      comment_id: data.comment.id,
      removed: Some(true),
      reason: None,
    };
    ModRemoveComment::create(pool, &form).await?;

    let form = ModRemoveCommunityForm {
      mod_person_id: data.timmy.id,
      community_id: data.community.id,
      removed: Some(true),
      reason: None,
    };
    ModRemoveCommunity::create(pool, &form).await?;

    let form = ModRemovePostForm {
      mod_person_id: data.timmy.id,
      post_id: data.post.id,
      removed: Some(true),
      reason: None,
    };
    ModRemovePost::create(pool, &form).await?;

    let form = ModTransferCommunityForm {
      mod_person_id: data.timmy.id,
      other_person_id: data.jessica.id,
      community_id: data.community.id,
    };
    ModTransferCommunity::create(pool, &form).await?;

    // A few extra ones to test different filters
    let form = ModTransferCommunityForm {
      mod_person_id: data.jessica.id,
      other_person_id: data.sara.id,
      community_id: data.community_2.id,
    };
    ModTransferCommunity::create(pool, &form).await?;

    let form = ModRemovePostForm {
      mod_person_id: data.jessica.id,
      post_id: data.post_2.id,
      removed: Some(true),
      reason: None,
    };
    ModRemovePost::create(pool, &form).await?;

    let form = ModRemoveCommentForm {
      mod_person_id: data.jessica.id,
      comment_id: data.comment_2.id,
      removed: Some(true),
      reason: None,
    };
    ModRemoveComment::create(pool, &form).await?;

    // The all view
    let modlog = ModlogCombinedQuery::default().list(pool).await?;
    assert_eq!(13, modlog.len());

    if let ModlogCombinedView::ModRemoveComment(v) = &modlog[0] {
      assert_eq!(data.comment_2.id, v.mod_remove_comment.comment_id);
      assert_eq!(data.comment_2.id, v.comment.id);
      assert_eq!(data.post_2.id, v.post.id);
      assert_eq!(data.community_2.id, v.community.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModRemovePost(v) = &modlog[1] {
      assert_eq!(data.post_2.id, v.mod_remove_post.post_id);
      assert_eq!(data.post_2.id, v.post.id);
      assert_eq!(data.sara.id, v.post.creator_id);
      assert_eq!(data.community_2.id, v.community.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.sara.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModTransferCommunity(v) = &modlog[2] {
      assert_eq!(data.community_2.id, v.mod_transfer_community.community_id);
      assert_eq!(data.community_2.id, v.community.id);
      assert_eq!(
        data.jessica.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.sara.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModTransferCommunity(v) = &modlog[3] {
      assert_eq!(data.community.id, v.mod_transfer_community.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModRemovePost(v) = &modlog[4] {
      assert_eq!(data.post.id, v.mod_remove_post.post_id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModRemoveCommunity(v) = &modlog[5] {
      assert_eq!(data.community.id, v.mod_remove_community.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModRemoveComment(v) = &modlog[6] {
      assert_eq!(data.comment.id, v.mod_remove_comment.comment_id);
      assert_eq!(data.comment.id, v.comment.id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModLockPost(v) = &modlog[7] {
      assert_eq!(data.post.id, v.mod_lock_post.post_id);
      assert!(v.mod_lock_post.locked);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModFeaturePost(v) = &modlog[8] {
      assert_eq!(data.post.id, v.mod_feature_post.post_id);
      assert!(v.mod_feature_post.featured);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.post.creator_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.timmy.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModBanFromCommunity(v) = &modlog[9] {
      assert_eq!(data.community.id, v.mod_ban_from_community.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModBan(v) = &modlog[10] {
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModAddCommunity(v) = &modlog[11] {
      assert_eq!(data.community.id, v.mod_add_community.community_id);
      assert_eq!(data.community.id, v.community.id);
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    if let ModlogCombinedView::ModAdd(v) = &modlog[12] {
      assert_eq!(
        data.timmy.id,
        v.moderator.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
      assert_eq!(data.jessica.id, v.other_person.id);
    } else {
      panic!("wrong type");
    }

    // Filter by moderator
    let modlog_mod_timmy_filter = ModlogCombinedQuery {
      mod_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(10, modlog_mod_timmy_filter.len());

    let modlog_mod_jessica_filter = ModlogCombinedQuery {
      mod_person_id: Some(data.jessica.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(3, modlog_mod_jessica_filter.len());

    // Filter by other_person
    // Gets a little complicated because things aren't directly linked,
    // you have to go into the item to see who created it.

    let modlog_modded_timmy_filter = ModlogCombinedQuery {
      other_person_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(4, modlog_modded_timmy_filter.len());

    let modlog_modded_jessica_filter = ModlogCombinedQuery {
      other_person_id: Some(data.jessica.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(6, modlog_modded_jessica_filter.len());

    let modlog_modded_sara_filter = ModlogCombinedQuery {
      other_person_id: Some(data.sara.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_modded_sara_filter.len());

    // Filter by community
    let modlog_community_filter = ModlogCombinedQuery {
      community_id: Some(data.community.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(8, modlog_community_filter.len());

    let modlog_community_2_filter = ModlogCombinedQuery {
      community_id: Some(data.community_2.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(3, modlog_community_2_filter.len());

    // Filter by post
    let modlog_post_filter = ModlogCombinedQuery {
      post_id: Some(data.post.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(4, modlog_post_filter.len());

    let modlog_post_2_filter = ModlogCombinedQuery {
      post_id: Some(data.post_2.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_post_2_filter.len());

    // Filter by comment
    let modlog_comment_filter = ModlogCombinedQuery {
      comment_id: Some(data.comment.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(1, modlog_comment_filter.len());

    let modlog_comment_2_filter = ModlogCombinedQuery {
      comment_id: Some(data.comment_2.id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(1, modlog_comment_2_filter.len());

    // Filter by type
    let modlog_type_filter = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModRemoveComment),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, modlog_type_filter.len());

    // Assert that the types are correct
    assert!(matches!(
      modlog_type_filter[0],
      ModlogCombinedView::ModRemoveComment(_)
    ));
    assert!(matches!(
      modlog_type_filter[1],
      ModlogCombinedView::ModRemoveComment(_)
    ));

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn hide_modlog_names() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = AdminAllowInstanceForm {
      instance_id: data.instance.id,
      admin_person_id: data.timmy.id,
      allowed: true,
      reason: None,
    };
    AdminAllowInstance::create(pool, &form).await?;

    let modlog = ModlogCombinedQuery::default().list(pool).await?;
    assert_eq!(1, modlog.len());

    if let ModlogCombinedView::AdminAllowInstance(v) = &modlog[0] {
      assert_eq!(
        data.timmy.id,
        v.admin.as_ref().map(|a| a.id).unwrap_or(PersonId(-1))
      );
    } else {
      panic!("wrong type");
    }

    // Filter out the names
    let modlog_hide_names_filter = ModlogCombinedQuery {
      hide_modlog_names: Some(true),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(1, modlog_hide_names_filter.len());

    if let ModlogCombinedView::AdminAllowInstance(v) = &modlog_hide_names_filter[0] {
      assert!(v.admin.is_none())
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }
}
