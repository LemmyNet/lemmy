use crate::{
  structs::{
    CommentReportView,
    LocalUserView,
    PostReportView,
    PrivateMessageReportView,
    ReportCombinedPaginationCursor,
    ReportCombinedView,
    ReportCombinedViewInternal,
  },
  InternalToCombinedView,
};
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  PgExpressionMethods,
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
  source::{
    combined::report::{report_combined_keys as key, ReportCombined},
    community::CommunityFollower,
  },
  utils::{actions, actions_alias, functions::coalesce, get_conn, DbPool, ReverseTimestampKey},
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
      .filter(
        post_report::resolved
          .or(comment_report::resolved)
          .or(private_message_report::resolved)
          .is_distinct_from(true),
      )
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
    let (prefix, id) = match view {
      ReportCombinedView::Comment(v) => ('C', v.comment_report.id.0),
      ReportCombinedView::Post(v) => ('P', v.post_report.id.0),
      ReportCombinedView::PrivateMessage(v) => ('M', v.private_message_report.id.0),
    };
    // hex encoding to prevent ossification
    ReportCombinedPaginationCursor(format!("{prefix}{id:x}"))
  }

  pub async fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
    let err_msg = || Error::QueryBuilderError("Could not parse pagination token".into());
    let mut query = report_combined::table
      .select(ReportCombined::as_select())
      .into_boxed();
    let (prefix, id_str) = self.0.split_at_checked(1).ok_or_else(err_msg)?;
    let id = i32::from_str_radix(id_str, 16).map_err(|_err| err_msg())?;
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

#[derive(Clone)]
pub struct PaginationCursorData(ReportCombined);

#[derive(Default)]
pub struct ReportCombinedQuery {
  pub community_id: Option<CommunityId>,
  pub unresolved_only: Option<bool>,
  pub page_after: Option<PaginationCursorData>,
  pub page_back: Option<bool>,
}

impl ReportCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> LemmyResult<Vec<ReportCombinedView>> {
    let my_person_id = user.local_user.person_id;
    let report_creator = person::id;
    let item_creator = aliases::person1.field(person::id);
    let resolver = aliases::person2.field(person::id).nullable();

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
            .eq(report_creator)
            .or(comment_report::creator_id.eq(report_creator))
            .or(private_message_report::creator_id.eq(report_creator)),
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
      // The item creator (`item_creator` is the id of this person)
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
      // The resolver
      .left_join(
        aliases::person2.on(
          private_message_report::resolver_id
            .eq(resolver)
            .or(post_report::resolver_id.eq(resolver))
            .or(comment_report::resolver_id.eq(resolver)),
        ),
      )
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

    // If viewing all reports, order by newest, but if viewing unresolved only, show the oldest
    // first (FIFO)
    if self.unresolved_only.unwrap_or_default() {
      query = query
        .filter(
          post_report::resolved
            .or(comment_report::resolved)
            .or(private_message_report::resolved)
            .is_distinct_from(true),
        )
        // TODO: when a `then_asc` method is added, use it here, make the id sort direction match,
        // and remove the separate index; unless additional columns are added to this sort
        .then_desc(ReverseTimestampKey(key::published));
    } else {
      query = query.then_desc(key::published);
    }

    // Tie breaker
    query = query.then_desc(key::id);

    let res = query.load::<ReportCombinedViewInternal>(conn).await?;

    // Map the query results to the enum
    let out = res.into_iter().filter_map(|u| u.map_to_enum()).collect();

    Ok(out)
  }
}

impl InternalToCombinedView for ReportCombinedViewInternal {
  type CombinedView = ReportCombinedView;

