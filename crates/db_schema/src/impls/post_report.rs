use crate::{
  diesel::{
    query_dsl::positional_order_dsl::{IntoOrderColumn, OrderColumn, PositionalOrderDsl},
    JoinOnDsl,
    NullableExpressionMethods,
  },
  newtypes::{CommentReportId, PersonId, PostId, PostReportId},
  schema::{comment_report, post, post_report},
  source::{
    comment_report::CommentReport,
    post::Post,
    post_report::{PostReport, PostReportForm},
  },
  traits::Reportable,
  utils::{get_conn, DbPool},
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{insert_into, sql, update},
  result::Error,
  sql_types::Integer,
  CombineDsl,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ts_rs::TS;

#[async_trait]
impl Reportable for PostReport {
  type Form = PostReportForm;
  type IdType = PostReportId;
  type ObjectIdType = PostId;

  async fn report(pool: &mut DbPool<'_>, post_report_form: &PostReportForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_report::table)
      .values(post_report_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn resolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    update(post_report::table.find(report_id))
      .set((
        post_report::resolved.eq(true),
        post_report::resolver_id.eq(by_resolver_id),
        post_report::updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
  }

  async fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    post_id_: PostId,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    update(post_report::table.filter(post_report::post_id.eq(post_id_)))
      .set((
        post_report::resolved.eq(true),
        post_report::resolver_id.eq(by_resolver_id),
        post_report::updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
  }

  async fn unresolve(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    update(post_report::table.find(report_id))
      .set((
        post_report::resolved.eq(false),
        post_report::resolver_id.eq(by_resolver_id),
        post_report::updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
  }
}

enum PostOrCommentReport {
  Post(PostReport),
  Comment(CommentReport),
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS, Queryable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PostOrCommentTest {
  published: DateTime<Utc>,
  post_report_id: Option<PostReport>,
  comment_report_id: Option<CommentReport>,
  post: Option<Post>,
}

// type PostOrCommentTestType = (DateTime<Utc>, Option<PostReportId>, Option<CommentReportId>);

pub async fn test_post_or_comment_report(
  pool: &mut DbPool<'_>,
) -> Result<Vec<PostOrCommentTest>, Error> {
  let conn = &mut get_conn(pool).await?;

  let post_report_query = post_report::table
    .select((
      post_report::published,
      post_report::id.nullable(),
      sql::<Integer>("null").nullable(),
    ))
    .inner_join(post::table)
    .order_by(post_report::published.desc())
    .limit(20);
  let comment_report_query = comment_report::table
    .select((
      comment_report::published,
      sql::<Integer>("null").nullable(),
      comment_report::id.nullable(),
    ))
    .order_by(comment_report::published.desc())
    .limit(20);

  let combined = post_report_query
    .union_all(comment_report_query)
    .left_join(post::table.on(post_report::post_id.eq(post::id)));

  // .positional_order_by(OrderColumn::from(1).desc());
  // TODO waiting on diesel release for this one
  // .limit(20);
  // let query = combined
  // .left_join(post::table.on(post_report::post_id.eq(post::id)));
  // let query = combined;

  let res = query.load::<PostOrCommentTest>(conn).await;
  res
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::{
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use diesel::result::Error;
  use serial_test::serial;

  async fn init(pool: &mut DbPool<'_>) -> Result<(Person, PostReport), Error> {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;
    let person_form = PersonInsertForm::test_form(inserted_instance.id, "jim");
    let person = Person::create(pool, &person_form).await?;

    let community_form = CommunityInsertForm::new(
      inserted_instance.id,
      "test community_4".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;

    let form = PostInsertForm::new("A test post".into(), person.id, community.id);
    let post = Post::create(pool, &form).await?;

    let report_form = PostReportForm {
      post_id: post.id,
      creator_id: person.id,
      reason: "my reason".to_string(),
      ..Default::default()
    };
    let report = PostReport::report(pool, &report_form).await?;

    Ok((person, report))
  }

  #[tokio::test]
  #[serial]
  async fn test_resolve_post_report() -> Result<(), Error> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (person, report) = init(pool).await?;

    let resolved_count = PostReport::resolve(pool, report.id, person.id).await?;
    assert_eq!(resolved_count, 1);

    let unresolved_count = PostReport::unresolve(pool, report.id, person.id).await?;
    assert_eq!(unresolved_count, 1);

    Person::delete(pool, person.id).await?;
    Post::delete(pool, report.post_id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_resolve_all_post_reports() -> Result<(), Error> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (person, report) = init(pool).await?;

    let resolved_count =
      PostReport::resolve_all_for_object(pool, report.post_id, person.id).await?;
    assert_eq!(resolved_count, 1);

    Person::delete(pool, person.id).await?;
    Post::delete(pool, report.post_id).await?;

    Ok(())
  }
}
