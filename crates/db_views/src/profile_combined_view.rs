use crate::structs::{
  CommentView,
  LocalUserView,
  PostView,
  ProfileCombinedPaginationCursor,
  ProfileCombinedView,
  ProfileCombinedViewInternal,
};
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::PaginatedQueryBuilder;
use lemmy_db_schema::{
  aliases::{self, creator_community_actions},
  newtypes::CommunityId,
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    community,
    community_actions,
    local_user,
    person,
    person_actions,
    post,
    post_actions,
    post_aggregates,
    profile_combined,
  },
  source::{
    combined::profile::{profile_combined_keys as key, ProfileCombined},
    community::CommunityFollower,
  },
  utils::{actions, actions_alias, functions::coalesce, get_conn, DbPool, ReverseTimestampKey},
};
use lemmy_utils::error::LemmyResult;

impl ProfileCombinedPaginationCursor {
  // get cursor for page that starts immediately after the given post
  pub fn after_post(view: &ProfileCombinedView) -> ProfileCombinedPaginationCursor {
    let (prefix, id) = match view {
      ProfileCombinedView::Comment(v) => ('C', v.comment.id.0),
      ProfileCombinedView::Post(v) => ('P', v.post.id.0),
    };
    // hex encoding to prevent ossification
    ProfileCombinedPaginationCursor(format!("{prefix}{id:x}"))
  }

  pub async fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
    let err_msg = || Error::QueryBuilderError("Could not parse pagination token".into());
    let mut query = profile_combined::table
      .select(ProfileCombined::as_select())
      .into_boxed();
    let (prefix, id_str) = self.0.split_at_checked(1).ok_or_else(err_msg)?;
    let id = i32::from_str_radix(id_str, 16).map_err(|_err| err_msg())?;
    query = match prefix {
      "C" => query.filter(profile_combined::comment_id.eq(id)),
      "P" => query.filter(profile_combined::post_id.eq(id)),
      _ => return Err(err_msg()),
    };
    let token = query.first(&mut get_conn(pool).await?).await?;

    Ok(PaginationCursorData(token))
  }
}

#[derive(Clone)]
pub struct PaginationCursorData(ProfileCombined);

// TODO check these
#[derive(Default)]
pub struct ProfileCombinedQuery {
  pub community_id: Option<CommunityId>,
  pub unresolved_only: Option<bool>,
  pub page_after: Option<PaginationCursorData>,
  pub page_back: Option<bool>,
}

impl ProfileCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> LemmyResult<Vec<ProfileCombinedView>> {
    let my_person_id = user.local_user.person_id;
    // let item_creator = aliases::person1.field(person::id);
    let item_creator = person::id;

    let conn = &mut get_conn(pool).await?;

    // Notes: since the post_id and comment_id are optional columns,
    // many joins must use an OR condition.
    // For example, the creator must be the person table joined to either:
    // - post.creator_id
    // - comment.creator_id
    let mut query = profile_combined::table
      // The comment
      .left_join(comment::table.on(profile_combined::comment_id.eq(comment::id.nullable())))
      // The post
      .inner_join(
        post::table.on(
          profile_combined::post_id
            .eq(post::id.nullable())
            .or(comment::post_id.nullable().eq(profile_combined::post_id)),
        ),
      )
      // The item creator
      .inner_join(
        person::table.on(
          post::creator_id
            .eq(person::id)
            .or(comment::creator_id.eq(person::id)),
        ),
      )
      // The item creator
      // You can now use aliases::person1.field(person::id) / item_creator for all the item actions
      // .inner_join(
      //   aliases::person1.on(
      //     post::creator_id
      //       .eq(item_creator)
      //       .or(comment::creator_id.eq(item_creator)),
      //   ),
      // )
      // The community
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .left_join(actions_alias(
        creator_community_actions,
        item_creator,
        post::community_id,
      ))
      .left_join(
        local_user::table.on(
          item_creator
            .eq(local_user::person_id)
            .and(local_user::admin.eq(true)),
        ),
      )
      .left_join(actions(
        community_actions::table,
        Some(my_person_id),
        post::community_id,
      ))
      .left_join(actions(post_actions::table, Some(my_person_id), post::id))
      .left_join(actions(
        person_actions::table,
        Some(my_person_id),
        item_creator,
      ))
      .left_join(
        post_aggregates::table
          .on(profile_combined::post_id.eq(post_aggregates::post_id.nullable())),
      )
      .left_join(
        comment_aggregates::table
          .on(profile_combined::comment_id.eq(comment_aggregates::comment_id.nullable())),
      )
      .left_join(actions(
        comment_actions::table,
        Some(my_person_id),
        comment::id,
      ))
      .select((
        // Post-specific
        post::all_columns.nullable(),
        // post_aggregates::all_columns.nullable(),
        // coalesce(
        //   post_aggregates::comments.nullable() - post_actions::read_comments_amount.nullable(),
        //   post_aggregates::comments,
        // )
        // .nullable(),
        // post_actions::saved.nullable().is_not_null(),
        // post_actions::read.nullable().is_not_null(),
        // post_actions::hidden.nullable().is_not_null(),
        // post_actions::like_score.nullable(),
        // // Comment-specific
        // comment::all_columns.nullable(),
        // comment_aggregates::all_columns.nullable(),
        // comment_actions::saved.nullable().is_not_null(),
        // comment_actions::like_score.nullable(),
        // // Private-message-specific
        // private_message_profile::all_columns.nullable(),
        // private_message::all_columns.nullable(),
        // // Shared
        // person::all_columns,
        // aliases::person1.fields(person::all_columns),
        // community::all_columns.nullable(),
        // CommunityFollower::select_subscribed_type(),
        // aliases::person2.fields(person::all_columns.nullable()),
        // local_user::admin.nullable().is_not_null(),
        // creator_community_actions
        //   .field(community_actions::received_ban)
        //   .nullable()
        //   .is_not_null(),
        // creator_community_actions
        //   .field(community_actions::became_moderator)
        //   .nullable()
        //   .is_not_null(),
        // person_actions::blocked.nullable().is_not_null(),
      ))
      .into_boxed();