  fn map_to_enum(&self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self.clone();

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
      v.post,
      v.community,
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
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::{
    report_combined_view::ReportCombinedQuery,
    structs::{
      CommentReportView,
      LocalUserView,
      PostReportView,
      ReportCombinedView,
      ReportCombinedViewInternal,
    },
  };
  use lemmy_db_schema::{
    aggregates::structs::{CommentAggregates, PostAggregates},
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
    utils::{build_db_pool_for_tests, DbPool},
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    instance: Instance,
    timmy: Person,
    sara: Person,
    jessica: Person,
    timmy_view: LocalUserView,
    admin_view: LocalUserView,
    community: Community,
    post: Post,
    post_2: Post,
    comment: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
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

    let jessica_form = PersonInsertForm::test_form(inserted_instance.id, "jessica_mrv");
    let inserted_jessica = Person::create(pool, &jessica_form).await?;

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

    let new_post_2 = PostInsertForm::new(
      "A test post crv 2".into(),
      inserted_timmy.id,
      inserted_community.id,
    );
    let inserted_post_2 = Post::create(pool, &new_post_2).await?;

    // Timmy creates a comment
    let comment_form = CommentInsertForm::new(
      inserted_timmy.id,
      inserted_post.id,
      "A test comment rv".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    Ok(Data {
      instance: inserted_instance,
      timmy: inserted_timmy,
      sara: inserted_sara,
      jessica: inserted_jessica,
      admin_view,
      timmy_view,
      community: inserted_community,
      post: inserted_post,
      post_2: inserted_post_2,
      comment: inserted_comment,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_combined() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // sara reports the post
    let sara_report_post_form = PostReportForm {
      creator_id: data.sara.id,
      post_id: data.post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from sara".into(),
    };
    let inserted_post_report = PostReport::report(pool, &sara_report_post_form).await?;

    // Sara reports the comment
    let sara_report_comment_form = CommentReportForm {
      creator_id: data.sara.id,
      comment_id: data.comment.id,
      original_comment_text: "A test comment rv".into(),
      reason: "from sara".into(),
    };
    CommentReport::report(pool, &sara_report_comment_form).await?;

    // Timmy creates a private message
    let pm_form = PrivateMessageInsertForm::new(
      data.timmy.id,
      data.sara.id,
      "something offensive crv".to_string(),
    );
    let inserted_pm = PrivateMessage::create(pool, &pm_form).await?;

    // sara reports private message
    let pm_report_form = PrivateMessageReportForm {
      creator_id: data.sara.id,
      original_pm_text: inserted_pm.content.clone(),
      private_message_id: inserted_pm.id,
      reason: "its offensive".to_string(),
    };
    PrivateMessageReport::report(pool, &pm_report_form).await?;

    // Do a batch read of admins reports
    let reports = ReportCombinedQuery::default()
      .list(pool, &data.admin_view)
      .await?;
    assert_eq!(3, reports.len());

    // Make sure the report types are correct
    if let ReportCombinedView::Post(v) = &reports[2] {
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.sara.id, v.creator.id);
      assert_eq!(data.timmy.id, v.post_creator.id);
    } else {
      panic!("wrong type");
    }
    if let ReportCombinedView::Comment(v) = &reports[1] {
      assert_eq!(data.comment.id, v.comment.id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.comment_creator.id);
    } else {
      panic!("wrong type");
    }
    if let ReportCombinedView::PrivateMessage(v) = &reports[0] {
      assert_eq!(inserted_pm.id, v.private_message.id);
    } else {
      panic!("wrong type");
    }

    let report_count_admin =
      ReportCombinedViewInternal::get_report_count(pool, &data.admin_view, None).await?;
    assert_eq!(3, report_count_admin);

    // Timmy should only see 2 reports, since they're not an admin,
    // but they do mod the community
    let reports = ReportCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_eq!(2, reports.len());

    // Make sure the report types are correct
    if let ReportCombinedView::Post(v) = &reports[1] {
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.sara.id, v.creator.id);
      assert_eq!(data.timmy.id, v.post_creator.id);
    } else {
      panic!("wrong type");
    }
    if let ReportCombinedView::Comment(v) = &reports[0] {
      assert_eq!(data.comment.id, v.comment.id);
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.timmy.id, v.comment_creator.id);
    } else {
      panic!("wrong type");
    }

    let report_count_timmy =
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view, None).await?;
    assert_eq!(2, report_count_timmy);

    // Resolve the post report
    PostReport::resolve(pool, inserted_post_report.id, data.timmy.id).await?;

    // Do a batch read of timmys reports
    // It should only show saras, which is unresolved
    let reports_after_resolve = ReportCombinedQuery {
      unresolved_only: Some(true),
      ..Default::default()
    }
    .list(pool, &data.timmy_view)
    .await?;
    assert_length!(1, reports_after_resolve);

    // Make sure the counts are correct
    let report_count_after_resolved =
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view, None).await?;
    assert_eq!(1, report_count_after_resolved);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_private_message_reports() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // timmy sends private message to jessica
    let pm_form = PrivateMessageInsertForm::new(
      data.timmy.id,
      data.jessica.id,
      "something offensive".to_string(),
    );
    let pm = PrivateMessage::create(pool, &pm_form).await?;

