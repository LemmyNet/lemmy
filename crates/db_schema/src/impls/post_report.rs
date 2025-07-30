use crate::{
  newtypes::{PersonId, PostId, PostReportId},
  source::post_report::{PostReport, PostReportForm},
  traits::Reportable,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{insert_into, update},
  BoolExpressionMethods,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::post_report;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Reportable for PostReport {
  type Form = PostReportForm;
  type IdType = PostReportId;
  type ObjectIdType = PostId;

  async fn report(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_report::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update_resolved(
    pool: &mut DbPool<'_>,
    report_id: Self::IdType,
    by_resolver_id: PersonId,
    is_resolved: bool,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(post_report::table.find(report_id))
      .set((
        post_report::resolved.eq(is_resolved),
        post_report::resolver_id.eq(by_resolver_id),
        post_report::updated_at.eq(Utc::now()),
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
      post_report::table.filter(
        post_report::post_id
          .eq(object_id)
          .and(post_report::creator_id.eq(report_creator_id)),
      ),
    )
    .set((
      post_report::resolved.eq(true),
      post_report::resolver_id.eq(resolver_id),
      post_report::updated_at.eq(Utc::now()),
    ))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  async fn resolve_all_for_object(
    pool: &mut DbPool<'_>,
    post_id_: PostId,
    by_resolver_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(post_report::table.filter(post_report::post_id.eq(post_id_)))
      .set((
        post_report::resolved.eq(true),
        post_report::resolver_id.eq(by_resolver_id),
        post_report::updated_at.eq(Utc::now()),
      ))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
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
  use serial_test::serial;

  async fn init(pool: &mut DbPool<'_>) -> LemmyResult<(Person, PostReport)> {
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
  async fn test_resolve_post_report() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (person, report) = init(pool).await?;

    let resolved_count = PostReport::update_resolved(pool, report.id, person.id, true).await?;
    assert_eq!(resolved_count, 1);

    let unresolved_count = PostReport::update_resolved(pool, report.id, person.id, false).await?;
    assert_eq!(unresolved_count, 1);

    Person::delete(pool, person.id).await?;
    Post::delete(pool, report.post_id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_resolve_all_post_reports() -> LemmyResult<()> {
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
