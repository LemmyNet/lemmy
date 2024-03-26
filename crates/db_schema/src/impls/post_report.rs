use crate::{
  newtypes::{PersonId, PostId, PostReportId},
  schema::post_report::{
    dsl::{post_report, resolved, resolver_id, updated},
    post_id,
  },
  source::post_report::{PostReport, PostReportForm},
  traits::Reportable,
  utils::{get_conn, naive_now, DbPool},
};
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
        updated.eq(naive_now()),
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
        updated.eq(naive_now()),
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
        updated.eq(naive_now()),
      ))
      .execute(conn)
      .await
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
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

  async fn init(pool: &mut DbPool<'_>) -> (Person, PostReport) {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();
    let person_form = PersonInsertForm::builder()
      .name("jim".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let person = Person::create(pool, &person_form).await.unwrap();

    let community_form = CommunityInsertForm::builder()
      .name("test community_4".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let community = Community::create(pool, &community_form).await.unwrap();

    let form = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(person.id)
      .community_id(community.id)
      .build();
    let post = Post::create(pool, &form).await.unwrap();

    let report_form = PostReportForm {
      post_id: post.id,
      creator_id: person.id,
      reason: "my reason".to_string(),
      ..Default::default()
    };
    let report = PostReport::report(pool, &report_form).await.unwrap();
    (person, report)
  }

  #[tokio::test]
  #[serial]
  async fn test_resolve_post_report() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let (person, report) = init(pool).await;

    let resolved_count = PostReport::resolve(pool, report.id, person.id)
      .await
      .unwrap();
    assert_eq!(resolved_count, 1);

    let unresolved_count = PostReport::unresolve(pool, report.id, person.id)
      .await
      .unwrap();
    assert_eq!(unresolved_count, 1);

    Person::delete(pool, person.id).await.unwrap();
    Post::delete(pool, report.post_id).await.unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn test_resolve_all_post_reports() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let (person, report) = init(pool).await;

    let resolved_count = PostReport::resolve_all_for_object(pool, report.post_id, person.id)
      .await
      .unwrap();
    assert_eq!(resolved_count, 1);

    Person::delete(pool, person.id).await.unwrap();
    Post::delete(pool, report.post_id).await.unwrap();
  }
}
