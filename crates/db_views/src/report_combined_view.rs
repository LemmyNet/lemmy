use i_love_jesus::CursorKeysModule;
use diesel::Selectable;
use crate::structs::{
  CommentReportView,
  LocalUserView,
  PostReportView,
  PrivateMessageReportView,
  ReportCombinedPaginationCursor,
  ReportCombinedView,
  ReportCombinedViewInternal,
};
use chrono::{DateTime, Utc};
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
  source::{combined::report::{report_combined_keys as key, ReportCombined}, community::CommunityFollower},
  utils::{actions, actions_alias, functions::coalesce, get_conn, limit_and_offset, DbPool},
};
use lemmy_utils::error::LemmyResult;

impl ReportCombinedViewInternal {
  /// returns the current unresolved report count for the communities you mod
  pub async fn get_report_count(
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
    community_id: Option<CommunityId>,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;

    let conn = &mut get_conn(pool).await?;
    let my_person_id = user.local_user.person_id;

    let mut query = report_combined::table
      .left_join(post_report::table)
      .left_join(comment_report::table)
      .left_join(private_message_report::table)
      // Need to join to comment and post to get the community
      .left_join(comment::table.on(comment_report::comment_id.eq(comment::id)))
      // The post
      .left_join(
        post::table.on(
          post_report::post_id
            .eq(post::id)
            .or(comment::post_id.eq(post::id)),
        ),
      )
      .left_join(community::table.on(post::community_id.eq(community::id)))
      .left_join(actions(
        community_actions::table,
        Some(my_person_id),
        post::community_id,
      ))
      .filter(post_report::resolved.eq(false))
      .or_filter(comment_report::resolved.eq(false))
      .or_filter(private_message_report::resolved.eq(false))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id))
    }

    // If its not an admin, get only the ones you mod
    if !user.local_user.admin {
      query = query.filter(community_actions::became_moderator.is_not_null());
    }

    query
      .select(count(report_combined::id))
      .first::<i64>(conn)
      .await
  }
}

impl ReportCombinedPaginationCursor {
  // get cursor for page that starts immediately after the given post
  pub fn after_post(view: &ReportCombinedView) -> ReportCombinedPaginationCursor {
    // This is the simplest way to encode the cursor without making `report_combined::id` part of the public API
    let published = match view {
      ReportCombinedView::Comment(v) => v.comment_report.published,
      ReportCombinedView::Post(v) => v.post_report.published,
      ReportCombinedView::PrivateMessage(v) => v.private_message_report.published,
    };
    // hex encoding to prevent ossification
    ReportCombinedPaginationCursor(format!("published_{:x}", published.timestamp_micros()))
  }
  pub fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
    let err_msg = || Error::QueryBuilderError("Could not parse pagination token".into());
    let mut query = report_combined::table
      .select(ReportCombined::as_select())
      .into_boxed();
    let (prefix, id_str) = self.0.split_at_checked(1).ok_or_else(err_msg)?;
    let id = i32::from_str_radix(id_str, 16).map_err(|_| err_msg())?;
    query = match prefix {
      "C" => query.filter(report_combined::comment_report_id.eq(id)),
      "P" => query.filter(report_combined::post_report_id.eq(id)),
      "M" => query.filter(report_combined::private_message_report_id.eq(id)),
      _ => return Err(err_msg()),
    };
    let token = query.first(&mut get_conn(pool).await?).await?;

    Ok(PaginationCursorData(token))
  }
}

// TODO: when the CursorKeysModule macro allows specifying the table name without a diesel macro, remove the Selectable macro
#[derive(Clone, Selectable, CursorKeysModule)]
#[diesel(table_name = report_combined)]
#[cursor_keys_module(name = key)]
pub struct PaginationCursorData {
  published: DateTime<Utc>,
}

