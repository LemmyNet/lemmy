use crate::structs::{
  CommentReportView,
  LocalUserView,
  PostReportView,
  PrivateMessageReportView,
  ReportCombinedView,
  ReportCombinedViewInternal,
};
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases::{self, creator_community_actions},
  newtypes::{CommunityId, PersonId},
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    comment_report,
    community,
    community_actions,
    local_user,
    person,
    person_actions,
    post,
    post_actions,
    post_aggregates,
    post_report,
    private_message,
    private_message_report,
    report_combined,
  },
  source::community::CommunityFollower,
  utils::{actions, actions_alias, functions::coalesce, get_conn, limit_and_offset, DbPool},
};
use lemmy_utils::error::LemmyResult;

// TODO fix
impl ReportCombinedViewInternal {
  /// returns the current unresolved report count for the communities you mod
  pub async fn get_report_count(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    admin: bool,
    community_id: Option<CommunityId>,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;
    let mut query = post_report::table
      .inner_join(post::table)
      .filter(post_report::resolved.eq(false))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id))
    }

    // If its not an admin, get only the ones you mod
    if !admin {
      query
        .inner_join(
          community_actions::table.on(
            community_actions::community_id
              .eq(post::community_id)
              .and(community_actions::person_id.eq(my_person_id))
              .and(community_actions::became_moderator.is_not_null()),
          ),
        )
        .select(count(post_report::id))
        .first::<i64>(conn)
        .await
    } else {
      query
        .select(count(post_report::id))
        .first::<i64>(conn)
        .await
    }
  }
}

#[derive(Default)]
pub struct ReportCombinedQuery {
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unresolved_only: bool,
}

// TODO need to add private message
impl ReportCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> LemmyResult<Vec<ReportCombinedView>> {
    let options = self;
    let my_person_id = user.local_user.person_id;
    let item_creator = aliases::person1.field(person::id);
    let conn = &mut get_conn(pool).await?;

    // Notes: since the post_report_id and comment_report_id are optional columns,
    // many joins must use an OR condition.
    // For example, the report creator must be the person table joined to either:
    // - post_report.creator_id
    // - comment_report.creator_id
    let mut query = report_combined::table
      .left_join(post_report::table)
      .left_join(comment_report::table)
      .left_join(private_message_report::table)
      // The report creator
      .inner_join(
        person::table.on(
          post_report::creator_id
            .eq(person::id)
            .or(comment_report::creator_id.eq(person::id))
            .or(private_message_report::creator_id.eq(person::id)),
        ),
      )
      // The comment
      .left_join(comment::table.on(comment_report::comment_id.eq(comment::id)))
      // The private message
      .left_join(
        private_message::table
          .on(private_message_report::private_message_id.eq(private_message::id)),
      )
      // The post
      .left_join(
        post::table.on(
          post_report::post_id
            .eq(post::id)
            .or(comment::post_id.eq(post::id)),
        ),
      )
      // The item creator
      // You can now use aliases::person1.field(person::id) / item_creator for all the item actions
      .inner_join(
        aliases::person1.on(
          post::creator_id
            .eq(item_creator)
            .or(comment::creator_id.eq(item_creator))
            .or(private_message::creator_id.eq(item_creator)),
        ),
      )
      // The community
      .left_join(community::table.on(post::community_id.eq(community::id)))
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
      .left_join(post_aggregates::table.on(post_report::post_id.eq(post_aggregates::post_id)))
      .left_join(
        comment_aggregates::table.on(comment_report::comment_id.eq(comment_aggregates::comment_id)),
      )
      .left_join(aliases::person2.on(item_creator.eq(aliases::person2.field(person::id))))
      .left_join(actions(
        comment_actions::table,
        Some(my_person_id),
        comment_report::comment_id,
      ))
      .select((
        // Post-specific
        post_report::all_columns.nullable(),
        post::all_columns.nullable(),
        post_aggregates::all_columns.nullable(),
        coalesce(
          post_aggregates::comments.nullable() - post_actions::read_comments_amount.nullable(),
          post_aggregates::comments,
        )
        .nullable(),
        post_actions::saved.nullable().is_not_null(),
        post_actions::read.nullable().is_not_null(),
        post_actions::hidden.nullable().is_not_null(),
        post_actions::like_score.nullable(),
        // Comment-specific
        comment_report::all_columns.nullable(),
        comment::all_columns.nullable(),
        comment_aggregates::all_columns.nullable(),
        comment_actions::saved.nullable().is_not_null(),
        comment_actions::like_score.nullable(),
        // Private-message-specific
        private_message_report::all_columns.nullable(),
        private_message::all_columns.nullable(),
        // Shared
        person::all_columns,
        aliases::person1.fields(person::all_columns),
        community::all_columns.nullable(),
        CommunityFollower::select_subscribed_type(),
        aliases::person2.fields(person::all_columns.nullable()),
        local_user::admin.nullable().is_not_null(),
        creator_community_actions
          .field(community_actions::received_ban)
          .nullable()
          .is_not_null(),
        creator_community_actions
          .field(community_actions::became_moderator)
          .nullable()
          .is_not_null(),
        person_actions::blocked.nullable().is_not_null(),
      ))
      .into_boxed();

