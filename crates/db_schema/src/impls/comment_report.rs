use crate::{
  newtypes::{CommentId, CommentReportId, PostId},
  source::comment_report::{CommentReport, CommentReportForm},
  traits::Reportable,
};
use chrono::Utc;
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
  dsl::{insert_into, update},
};
use diesel_async::RunQueryDsl;
use diesel_ltree::{Ltree, LtreeExtensions};
use lemmy_db_schema_file::{
  PersonId,
  schema::{comment, comment_report},
};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Reportable for CommentReport {
  type Form = CommentReportForm;
  type IdType = CommentReportId;
  type ObjectIdType = CommentId;
  /// creates a comment report and returns it
  ///
  /// * `conn` - the postgres connection
  /// * `comment_report_form` - the filled CommentReportForm to insert
  async fn report(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(comment_report::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  /// resolve a comment report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to resolve
  /// * `by_resolver_id` - the id of the user resolving the report
  async fn update_resolved(
    pool: &mut DbPool<'_>,
    report_id_: Self::IdType,
    by_resolver_id: PersonId,
    is_resolved: bool,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(comment_report::table.find(report_id_))
      .set((
        comment_report::resolved.eq(is_resolved),
        comment_report::resolver_id.eq(by_resolver_id),
        comment_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  async fn resolve_apub(
    pool: &mut DbPool<'_>,
    object_id: Self::ObjectIdType,
    report_creator_id: PersonId,
    resolver_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(
      comment_report::table.filter(
        comment_report::comment_id
          .eq(object_id)
          .and(comment_report::creator_id.eq(report_creator_id)),
      ),
    )
    .set((
      comment_report::resolved.eq(true),
      comment_report::resolver_id.eq(resolver_id),
      comment_report::updated_at.eq(Utc::now()),
    ))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  async fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    comment_id_: CommentId,
    by_resolver_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(comment_report::table.filter(comment_report::comment_id.eq(comment_id_)))
      .set((
        comment_report::resolved.eq(true),
        comment_report::resolver_id.eq(by_resolver_id),
        comment_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl CommentReport {
  pub async fn resolve_all_for_thread(
    pool: &mut DbPool<'_>,
    comment_path: &Ltree,
    by_resolver_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    let report_alias = diesel::alias!(comment_report as cr);
    let report_subquery = report_alias
      .inner_join(comment::table.on(comment::id.eq(report_alias.field(comment_report::comment_id))))
      .filter(comment::path.contained_by(comment_path));
    update(comment_report::table.filter(
      comment_report::id.eq_any(report_subquery.select(report_alias.field(comment_report::id))),
    ))
    .set((
      comment_report::resolved.eq(true),
      comment_report::resolver_id.eq(by_resolver_id),
      comment_report::updated_at.eq(Utc::now()),
    ))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn resolve_all_for_post(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    by_resolver_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    let report_alias = diesel::alias!(comment_report as cr);
    let report_subquery = report_alias
      .inner_join(comment::table.on(comment::id.eq(report_alias.field(comment_report::comment_id))))
      .filter(comment::post_id.eq(post_id));
    update(comment_report::table.filter(
      comment_report::id.eq_any(report_subquery.select(report_alias.field(comment_report::id))),
    ))
    .set((
      comment_report::resolved.eq(true),
      comment_report::resolver_id.eq(by_resolver_id),
      comment_report::updated_at.eq(Utc::now()),
    ))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}