    // jessica reports private message
    let pm_report_form = PrivateMessageReportForm {
      creator_id: data.jessica.id,
      original_pm_text: pm.content.clone(),
      private_message_id: pm.id,
      reason: "its offensive".to_string(),
    };
    let pm_report = PrivateMessageReport::report(pool, &pm_report_form).await?;

    let reports = ReportCombinedQuery::default()
      .list(pool, &data.admin_view)
      .await?;
    assert_length!(1, reports);
    if let ReportCombinedView::PrivateMessage(v) = &reports[0] {
      assert!(!v.private_message_report.resolved);
      assert_eq!(data.timmy.name, v.private_message_creator.name);
      assert_eq!(data.jessica.name, v.creator.name);
      assert_eq!(pm_report.reason, v.private_message_report.reason);
      assert_eq!(pm.content, v.private_message.content);
    } else {
      panic!("wrong type");
    }

    // admin resolves the report (after taking appropriate action)
    PrivateMessageReport::resolve(pool, pm_report.id, data.admin_view.person.id).await?;

    let reports = ReportCombinedQuery::default()
      .list(pool, &data.admin_view)
      .await?;
    assert_length!(1, reports);
    if let ReportCombinedView::PrivateMessage(v) = &reports[0] {
      assert!(v.private_message_report.resolved);
      assert!(v.resolver.is_some());
      assert_eq!(
        Some(&data.admin_view.person.name),
        v.resolver.as_ref().map(|r| &r.name)
      );
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_post_reports() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // sara reports
    let sara_report_form = PostReportForm {
      creator_id: data.sara.id,
      post_id: data.post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from sara".into(),
    };

    PostReport::report(pool, &sara_report_form).await?;

    // jessica reports
    let jessica_report_form = PostReportForm {
      creator_id: data.jessica.id,
      post_id: data.post_2.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from jessica".into(),
    };

    let inserted_jessica_report = PostReport::report(pool, &jessica_report_form).await?;

    let read_jessica_report_view =
      PostReportView::read(pool, inserted_jessica_report.id, data.timmy.id).await?;

    // Make sure the triggers are reading the aggregates correctly.
    let agg_1 = PostAggregates::read(pool, data.post.id).await?;
    let agg_2 = PostAggregates::read(pool, data.post_2.id).await?;

    assert_eq!(
      read_jessica_report_view.post_report,
      inserted_jessica_report
    );
    assert_eq!(read_jessica_report_view.post, data.post_2);
    assert_eq!(read_jessica_report_view.community.id, data.community.id);
    assert_eq!(read_jessica_report_view.creator.id, data.jessica.id);
    assert_eq!(read_jessica_report_view.post_creator.id, data.timmy.id);
    assert_eq!(read_jessica_report_view.my_vote, None);
    assert_eq!(read_jessica_report_view.resolver, None);
    assert_eq!(agg_1.report_count, 1);
    assert_eq!(agg_1.unresolved_report_count, 1);
    assert_eq!(agg_2.report_count, 1);
    assert_eq!(agg_2.unresolved_report_count, 1);

    // Do a batch read of timmys reports
    let reports = ReportCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;

    if let ReportCombinedView::Post(v) = &reports[1] {
      assert_eq!(v.creator.id, data.sara.id);
    } else {
      panic!("wrong type");
    }
    if let ReportCombinedView::Post(v) = &reports[0] {
      assert_eq!(v.creator.id, data.jessica.id);
    } else {
      panic!("wrong type");
    }

    // Make sure the counts are correct
    let report_count =
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view, None).await?;
    assert_eq!(2, report_count);

    // Pretend the post was removed, and resolve all reports for that object.
    // This is called manually in the API for post removals
    PostReport::resolve_all_for_object(pool, inserted_jessica_report.post_id, data.timmy.id)
      .await?;