    if let Some(community_id) = options.community_id {
      query = query.filter(community::id.eq(community_id));
    }

    // If viewing all reports, order by newest, but if viewing unresolved only, show the oldest
    // first (FIFO)
    if options.unresolved_only {
      query = query
        .filter(post_report::resolved.eq(false))
        .or_filter(comment_report::resolved.eq(false))
        .or_filter(private_message_report::resolved.eq(false))
        .order_by(report_combined::published.asc());
    } else {
      query = query.order_by(report_combined::published.desc());
    }

    // If its not an admin, get only the ones you mod
    if !user.local_user.admin {
      query = query.filter(community_actions::became_moderator.is_not_null());
    }

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query = query.limit(limit).offset(offset);

    let res = query.load::<ReportCombinedViewInternal>(conn).await?;

    // Map the query results to the enum
    let out = res.into_iter().filter_map(map_to_enum).collect();

    Ok(out)
  }
}

/// Maps the combined DB row to an enum
fn map_to_enum(view: ReportCombinedViewInternal) -> Option<ReportCombinedView> {
  // Use for a short alias
  let v = view;

  if let (Some(post_report), Some(post), Some(community), Some(unread_comments), Some(counts)) = (
    v.post_report,
    v.post.clone(),
    v.community.clone(),
    v.post_unread_comments,
    v.post_counts,
  ) {
    Some(ReportCombinedView::Post(PostReportView {
      post_report,
      post,
      community,
      unread_comments,
      counts,
      creator: v.report_creator,
      post_creator: v.item_creator,
      creator_banned_from_community: v.item_creator_banned_from_community,
      creator_is_moderator: v.item_creator_is_moderator,
      creator_is_admin: v.item_creator_is_admin,
      creator_blocked: v.item_creator_blocked,
      subscribed: v.subscribed,
      saved: v.post_saved,
      read: v.post_read,
      hidden: v.post_hidden,
      my_vote: v.my_post_vote,
      resolver: v.resolver,
    }))
  } else if let (Some(comment_report), Some(comment), Some(counts), Some(post), Some(community)) = (
    v.comment_report,
    v.comment,
    v.comment_counts,
    v.post.clone(),
    v.community.clone(),
  ) {
    Some(ReportCombinedView::Comment(CommentReportView {
      comment_report,
      comment,
      counts,
      post,
      community,
      creator: v.report_creator,
      comment_creator: v.item_creator,
      creator_banned_from_community: v.item_creator_banned_from_community,
      creator_is_moderator: v.item_creator_is_moderator,
      creator_is_admin: v.item_creator_is_admin,
      creator_blocked: v.item_creator_blocked,
      subscribed: v.subscribed,
      saved: v.comment_saved,
      my_vote: v.my_comment_vote,
      resolver: v.resolver,
    }))
  } else if let (Some(private_message_report), Some(private_message)) =
    (v.private_message_report, v.private_message)
  {
    Some(ReportCombinedView::PrivateMessage(
      PrivateMessageReportView {
        private_message_report,
        private_message,
        creator: v.report_creator,
        private_message_creator: v.item_creator,
        resolver: v.resolver,
      },
    ))
  } else {
    None
  }
}

// TODO add tests
