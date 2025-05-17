use crate::{
  newtypes::{PersonId, PostId, PostReportId},
  schema::post_report::{
    dsl::{post_report, resolved, resolver_id, updated},
    post_id,
  },
  source::post_report::{PostReport, PostReportForm},
  traits::Reportable,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{
  dsl::{insert_into, update},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Reportable for PostReport {
  type Form = PostReportForm;
  type IdType = PostReportId;
  type ObjectIdType = PostId;

  async fn report(pool: &mut DbPool<'_>, post_report_form: &PostReportForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_report)
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
    update(post_report.find(report_id))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(Utc::now()),
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
    update(post_report.filter(post_id.eq(post_id_)))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(Utc::now()),
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
    update(post_report.find(report_id))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(Utc::now()),
      ))
      .execute(conn)
      .await
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