#[derive(Default)]
pub struct ReportCombinedQuery {
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unresolved_only: bool,
  pub page_after: Option<PaginationCursorData>,
  pub page_back: bool,
}

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

    let mut query = PaginatedQueryBuilder::new(query);

    let page_after = options.page_after.map(|c| c.0);

    if options.page_back {
      query = query.before(page_after).limit_and_offset_from_end();
    } else {
      query = query.after(page_after);
    }

    // If viewing all reports, order by newest, but if viewing unresolved only, show the oldest
    // first (FIFO)
    if options.unresolved_only {
      query = query
        .filter(post_report::resolved.eq(false)
        .or(comment_report::resolved.eq(false))
        .or(private_message_report::resolved.eq(false)))
        .then_desc(report_combined::published);
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

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::{
    report_combined_view::ReportCombinedQuery,
    structs::{LocalUserView, ReportCombinedView, ReportCombinedViewInternal},
  };
  use lemmy_db_schema::{
    assert_length,
    source::{
      comment::{Comment, CommentInsertForm},
      comment_report::{CommentReport, CommentReportForm},
      community::{Community, CommunityInsertForm, CommunityModerator, CommunityModeratorForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      local_user_vote_display_mode::LocalUserVoteDisplayMode,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
      post_report::{PostReport, PostReportForm},
      private_message::{PrivateMessage, PrivateMessageInsertForm},
      private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
    },
    traits::{Crud, Joinable, Reportable},
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy_form = PersonInsertForm::test_form(inserted_instance.id, "timmy_rcv");
    let inserted_timmy = Person::create(pool, &timmy_form).await?;
    let timmy_local_user_form = LocalUserInsertForm::test_form(inserted_timmy.id);
    let timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;
    let timmy_view = LocalUserView {
      local_user: timmy_local_user,
      local_user_vote_display_mode: LocalUserVoteDisplayMode::default(),
      person: inserted_timmy.clone(),
      counts: Default::default(),
    };

    // Make an admin, to be able to see private message reports.
    let admin_form = PersonInsertForm::test_form(inserted_instance.id, "admin_rcv");
    let inserted_admin = Person::create(pool, &admin_form).await?;
    let admin_local_user_form = LocalUserInsertForm::test_form_admin(inserted_admin.id);
    let admin_local_user = LocalUser::create(pool, &admin_local_user_form, vec![]).await?;
    let admin_view = LocalUserView {
      local_user: admin_local_user,
      local_user_vote_display_mode: LocalUserVoteDisplayMode::default(),
      person: inserted_admin.clone(),
      counts: Default::default(),
    };

    let sara_form = PersonInsertForm::test_form(inserted_instance.id, "sara_rcv");
    let inserted_sara = Person::create(pool, &sara_form).await?;

    let community_form = CommunityInsertForm::new(
      inserted_instance.id,
      "test community crv".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &community_form).await?;

    // Make timmy a mod
    let timmy_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: inserted_timmy.id,
    };
    CommunityModerator::join(pool, &timmy_moderator_form).await?;

    let post_form = PostInsertForm::new(
      "A test post crv".into(),
      inserted_timmy.id,
      inserted_community.id,
    );
    let inserted_post = Post::create(pool, &post_form).await?;

    // sara reports the post
    let sara_report_post_form = PostReportForm {
      creator_id: inserted_sara.id,
      post_id: inserted_post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from sara".into(),
    };
    let inserted_post_report = PostReport::report(pool, &sara_report_post_form).await?;

    // Timmy creates a comment
    let comment_form = CommentInsertForm::new(
      inserted_timmy.id,
      inserted_post.id,
      "A test comment rv".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    // Sara reports the comment
    let sara_report_comment_form = CommentReportForm {
      creator_id: inserted_sara.id,
      comment_id: inserted_comment.id,
      original_comment_text: "A test comment rv".into(),
      reason: "from sara".into(),
    };
    CommentReport::report(pool, &sara_report_comment_form).await?;

    // Timmy creates a private message report
    let pm_form = PrivateMessageInsertForm::new(
      inserted_timmy.id,
      inserted_sara.id,
      "something offensive crv".to_string(),
    );
    let inserted_pm = PrivateMessage::create(pool, &pm_form).await?;

    // sara reports private message
    let pm_report_form = PrivateMessageReportForm {
      creator_id: inserted_sara.id,
      original_pm_text: inserted_pm.content.clone(),
      private_message_id: inserted_pm.id,
      reason: "its offensive".to_string(),
    };
    PrivateMessageReport::report(pool, &pm_report_form).await?;

    // Do a batch read of admins reports
    let reports = ReportCombinedQuery::default()
      .list(pool, &admin_view)
      .await?;
    assert_eq!(3, reports.len());

    // Make sure the report types are correct
    if let ReportCombinedView::Post(v) = &reports[2] {
      assert_eq!(inserted_post.id, v.post.id);
      assert_eq!(inserted_sara.id, v.creator.id);
      assert_eq!(inserted_timmy.id, v.post_creator.id);
    } else {
      panic!("wrong type");
    }
    if let ReportCombinedView::Comment(v) = &reports[1] {
      assert_eq!(inserted_comment.id, v.comment.id);
      assert_eq!(inserted_post.id, v.post.id);
      assert_eq!(inserted_timmy.id, v.comment_creator.id);
    } else {
      panic!("wrong type");
    }
    if let ReportCombinedView::PrivateMessage(v) = &reports[0] {
      assert_eq!(inserted_pm.id, v.private_message.id);
    } else {
      panic!("wrong type");
    }

    let report_count_admin =
      ReportCombinedViewInternal::get_report_count(pool, &admin_view, None).await?;
    assert_eq!(3, report_count_admin);

    // Timmy should only see 2 reports, since they're not an admin,
    // but they do mod the community
    let reports = ReportCombinedQuery::default()
      .list(pool, &timmy_view)
      .await?;
    assert_eq!(2, reports.len());

    // Make sure the report types are correct
    if let ReportCombinedView::Post(v) = &reports[1] {
      assert_eq!(inserted_post.id, v.post.id);
      assert_eq!(inserted_sara.id, v.creator.id);
      assert_eq!(inserted_timmy.id, v.post_creator.id);
    } else {
      panic!("wrong type");
    }
    if let ReportCombinedView::Comment(v) = &reports[0] {
      assert_eq!(inserted_comment.id, v.comment.id);
      assert_eq!(inserted_post.id, v.post.id);
      assert_eq!(inserted_timmy.id, v.comment_creator.id);
    } else {
      panic!("wrong type");
    }

    let report_count_timmy =
      ReportCombinedViewInternal::get_report_count(pool, &timmy_view, None).await?;
    assert_eq!(2, report_count_timmy);

    // Resolve the post report
    PostReport::resolve(pool, inserted_post_report.id, inserted_timmy.id).await?;

    // Do a batch read of timmys reports
    // It should only show saras, which is unresolved
    let reports_after_resolve = ReportCombinedQuery {
      unresolved_only: true,
      ..Default::default()
    }
    .list(pool, &timmy_view)
    .await?;
    assert_length!(1, reports_after_resolve);

    // Make sure the counts are correct
    let report_count_after_resolved =
      ReportCombinedViewInternal::get_report_count(pool, &timmy_view, None).await?;
    assert_eq!(1, report_count_after_resolved);

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
