use crate::MultiCommunityViewApub;
use diesel::{
  dsl::sql,
  result::Error,
  sql_types::{Array, Text},
  BoolExpressionMethods,
  ExpressionMethods,
  PgExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::utils::{get_conn, DbPool};
use lemmy_db_schema_file::schema::{community, multi_community, multi_community_entry, person};

impl MultiCommunityViewApub {
  pub async fn read_local(
    pool: &mut DbPool<'_>,
    user_name: &str,
    multi_name: &str,
  ) -> Result<MultiCommunityViewApub, Error> {
    let conn = &mut get_conn(pool).await?;
    let (multi, entries) = multi_community::table
      .left_join(person::table)
      .left_join(multi_community_entry::table.inner_join(community::table))
      .group_by(multi_community::id)
      .filter(
        community::removed
          .or(community::deleted)
          .is_distinct_from(true),
      )
      .filter(person::name.eq(user_name))
      .filter(person::local)
      .filter(multi_community::name.eq(multi_name))
      .select((
        multi_community::all_columns,
        // Get vec of community.ap_id. If no row exists for multi_community_entry this returns
        // [null] so we need to filter that with array_remove.
        sql::<Array<Text>>("array_remove(array_agg(community.ap_id), null)"),
      ))
      .first(conn)
      .await?;
    Ok(MultiCommunityViewApub { multi, entries })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      multi_community::{MultiCommunity, MultiCommunityInsertForm},
      person::{Person, PersonInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_multi_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let form = PersonInsertForm::test_form(instance.id, "bobby");
    let bobby = Person::create(pool, &form).await?;

    let form = CommunityInsertForm::new(
      instance.id,
      "TIL".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &form).await?;

    let form =
      MultiCommunityInsertForm::new(bobby.id, "multi".to_string(), community.ap_id.clone());
    let multi_create = MultiCommunity::create(pool, &form).await?;
    assert_eq!(form.creator_id, multi_create.creator_id);
    assert_eq!(form.name, multi_create.name);
    assert_eq!(form.ap_id, multi_create.ap_id);

    let multi_read_apub_empty =
      MultiCommunityViewApub::read_local(pool, &bobby.name, &multi_create.name).await?;
    assert!(multi_read_apub_empty.entries.is_empty());

    let multi_entries = vec![community.id];
    let conn = &mut get_conn(pool).await?;
    MultiCommunity::update_entries(conn, multi_create.id, &multi_entries).await?;

    let multi_read_apub =
      MultiCommunityViewApub::read_local(pool, &bobby.name, &multi_create.name).await?;
    assert_eq!(multi_read_apub.multi.creator_id, multi_create.creator_id);
    assert_eq!(vec![community.ap_id], multi_read_apub.entries);

    let list = MultiCommunity::list(pool, None).await?;
    assert_eq!(1, list.len());

    Instance::delete(pool, instance.id).await?;

    Ok(())
  }
}
