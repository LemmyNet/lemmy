use crate::structs::{
  CommentReportView,
  CommunityReportView,
  LocalUserView,
  PostReportView,
  PrivateMessageReportView,
  ReportCombinedPaginationCursor,
  ReportCombinedView,
  ReportCombinedViewInternal,
};
use chrono::{DateTime, Days, Utc};
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
  impls::community::community_follower_select_subscribed_type,
  newtypes::{CommunityId, PersonId, PostId},
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    comment_report,
    community,
    community_actions,
    community_aggregates,
    community_report,
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
  source::combined::report::{report_combined_keys as key, ReportCombined},
  traits::InternalToCombinedView,
  utils::{functions::coalesce, get_conn, DbPool, ReverseTimestampKey},
  ReportType,
};
use lemmy_utils::error::LemmyResult;

impl ReportCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: PersonId) -> _ {
    let report_creator = person::id;
    let item_creator = aliases::person1.field(person::id);
    let resolver = aliases::person2.field(person::id).nullable();

    let comment_join = comment::table.on(comment_report::comment_id.eq(comment::id));
    let private_message_join =
      private_message::table.on(private_message_report::private_message_id.eq(private_message::id));

    let post_join = post::table.on(
      post_report::post_id
        .eq(post::id)
        .or(comment::post_id.eq(post::id)),
    );

    let community_actions_join = community_actions::table.on(
      community_actions::community_id
        .eq(community::id)
        .and(community_actions::person_id.eq(my_person_id)),
    );

    let report_creator_join = person::table.on(
      post_report::creator_id
        .eq(report_creator)
        .or(comment_report::creator_id.eq(report_creator))
        .or(private_message_report::creator_id.eq(report_creator))
        .or(community_report::creator_id.eq(report_creator)),
    );

    let item_creator_join = aliases::person1.on(
      post::creator_id
        .eq(item_creator)
        .or(comment::creator_id.eq(item_creator))
        .or(private_message::creator_id.eq(item_creator)),
    );

    let resolver_join = aliases::person2.on(
      private_message_report::resolver_id
        .eq(resolver)
        .or(post_report::resolver_id.eq(resolver))
        .or(comment_report::resolver_id.eq(resolver))
        .or(community_report::resolver_id.eq(resolver)),
    );

    let community_join = community::table.on(
      community_report::community_id
        .eq(community::id)
        .or(post::community_id.eq(community::id)),
    );

    let local_user_join = local_user::table.on(
      item_creator
        .eq(local_user::person_id)
        .and(local_user::admin.eq(true)),
    );

    let creator_community_actions_join = creator_community_actions.on(
      creator_community_actions
        .field(community_actions::community_id)
        .eq(post::community_id)
        .and(
          creator_community_actions
            .field(community_actions::person_id)
            .eq(item_creator),
        ),
    );

    let post_actions_join = post_actions::table.on(
      post_actions::post_id
        .eq(post::id)
        .and(post_actions::person_id.eq(my_person_id)),
    );

    let person_actions_join = person_actions::table.on(
      person_actions::target_id
        .eq(item_creator)
        .and(person_actions::person_id.eq(my_person_id)),
    );

    let comment_actions_join = comment_actions::table.on(
      comment_actions::comment_id
        .eq(comment::id)
        .and(comment_actions::person_id.eq(my_person_id)),
    );

    let post_aggregates_join =
      post_aggregates::table.on(post_report::post_id.eq(post_aggregates::post_id));

    let comment_aggregates_join =
      comment_aggregates::table.on(comment_report::comment_id.eq(comment_aggregates::comment_id));

    let community_aggregates_join = community_aggregates::table
      .on(community_report::community_id.eq(community_aggregates::community_id));

