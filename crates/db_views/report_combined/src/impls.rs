use crate::{
  CommentReportView,
  CommunityReportView,
  LocalUserView,
  PostReportView,
  PrivateMessageReportView,
  ReportCombinedView,
  ReportCombinedViewInternal,
};
use chrono::{DateTime, Days, Utc};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  PgExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::asc_if;
use lemmy_db_schema::{
  ReportType,
  newtypes::{
    CommentReportId,
    CommunityId,
    CommunityReportId,
    PostId,
    PostReportId,
    PrivateMessageReportId,
  },
  source::{
    combined::report::{ReportCombined, report_combined_keys as key},
    person::Person,
  },
  traits::InternalToCombinedView,
  utils::limit_fetch,
};
use lemmy_db_schema_file::{
  aliases,
  schema::{
    comment_report,
    community,
    community_actions,
    community_report,
    person,
    post,
    post_report,
    private_message_report,
    report_combined,
  },
};
use lemmy_db_views_report_combined_sql::report_combined_joins;
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{
    CursorData,
    PagedResponse,
    PaginationCursor,
    PaginationCursorConversion,
    paginate_response,
  },
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl ReportCombinedViewInternal {
  pub async fn read_comment_report(
    pool: &mut DbPool<'_>,
    report_id: CommentReportId,
    my_person: &Person,
  ) -> LemmyResult<CommentReportView> {
    let conn = &mut get_conn(pool).await?;
    let res = report_combined_joins(my_person.id, my_person.instance_id)
      .filter(report_combined::comment_report_id.eq(report_id))
      .select(ReportCombinedViewInternal::as_select())
      .first(conn)
      .await?;

    let res = InternalToCombinedView::map_to_enum(res);
    let Some(ReportCombinedView::Comment(c)) = res else {
      return Err(LemmyErrorType::NotFound.into());
    };
    Ok(c)
  }

  pub async fn read_post_report(
    pool: &mut DbPool<'_>,
    report_id: PostReportId,
    my_person: &Person,
  ) -> LemmyResult<PostReportView> {
    let conn = &mut get_conn(pool).await?;
    let res = report_combined_joins(my_person.id, my_person.instance_id)
      .filter(report_combined::post_report_id.eq(report_id))
      .select(ReportCombinedViewInternal::as_select())
      .first(conn)
      .await?;

    let res = InternalToCombinedView::map_to_enum(res);
    let Some(ReportCombinedView::Post(p)) = res else {
      return Err(LemmyErrorType::NotFound.into());
    };
    Ok(p)
  }

  pub async fn read_community_report(
    pool: &mut DbPool<'_>,
    report_id: CommunityReportId,
    my_person: &Person,
  ) -> LemmyResult<CommunityReportView> {
    let conn = &mut get_conn(pool).await?;
    let res = report_combined_joins(my_person.id, my_person.instance_id)
      .filter(report_combined::community_report_id.eq(report_id))
      .select(ReportCombinedViewInternal::as_select())
      .first(conn)
      .await?;

    let res = InternalToCombinedView::map_to_enum(res);
    let Some(ReportCombinedView::Community(c)) = res else {
      return Err(LemmyErrorType::NotFound.into());
    };
    Ok(c)
  }

  pub async fn read_private_message_report(
    pool: &mut DbPool<'_>,
    report_id: PrivateMessageReportId,
    my_person: &Person,
  ) -> LemmyResult<PrivateMessageReportView> {
    let conn = &mut get_conn(pool).await?;
    let res = report_combined_joins(my_person.id, my_person.instance_id)
      .filter(report_combined::private_message_report_id.eq(report_id))
      .select(ReportCombinedViewInternal::as_select())
      .first(conn)
      .await?;

    let res = InternalToCombinedView::map_to_enum(res);
    let Some(ReportCombinedView::PrivateMessage(pm)) = res else {
      return Err(LemmyErrorType::NotFound.into());
    };
    Ok(pm)
  }

  /// returns the current unresolved report count for the communities you mod
  pub async fn get_report_count(pool: &mut DbPool<'_>, user: &LocalUserView) -> LemmyResult<i64> {
    use diesel::dsl::count;

    let conn = &mut get_conn(pool).await?;

    let mut query = report_combined_joins(user.person.id, user.person.instance_id)
      .filter(report_is_not_resolved())
      .select(count(report_combined::id))
      .into_boxed();

    if user.local_user.admin {
      query = query.filter(filter_admin_reports(Utc::now() - Days::new(3)));
    } else {
      query = query.filter(filter_mod_reports());
    }

    query
      .first::<i64>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl PaginationCursorConversion for ReportCombinedView {
  type PaginatedType = ReportCombined;

  fn to_cursor(&self) -> CursorData {
    let (prefix, id) = match &self {
      ReportCombinedView::Comment(v) => ('C', v.comment_report.id.0),
      ReportCombinedView::Post(v) => ('P', v.post_report.id.0),
      ReportCombinedView::PrivateMessage(v) => ('M', v.private_message_report.id.0),
      ReportCombinedView::Community(v) => ('Y', v.community_report.id.0),
    };
    CursorData::new_with_prefix(prefix, id)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let conn = &mut get_conn(pool).await?;
    let (prefix, id) = cursor.id_and_prefix()?;

    let mut query = report_combined::table
      .select(Self::PaginatedType::as_select())
      .into_boxed();

    query = match prefix {
      'C' => query.filter(report_combined::comment_report_id.eq(id)),
      'P' => query.filter(report_combined::post_report_id.eq(id)),
      'M' => query.filter(report_combined::private_message_report_id.eq(id)),
      'Y' => query.filter(report_combined::community_report_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}

#[derive(Default)]
pub struct ReportCombinedQuery {
  pub type_: Option<ReportType>,
  pub post_id: Option<PostId>,
  pub community_id: Option<CommunityId>,
  pub unresolved_only: Option<bool>,
  /// For admins, also show reports with `violates_instance_rules=false`
  pub show_community_rule_violations: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub my_reports_only: Option<bool>,
  pub limit: Option<i64>,
}

impl ReportCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> LemmyResult<PagedResponse<ReportCombinedView>> {
    let limit = limit_fetch(self.limit, None)?;

    let report_creator = aliases::person1.field(person::id);

    let mut query = report_combined_joins(user.person.id, user.person.instance_id)
      .select(ReportCombinedViewInternal::as_select())
      .limit(limit)
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
      query = query.filter(report_creator.eq(user.person.id));
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
    let unresolved_only = self.unresolved_only.unwrap_or_default();
    let sort_direction = asc_if(unresolved_only);

    if unresolved_only {
      query = query.filter(report_is_not_resolved())
    };

    // Sorting by published
    let paginated_query =
      ReportCombinedView::paginate(query, &self.page_cursor, sort_direction, pool, None)
        .await?
        .then_order_by(key::published_at)
        // Tie breaker
        .then_order_by(key::id);

    let conn = &mut get_conn(pool).await?;
    let res = paginated_query
      .load::<ReportCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    paginate_response(out, limit, self.page_cursor)
  }
}

/// Mods can only see reports for posts/comments inside of communities where they are moderator,
/// and which have `violates_instance_rules == false`.
#[diesel::dsl::auto_type]
fn filter_mod_reports() -> _ {
  community_actions::became_moderator_at
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
    .or(report_combined::published_at.lt(interval))
    // Also show community reports where the admin is a community mod
    .or(community_actions::became_moderator_at.is_not_null())
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

#[diesel::dsl::auto_type]
fn report_is_not_resolved() -> _ {
  post_report::resolved
    .or(comment_report::resolved)
    .or(private_message_report::resolved)
    .or(community_report::resolved)
    .is_distinct_from(true)
}

impl InternalToCombinedView for ReportCombinedViewInternal {
  type CombinedView = ReportCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let (Some(post_report), Some(post), Some(community), Some(post_creator)) = (
      v.post_report,
      v.post.clone(),
      v.community.clone(),
      v.creator.clone(),
    ) {
      Some(ReportCombinedView::Post(PostReportView {
        post_report,
        post,
        community,
        post_creator,
        creator: v.report_creator,
        resolver: v.resolver,
        community_actions: v.community_actions,
        post_actions: v.post_actions,
        person_actions: v.person_actions,
        creator_is_admin: v.creator_is_admin,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned: v.creator_banned,
        creator_ban_expires_at: v.creator_ban_expires_at,
        creator_banned_from_community: v.creator_banned_from_community,
        creator_community_ban_expires_at: v.creator_community_ban_expires_at,
      }))
    } else if let (
      Some(comment_report),
      Some(comment),
      Some(post),
      Some(community),
      Some(comment_creator),
    ) = (
      v.comment_report,
      v.comment,
      v.post,
      v.community.clone(),
      v.creator.clone(),
    ) {
      Some(ReportCombinedView::Comment(CommentReportView {
        comment_report,
        comment,
        post,
        community,
        creator: v.report_creator,
        comment_creator,
        resolver: v.resolver,
        community_actions: v.community_actions,
        comment_actions: v.comment_actions,
        person_actions: v.person_actions,
        creator_is_admin: v.creator_is_admin,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned: v.creator_banned,
        creator_ban_expires_at: v.creator_ban_expires_at,
        creator_banned_from_community: v.creator_banned_from_community,
        creator_community_ban_expires_at: v.creator_community_ban_expires_at,
      }))
    } else if let (
      Some(private_message_report),
      Some(private_message),
      Some(private_message_creator),
    ) = (v.private_message_report, v.private_message, v.creator)
    {
      Some(ReportCombinedView::PrivateMessage(
        PrivateMessageReportView {
          private_message_report,
          private_message,
          creator: v.report_creator,
          private_message_creator,
          resolver: v.resolver,
          creator_is_admin: v.creator_is_admin,
          creator_banned: v.creator_banned,
          creator_ban_expires_at: v.creator_ban_expires_at,
        },
      ))
    } else if let (Some(community), Some(community_report)) = (v.community, v.community_report) {
      Some(ReportCombinedView::Community(CommunityReportView {
        community_report,
        community,
        creator: v.report_creator,
        resolver: v.resolver,
        creator_is_admin: v.creator_is_admin,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned: v.creator_banned,
        creator_ban_expires_at: v.creator_ban_expires_at,
        creator_banned_from_community: v.creator_banned_from_community,
        creator_community_ban_expires_at: v.creator_community_ban_expires_at,
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
    LocalUserView,
    ReportCombinedView,
    ReportCombinedViewInternal,
    impls::ReportCombinedQuery,
  };
  use chrono::{Days, Utc};
  use diesel::{ExpressionMethods, QueryDsl, update};
  use diesel_async::RunQueryDsl;
  use lemmy_db_schema::{
    ReportType,
    assert_length,
    source::{
      comment::{Comment, CommentInsertForm},
      comment_report::{CommentReport, CommentReportForm},
      community::{Community, CommunityActions, CommunityInsertForm, CommunityModeratorForm},
      community_report::{CommunityReport, CommunityReportForm},
      instance::{Instance, InstanceActions, InstanceBanForm},
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
      post_report::{PostReport, PostReportForm},
      private_message::{PrivateMessage, PrivateMessageInsertForm},
      private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
    },
    traits::{Bannable, Reportable},
  };
  use lemmy_db_schema_file::schema::report_combined;
  use lemmy_diesel_utils::{
    connection::{DbPool, build_db_pool_for_tests, get_conn},
    traits::Crud,
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
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld").await?;

    let timmy_form = PersonInsertForm::test_form(inserted_instance.id, "timmy_rcv");
    let inserted_timmy = Person::create(pool, &timmy_form).await?;
    let timmy_local_user_form = LocalUserInsertForm::test_form(inserted_timmy.id);
    let timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;
    let timmy_view = LocalUserView {
      local_user: timmy_local_user,
      person: inserted_timmy.clone(),
      banned: false,
      ban_expires_at: None,
    };

    // Make an admin, to be able to see private message reports.
    let admin_form = PersonInsertForm::test_form(inserted_instance.id, "admin_rcv");
    let inserted_admin = Person::create(pool, &admin_form).await?;
    let admin_local_user_form = LocalUserInsertForm::test_form_admin(inserted_admin.id);
    let admin_local_user = LocalUser::create(pool, &admin_local_user_form, vec![]).await?;
    let admin_view = LocalUserView {
      local_user: admin_local_user,
      person: inserted_admin.clone(),
      banned: false,
      ban_expires_at: None,
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
    let timmy_moderator_form =
      CommunityModeratorForm::new(inserted_community.id, inserted_timmy.id);
    CommunityActions::join(pool, &timmy_moderator_form).await?;

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
      original_community_summary: None,
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
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view).await?;
    assert_eq!(2, report_count_mod);
    let report_count_admin =
      ReportCombinedViewInternal::get_report_count(pool, &data.admin_view).await?;
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
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view).await?;
    assert_eq!(2, report_count_timmy);

    // Resolve the post report
    PostReport::update_resolved(pool, inserted_post_report.id, data.timmy.id, true).await?;

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
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view).await?;
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
    PrivateMessageReport::update_resolved(pool, pm_report.id, data.admin_view.person.id, true)
      .await?;

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
      ReportCombinedViewInternal::read_post_report(pool, inserted_jessica_report.id, &data.timmy)
        .await?;

    // Make sure the triggers are reading the aggregates correctly.
    let agg_1 = Post::read(pool, data.post.id).await?;
    let agg_2 = Post::read(pool, data.post_2.id).await?;

    assert_eq!(
      read_jessica_report_view.post_report,
      inserted_jessica_report
    );
    assert_eq!(read_jessica_report_view.post.id, data.post_2.id);
    assert_eq!(read_jessica_report_view.community.id, data.community.id);
    assert_eq!(read_jessica_report_view.creator.id, data.jessica.id);
    assert_eq!(read_jessica_report_view.post_creator.id, data.timmy.id);
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
    let report_count = ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view).await?;
    assert_eq!(2, report_count);

    // Pretend the post was removed, and resolve all reports for that object.
    // This is called manually in the API for post removals
    PostReport::resolve_all_for_object(pool, inserted_jessica_report.post_id, data.timmy.id)
      .await?;

    let read_jessica_report_view_after_resolve =
      ReportCombinedViewInternal::read_post_report(pool, inserted_jessica_report.id, &data.timmy)
        .await?;
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
    let agg_2 = Post::read(pool, data.post_2.id).await?;
    assert_eq!(agg_2.report_count, 1);
    assert_eq!(agg_2.unresolved_report_count, 0);

    // Make sure the other unresolved report isn't changed
    let agg_1 = Post::read(pool, data.post.id).await?;
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
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view).await?;
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

    let comment = Comment::read(pool, data.comment.id).await?;
    assert_eq!(comment.report_count, 2);

    let read_jessica_report_view = ReportCombinedViewInternal::read_comment_report(
      pool,
      inserted_jessica_report.id,
      &data.timmy,
    )
    .await?;
    assert_eq!(read_jessica_report_view.comment.unresolved_report_count, 2);

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
    let report_count = ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view).await?;
    assert_eq!(2, report_count);

    // Resolve the report
    CommentReport::update_resolved(pool, inserted_jessica_report.id, data.timmy.id, true).await?;
    let read_jessica_report_view_after_resolve = ReportCombinedViewInternal::read_comment_report(
      pool,
      inserted_jessica_report.id,
      &data.timmy,
    )
    .await?;

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
      ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view).await?;
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
      original_community_summary: None,
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
      let read_report = ReportCombinedViewInternal::read_community_report(
        pool,
        community_report.id,
        &data.admin_view.person,
      )
      .await?;
      assert_eq!(&read_report, v);
    } else {
      panic!("wrong type");
    }

    // admin resolves the report (after taking appropriate action)
    CommunityReport::update_resolved(pool, community_report.id, data.admin_view.person.id, true)
      .await?;

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
    let count = ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view).await?;
    assert_eq!(0, count);

    // only admin can see the report
    let admin_reports = ReportCombinedQuery::default()
      .list(pool, &data.admin_view)
      .await?;
    assert_length!(1, admin_reports);
    let count = ReportCombinedViewInternal::get_report_count(pool, &data.admin_view).await?;
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
    let count = ReportCombinedViewInternal::get_report_count(pool, &data.timmy_view).await?;
    assert_eq!(1, count);

    // but not the admin
    let admin_reports = ReportCombinedQuery::default()
      .list(pool, &data.admin_view)
      .await?;
    assert_length!(0, admin_reports);
    let count = ReportCombinedViewInternal::get_report_count(pool, &data.admin_view).await?;
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
    .set(report_combined::published_at.eq(Utc::now() - Days::new(3)))
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

    let agg = Comment::read(pool, data.comment.id).await?;
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

  #[tokio::test]
  #[serial]
  async fn ensure_creator_data_is_correct() -> LemmyResult<()> {
    // The creator_banned and other creator_data should be the content creator, not the report
    // creator.

    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // sara reports timmys post
    let sara_report_form = PostReportForm {
      creator_id: data.sara.id,
      post_id: data.post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from sara".into(),
      violates_instance_rules: false,
    };
    let inserted_sara_report = PostReport::report(pool, &sara_report_form).await?;

    // Admin ban timmy (the post creator)
    let ban_timmy_form = InstanceBanForm::new(data.timmy.id, data.instance.id, None);
    InstanceActions::ban(pool, &ban_timmy_form).await?;

    let read_sara_report_view =
      ReportCombinedViewInternal::read_post_report(pool, inserted_sara_report.id, &data.timmy)
        .await?;

    // Make sure timmy is seen as banned.
    assert_eq!(read_sara_report_view.creator_banned, true);

    cleanup(data, pool).await?;

    Ok(())
  }
}