    if let Some(community_id) = self.community_id {
      query = query.filter(community::id.eq(community_id));
    }

    // If its not an admin, get only the ones you mod
    if !user.local_user.admin {
      query = query.filter(community_actions::became_moderator.is_not_null());
    }

    let mut query = PaginatedQueryBuilder::new(query);

    let page_after = self.page_after.map(|c| c.0);

    if self.page_back.unwrap_or_default() {
      query = query.before(page_after).limit_and_offset_from_end();
    } else {
      query = query.after(page_after);
    }

    // If viewing all profiles, order by newest, but if viewing unresolved only, show the oldest
    // first (FIFO)
    if self.unresolved_only.unwrap_or_default() {
      query = query
        .filter(
          post_profile::resolved
            .eq(false)
            .or(comment_profile::resolved.eq(false))
            .or(private_message_profile::resolved.eq(false)),
        )
        // TODO: when a `then_asc` method is added, use it here, make the id sort direction match,
        // and remove the separate index; unless additional columns are added to this sort
        .then_desc(ReverseTimestampKey(key::published));
    } else {
      query = query.then_desc(key::published);
    }

    // Tie breaker
    query = query.then_desc(key::id);

    let res = query.load::<ProfileCombinedViewInternal>(conn).await?;

    // Map the query results to the enum
    let out = res.into_iter().filter_map(map_to_enum).collect();

    Ok(out)
  }
}

/// Maps the combined DB row to an enum
fn map_to_enum(view: ProfileCombinedViewInternal) -> Option<ProfileCombinedView> {
  // Use for a short alias
  let v = view;

  if let (Some(post), Some(community), Some(unread_comments), Some(counts)) =
    (v.post, v.community, v.post_unread_comments, v.post_counts)
  {
    Some(ProfileCombinedView::Post(PostView {
      post,
      community,
      unread_comments,
      counts,
      creator: v.item_creator,
      creator_banned_from_community: v.item_creator_banned_from_community,
      creator_is_moderator: v.item_creator_is_moderator,
      creator_is_admin: v.item_creator_is_admin,
      creator_blocked: v.item_creator_blocked,
      subscribed: v.subscribed,
      saved: v.post_saved,
      read: v.post_read,
      hidden: v.post_hidden,
      my_vote: v.my_post_vote,
    }))
  } else if let (Some(comment), Some(counts), Some(post), Some(community)) = (
    v.comment,
    v.comment_counts,
    v.post.clone(),
    v.community.clone(),
  ) {
    Some(ProfileCombinedView::Comment(CommentView {
      comment,
      counts,
      post,
      community,
      creator: v.item_creator,
      creator_banned_from_community: v.item_creator_banned_from_community,
      creator_is_moderator: v.item_creator_is_moderator,
      creator_is_admin: v.item_creator_is_admin,
      creator_blocked: v.item_creator_blocked,
      subscribed: v.subscribed,
      saved: v.comment_saved,
      my_vote: v.my_comment_vote,
    }))
  } else {
    None
  }
}