    report_combined::table
      .left_join(post_report::table)
      .left_join(comment_report::table)
      .left_join(private_message_report::table)
      .left_join(community_report::table)
      .inner_join(report_creator_join)
      .left_join(comment_join)
      .left_join(private_message_join)
      .left_join(post_join)
      .left_join(item_creator_join)
      .left_join(resolver_join)
      .left_join(community_join)
      .left_join(creator_community_actions_join)
      .left_join(local_user_join)
      .left_join(community_actions_join)
      .left_join(post_actions_join)
      .left_join(person_actions_join)
      .left_join(post_aggregates_join)
      .left_join(comment_aggregates_join)
      .left_join(community_aggregates_join)
      .left_join(comment_actions_join)
  }

  /// returns the current unresolved report count for the communities you mod
  pub async fn get_report_count(
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
    community_id: Option<CommunityId>,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;

    let conn = &mut get_conn(pool).await?;
    let my_person_id = user.local_user.person_id;

    let mut query = Self::joins(my_person_id)
      .filter(
        post_report::resolved
          .or(comment_report::resolved)
          .or(private_message_report::resolved)
          .or(community_report::resolved)
          .is_distinct_from(true),
      )
      .select(count(report_combined::id))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(
        community::id
          .eq(community_id)
          .and(report_combined::community_report_id.is_null()),
      );
    }

    if user.local_user.admin {
      query = query.filter(filter_admin_reports(Utc::now() - Days::new(3)));
    } else {
      query = query.filter(filter_mod_reports());
    }

    query.first::<i64>(conn).await
  }
}