    let read_jessica_report_view_after_resolve =
      PostReportView::read(pool, inserted_jessica_report.id, data.timmy.id).await?;
    assert!(read_jessica_report_view_after_resolve.post_report.resolved);
    assert_eq!(
      read_jessica_report_view_after_resolve
        .post_report
        .resolver_id,
      Some(data.timmy.id)
    );
    assert_eq!(
      read_jessica_report_view_after_resolve
        .resolver
        .map(|r| r.id),
      Some(data.timmy.id)
    );

    // Make sure the unresolved_post report got decremented in the trigger
    let agg_2 = PostAggregates::read(pool, data.post_2.id).await?;
    assert_eq!(agg_2.report_count, 1);
    assert_eq!(agg_2.unresolved_report_count, 0);

    // Make sure the other unresolved report isn't changed
    let agg_1 = PostAggregates::read(pool, data.post.id).await?;
    assert_eq!(agg_1.report_count, 1);
    assert_eq!(agg_1.unresolved_report_count, 1);

    // Do a batch read of timmys reports
    // It should only show saras, which is unresolved
    let reports_after_resolve = ReportCombinedQuery {
      unresolved_only: Some(true),
      ..Default::default()
    }
    .list(pool, &data.timmy_view)
    .await?;

    if let ReportCombinedView::Post(v) = &reports_after_resolve[0] {
      assert_length!(1, reports_after_resolve);
      assert_eq!(v.creator.id, data.sara.id);
    } else {
      panic!("wrong type");
    }

    // Make sure the counts are correct
    let report_count_after_resolved =
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view, None).await?;
    assert_eq!(1, report_count_after_resolved);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_comment_reports() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // sara reports
    let sara_report_form = CommentReportForm {
      creator_id: data.sara.id,
      comment_id: data.comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from sara".into(),
    };

    CommentReport::report(pool, &sara_report_form).await?;

    // jessica reports
    let jessica_report_form = CommentReportForm {
      creator_id: data.jessica.id,
      comment_id: data.comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from jessica".into(),
    };

    let inserted_jessica_report = CommentReport::report(pool, &jessica_report_form).await?;

    let agg = CommentAggregates::read(pool, data.comment.id).await?;
    assert_eq!(agg.report_count, 2);

    let read_jessica_report_view =
      CommentReportView::read(pool, inserted_jessica_report.id, data.timmy.id).await?;
    assert_eq!(read_jessica_report_view.counts.unresolved_report_count, 2);

    // Do a batch read of timmys reports
    let reports = ReportCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;

    if let ReportCombinedView::Comment(v) = &reports[0] {
      assert_eq!(v.creator.id, data.jessica.id);
    } else {
      panic!("wrong type");
    }
    if let ReportCombinedView::Comment(v) = &reports[1] {
      assert_eq!(v.creator.id, data.sara.id);
    } else {
      panic!("wrong type");
    }

    // Make sure the counts are correct
    let report_count =
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view, None).await?;
    assert_eq!(2, report_count);

    // Resolve the report
    CommentReport::resolve(pool, inserted_jessica_report.id, data.timmy.id).await?;
    let read_jessica_report_view_after_resolve =
      CommentReportView::read(pool, inserted_jessica_report.id, data.timmy.id).await?;

    assert!(
      read_jessica_report_view_after_resolve
        .comment_report
        .resolved
    );
    assert_eq!(
      read_jessica_report_view_after_resolve
        .comment_report
        .resolver_id,
      Some(data.timmy.id)
    );
    assert_eq!(
      read_jessica_report_view_after_resolve
        .resolver
        .map(|r| r.id),
      Some(data.timmy.id)
    );

    // Do a batch read of timmys reports
    // It should only show saras, which is unresolved
    let reports_after_resolve = ReportCombinedQuery {
      unresolved_only: Some(true),
      ..Default::default()
    }
    .list(pool, &data.timmy_view)
    .await?;

    if let ReportCombinedView::Comment(v) = &reports_after_resolve[0] {
      assert_length!(1, reports_after_resolve);
      assert_eq!(v.creator.id, data.sara.id);
    } else {
      panic!("wrong type");
    }

    // Make sure the counts are correct
    let report_count_after_resolved =
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view, None).await?;
    assert_eq!(1, report_count_after_resolved);

    cleanup(data, pool).await?;

    Ok(())
  }
}