impl ReportCombinedPaginationCursor {
  // get cursor for page that starts immediately after the given post
  pub fn after_post(view: &ReportCombinedView) -> ReportCombinedPaginationCursor {
    let (prefix, id) = match view {
      ReportCombinedView::Comment(v) => ('C', v.comment_report.id.0),
      ReportCombinedView::Post(v) => ('P', v.post_report.id.0),
      ReportCombinedView::PrivateMessage(v) => ('M', v.private_message_report.id.0),
      ReportCombinedView::Community(v) => ('Y', v.community_report.id.0),
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
      "Y" => query.filter(report_combined::community_report_id.eq(id)),
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
  pub type_: Option<ReportType>,
  pub post_id: Option<PostId>,
  pub community_id: Option<CommunityId>,
  pub unresolved_only: Option<bool>,
  /// For admins, also show reports with `violates_instance_rules=false`
  pub show_community_rule_violations: Option<bool>,
  pub my_reports_only: Option<bool>,
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

    let conn = &mut get_conn(pool).await?;
    let mut query = ReportCombinedViewInternal::joins(my_person_id)
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
        post_actions::saved.nullable(),
        post_actions::read.nullable().is_not_null(),
        post_actions::hidden.nullable().is_not_null(),
        post_actions::like_score.nullable(),
        // Comment-specific
        comment_report::all_columns.nullable(),
        comment::all_columns.nullable(),
        comment_aggregates::all_columns.nullable(),
        comment_actions::saved.nullable(),
        comment_actions::like_score.nullable(),
        // Private-message-specific
        private_message_report::all_columns.nullable(),
        private_message::all_columns.nullable(),
        // Community-specific
        community_report::all_columns.nullable(),
        community_aggregates::all_columns.nullable(),
        // Shared
        person::all_columns,
        aliases::person1.fields(person::all_columns.nullable()),
        community::all_columns.nullable(),
        community_follower_select_subscribed_type(),
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
      query = query.filter(
        community::id
          .eq(community_id)
          .and(report_combined::community_report_id.is_null()),
      );
    }

    if user.local_user.admin {
      let show_community_rule_violations = self.show_community_rule_violations.unwrap_or_default();
      if !show_community_rule_violations {
        query = query.filter(filter_admin_reports(Utc::now() - Days::new(3)));
      }
    } else {
      query = query.filter(filter_mod_reports());
    }

    if let Some(post_id) = self.post_id {
      query = query.filter(post::id.eq(post_id));
    }

    if self.my_reports_only.unwrap_or_default() {
      query = query.filter(person::id.eq(my_person_id));
    }

    let mut query = PaginatedQueryBuilder::new(query);

    let page_after = self.page_after.map(|c| c.0);

    if self.page_back.unwrap_or_default() {
      query = query.before(page_after).limit_and_offset_from_end();
    } else {
      query = query.after(page_after);
    }

    if let Some(type_) = self.type_ {
      query = match type_ {
        ReportType::All => query,
        ReportType::Posts => query.filter(report_combined::post_report_id.is_not_null()),
        ReportType::Comments => query.filter(report_combined::comment_report_id.is_not_null()),
        ReportType::PrivateMessages => {
          query.filter(report_combined::private_message_report_id.is_not_null())
        }
        ReportType::Communities => query.filter(report_combined::community_report_id.is_not_null()),
      }
    }

    // If viewing all reports, order by newest, but if viewing unresolved only, show the oldest
    // first (FIFO)
    if self.unresolved_only.unwrap_or_default() {
      query = query
        .filter(
          post_report::resolved
            .or(comment_report::resolved)
            .or(private_message_report::resolved)
            .or(community_report::resolved)
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
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    Ok(out)
  }
}

/// Mods can only see reports for posts/comments inside of communities where they are moderator,
/// and which have `violates_instance_rules == false`.
#[diesel::dsl::auto_type]
fn filter_mod_reports() -> _ {
  community_actions::became_moderator
    .is_not_null()
    // Reporting a community or private message must go to admins
    .and(report_combined::community_report_id.is_null())
    .and(report_combined::private_message_report_id.is_null())
    .and(filter_violates_instance_rules().is_distinct_from(true))
}

/// Admins can see reports intended for them, or mod reports older than 3 days. Also reports
/// on communities, person and private messages.
#[diesel::dsl::auto_type]
fn filter_admin_reports(interval: DateTime<Utc>) -> _ {
  filter_violates_instance_rules()
    .or(report_combined::published.lt(interval))
    // Also show community reports where the admin is a community mod
    .or(community_actions::became_moderator.is_not_null())
}

/// Filter reports which are only for admins (either post/comment report with
/// `violates_instance_rules=true`, or report on a community/person/private message.
#[diesel::dsl::auto_type]
fn filter_violates_instance_rules() -> _ {
  post_report::violates_instance_rules
    .or(comment_report::violates_instance_rules)
    .or(report_combined::community_report_id.is_not_null())
    .or(report_combined::private_message_report_id.is_not_null())
}

impl InternalToCombinedView for ReportCombinedViewInternal {
  type CombinedView = ReportCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let (
      Some(post_report),
      Some(post),
      Some(community),
      Some(unread_comments),
      Some(counts),
      Some(post_creator),
    ) = (
      v.post_report,
      v.post.clone(),
      v.community.clone(),
      v.post_unread_comments,
      v.post_counts,
      v.item_creator.clone(),
    ) {
      Some(ReportCombinedView::Post(PostReportView {
        post_report,
        post,
        community,
        unread_comments,
        counts,
        creator: v.report_creator,
        post_creator,
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
    } else if let (
      Some(comment_report),
      Some(comment),
      Some(counts),
      Some(post),
      Some(community),
      Some(comment_creator),
    ) = (
      v.comment_report,
      v.comment,
      v.comment_counts,
      v.post,
      v.community.clone(),
      v.item_creator.clone(),
    ) {
      Some(ReportCombinedView::Comment(CommentReportView {
        comment_report,
        comment,
        counts,
        post,
        community,
        creator: v.report_creator,
        comment_creator,
        creator_banned_from_community: v.item_creator_banned_from_community,
        creator_is_moderator: v.item_creator_is_moderator,
        creator_is_admin: v.item_creator_is_admin,
        creator_blocked: v.item_creator_blocked,
        subscribed: v.subscribed,
        saved: v.comment_saved,
        my_vote: v.my_comment_vote,
        resolver: v.resolver,
      }))
    } else if let (
      Some(private_message_report),
      Some(private_message),
      Some(private_message_creator),
    ) = (v.private_message_report, v.private_message, v.item_creator)
    {
      Some(ReportCombinedView::PrivateMessage(
        PrivateMessageReportView {
          private_message_report,
          private_message,
          creator: v.report_creator,
          private_message_creator,
          resolver: v.resolver,
        },
      ))
    } else if let (Some(community), Some(community_report), Some(counts)) =
      (v.community, v.community_report, v.community_counts)
    {
      Some(ReportCombinedView::Community(CommunityReportView {
        community_report,
        community,
        creator: v.report_creator,
        counts,
        subscribed: v.subscribed,
        resolver: v.resolver,
      }))
    } else {
      None
    }
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::{
    combined::report_combined_view::ReportCombinedQuery,
    structs::{
      CommentReportView,
      LocalUserView,
      PostReportView,
      ReportCombinedView,
      ReportCombinedViewInternal,
    },
  };
  use chrono::{Days, Utc};
  use diesel::{update, ExpressionMethods, QueryDsl};
  use diesel_async::RunQueryDsl;
  use lemmy_db_schema::{
    aggregates::structs::{CommentAggregates, PostAggregates},
    assert_length,
    schema::report_combined,
    source::{
      comment::{Comment, CommentInsertForm},
      comment_report::{CommentReport, CommentReportForm},
      community::{Community, CommunityInsertForm, CommunityModerator, CommunityModeratorForm},
      community_report::{CommunityReport, CommunityReportForm},
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
    utils::{build_db_pool_for_tests, get_conn, DbPool},
    ReportType,
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
  async fn combined() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Sara reports the community
    let sara_report_community_form = CommunityReportForm {
      creator_id: data.sara.id,
      community_id: data.community.id,
      original_community_name: data.community.name.clone(),
      original_community_title: data.community.title.clone(),
      original_community_banner: None,
      original_community_description: None,
      original_community_sidebar: None,
      original_community_icon: None,
      reason: "from sara".into(),
    };
    CommunityReport::report(pool, &sara_report_community_form).await?;

    // sara reports the post
    let sara_report_post_form = PostReportForm {
      creator_id: data.sara.id,
      post_id: data.post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from sara".into(),
      violates_instance_rules: false,
    };
    let inserted_post_report = PostReport::report(pool, &sara_report_post_form).await?;

    // Sara reports the comment
    let sara_report_comment_form = CommentReportForm {
      creator_id: data.sara.id,
      comment_id: data.comment.id,
      original_comment_text: "A test comment rv".into(),
      reason: "from sara".into(),
      violates_instance_rules: false,
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
    let reports = ReportCombinedQuery {
      show_community_rule_violations: Some(true),
      ..Default::default()
    }
    .list(pool, &data.admin_view)
    .await?;
    assert_length!(4, reports);

    // Make sure the report types are correct
    if let ReportCombinedView::Community(v) = &reports[3] {
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }
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

    let report_count_mod =
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view, None).await?;
    assert_eq!(2, report_count_mod);
    let report_count_admin =
      ReportCombinedViewInternal::get_report_count(pool, &data.admin_view, None).await?;
    assert_eq!(2, report_count_admin);

    // Make sure the type_ filter is working
    let reports_by_type = ReportCombinedQuery {
      type_: Some(ReportType::Posts),
      ..Default::default()
    }
    .list(pool, &data.timmy_view)
    .await?;
    assert_length!(1, reports_by_type);

    // Filter by the post id
    // Should be 2, for the post, and the comment on that post
    let reports_by_post_id = ReportCombinedQuery {
      post_id: Some(data.post.id),
      ..Default::default()
    }
    .list(pool, &data.timmy_view)
    .await?;
    assert_length!(2, reports_by_post_id);

    // Timmy should only see 2 reports, since they're not an admin,
    // but they do mod the community
    let timmys_reports = ReportCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_length!(2, timmys_reports);

    // Make sure the report types are correct
    if let ReportCombinedView::Post(v) = &timmys_reports[1] {
      assert_eq!(data.post.id, v.post.id);
      assert_eq!(data.sara.id, v.creator.id);
      assert_eq!(data.timmy.id, v.post_creator.id);
    } else {
      panic!("wrong type");
    }
    if let ReportCombinedView::Comment(v) = &timmys_reports[0] {
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
  async fn private_message_reports() -> LemmyResult<()> {
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

    let reports = ReportCombinedQuery {
      show_community_rule_violations: Some(true),
      ..Default::default()
    }
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
  async fn post_reports() -> LemmyResult<()> {
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
      violates_instance_rules: false,
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
      violates_instance_rules: false,
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
  async fn comment_reports() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // sara reports
    let sara_report_form = CommentReportForm {
      creator_id: data.sara.id,
      comment_id: data.comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from sara".into(),
      violates_instance_rules: false,
    };

    CommentReport::report(pool, &sara_report_form).await?;

    // jessica reports
    let jessica_report_form = CommentReportForm {
      creator_id: data.jessica.id,
      comment_id: data.comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from jessica".into(),
      violates_instance_rules: false,
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

    // Filter by post id, which should still include the comments.
    let reports_post_id_filter = ReportCombinedQuery {
      post_id: Some(data.post.id),
      ..Default::default()
    }
    .list(pool, &data.timmy_view)
    .await?;

    assert_length!(2, reports_post_id_filter);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn community_reports() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // jessica reports community
    let community_report_form = CommunityReportForm {
      creator_id: data.jessica.id,
      community_id: data.community.id,
      original_community_name: data.community.name.clone(),
      original_community_title: data.community.title.clone(),
      original_community_banner: None,
      original_community_description: None,
      original_community_sidebar: None,
      original_community_icon: None,
      reason: "the ice cream incident".into(),
    };
    let community_report = CommunityReport::report(pool, &community_report_form).await?;

    let reports = ReportCombinedQuery {
      show_community_rule_violations: Some(true),
      ..Default::default()
    }
    .list(pool, &data.admin_view)
    .await?;
    assert_length!(1, reports);
    if let ReportCombinedView::Community(v) = &reports[0] {
      assert!(!v.community_report.resolved);
      assert_eq!(data.jessica.name, v.creator.name);
      assert_eq!(community_report.reason, v.community_report.reason);
      assert_eq!(data.community.name, v.community.name);
      assert_eq!(data.community.title, v.community.title);
    } else {
      panic!("wrong type");
    }

    // admin resolves the report (after taking appropriate action)
    CommunityReport::resolve(pool, community_report.id, data.admin_view.person.id).await?;

    let reports = ReportCombinedQuery {
      show_community_rule_violations: Some(true),
      ..Default::default()
    }
    .list(pool, &data.admin_view)
    .await?;
    assert_length!(1, reports);
    if let ReportCombinedView::Community(v) = &reports[0] {
      assert!(v.community_report.resolved);
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
  async fn violates_instance_rules() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // create report to admins
    let report_form = PostReportForm {
      creator_id: data.sara.id,
      post_id: data.post_2.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from sara".into(),
      violates_instance_rules: true,
    };
    PostReport::report(pool, &report_form).await?;

    // timmy is a mod and cannot see the report
    let mod_reports = ReportCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_length!(0, mod_reports);
    let count = ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view, None).await?;
    assert_eq!(0, count);

    // only admin can see the report
    let admin_reports = ReportCombinedQuery::default()
      .list(pool, &data.admin_view)
      .await?;
    assert_length!(1, admin_reports);
    let count = ReportCombinedViewInternal::get_report_count(pool, &data.admin_view, None).await?;
    assert_eq!(1, count);

    // cleanup the report for easier checks below
    Post::delete(pool, data.post_2.id).await?;

    // now create a mod report
    let report_form = CommentReportForm {
      creator_id: data.sara.id,
      comment_id: data.comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from sara".into(),
      violates_instance_rules: false,
    };
    let comment_report = CommentReport::report(pool, &report_form).await?;

    // this time the mod can see it
    let mod_reports = ReportCombinedQuery::default()
      .list(pool, &data.timmy_view)
      .await?;
    assert_length!(1, mod_reports);
    let count = ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view, None).await?;
    assert_eq!(1, count);

    // but not the admin
    let admin_reports = ReportCombinedQuery::default()
      .list(pool, &data.admin_view)
      .await?;
    assert_length!(0, admin_reports);
    let count = ReportCombinedViewInternal::get_report_count(pool, &data.admin_view, None).await?;
    assert_eq!(0, count);

    // admin can see the report with `view_mod_reports` set
    let admin_reports = ReportCombinedQuery {
      show_community_rule_violations: Some(true),
      ..Default::default()
    }
    .list(pool, &data.timmy_view)
    .await?;
    assert_length!(1, admin_reports);

    // change a comment to be 3 days old, now admin can also see it by default
    update(
      report_combined::table.filter(report_combined::dsl::comment_report_id.eq(comment_report.id)),
    )
    .set(report_combined::published.eq(Utc::now() - Days::new(3)))
    .execute(&mut get_conn(pool).await?)
    .await?;
    let admin_reports = ReportCombinedQuery::default()
      .list(pool, &data.admin_view)
      .await?;
    assert_length!(1, admin_reports);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn my_reports_only() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // sara reports
    let sara_report_form = CommentReportForm {
      creator_id: data.sara.id,
      comment_id: data.comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from sara".into(),
      violates_instance_rules: false,
    };
    CommentReport::report(pool, &sara_report_form).await?;

    // timmy reports
    let timmy_report_form = CommentReportForm {
      creator_id: data.timmy.id,
      comment_id: data.comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from timmy".into(),
      violates_instance_rules: false,
    };
    CommentReport::report(pool, &timmy_report_form).await?;

    let agg = CommentAggregates::read(pool, data.comment.id).await?;
    assert_eq!(agg.report_count, 2);

    // Do a batch read of timmys reports, it should only show his own
    let reports = ReportCombinedQuery {
      my_reports_only: Some(true),
      ..Default::default()
    }
    .list(pool, &data.timmy_view)
    .await?;

    assert_length!(1, reports);

    if let ReportCombinedView::Comment(v) = &reports[0] {
      assert_eq!(v.creator.id, data.timmy.id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }
}
