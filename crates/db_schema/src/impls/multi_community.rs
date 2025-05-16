use crate::{
  diesel::NullableExpressionMethods,
  newtypes::{CommunityId, DbUrl, MultiCommunityId, PersonId},
  source::multi_community::{
    MultiCommunity,
    MultiCommunityInsertForm,
    MultiCommunityView,
    MultiCommunityViewApub,
  },
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{delete, insert_into, sql},
  result::Error,
  sql_types::{Array, Integer, Text},
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::{community, multi_community, multi_community_entry, person};

pub enum ReadParams {
  Id(MultiCommunityId),
  ApId(DbUrl),
}

impl MultiCommunity {
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &MultiCommunityInsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(multi_community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    new_communities: &Vec<CommunityId>,
  ) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    conn
      .transaction::<_, Error, _>(|conn| {
        async move {
          delete(multi_community_entry::table)
            .filter(multi_community_entry::multi_community_id.eq(id))
            .filter(multi_community_entry::community_id.ne_all(new_communities))
            .execute(conn)
            .await?;
          let forms = new_communities
            .into_iter()
            .map(|k| {
              (
                multi_community_entry::multi_community_id.eq(id),
                multi_community_entry::community_id.eq(k),
              )
            })
            .collect::<Vec<_>>();
          insert_into(multi_community_entry::table)
            .values(forms)
            .on_conflict_do_nothing()
            .execute(conn)
            .await
        }
        .scope_boxed()
      })
      .await?;
    Ok(())
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    params: ReadParams,
  ) -> Result<MultiCommunityView, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = multi_community_entry::table
      .left_join(multi_community::table.left_join(person::table))
      .filter(multi_community::id.is_not_null())
      .group_by(multi_community::id)
      .select((
        multi_community::all_columns.assume_not_null(),
        sql::<Array<Integer>>("array_agg(multi_community_entry.community_id)"),
      ))
      .into_boxed();

    query = match params {
      ReadParams::Id(id) => query.filter(multi_community::id.eq(id)),
      ReadParams::ApId(ap_id) => query.filter(multi_community::ap_id.eq(ap_id)),
    };
    let (multi, entries) = query.first(conn).await?;
    Ok(MultiCommunityView { multi, entries })
  }

  pub async fn read_apub(
    pool: &mut DbPool<'_>,
    user_name: &str,
    multi_name: &str,
  ) -> Result<MultiCommunityViewApub, Error> {
    let conn = &mut get_conn(pool).await?;
    let (multi, entries) = multi_community_entry::table
      .inner_join(community::table)
      .left_join(multi_community::table.left_join(person::table))
      .filter(multi_community::id.is_not_null())
      .group_by(multi_community::id)
      .select((
        multi_community::all_columns.assume_not_null(),
        sql::<Array<Text>>("array_agg(community.ap_id)"),
      ))
      .filter(person::name.eq(user_name))
      .filter(multi_community::name.eq(multi_name))
      .first(conn)
      .await?;
    Ok(MultiCommunityViewApub { multi, entries })
  }

  pub async fn list(pool: &mut DbPool<'_>, owner_id: Option<PersonId>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = multi_community::table.into_boxed();
    if let Some(owner_id) = owner_id {
      query = query.filter(multi_community::owner_id.eq(owner_id));
    }
    query.get_results(conn).await
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

    let form = MultiCommunityInsertForm {
      owner_id: bobby.id,
      name: "multi".to_string(),
      ap_id: DbUrl(Box::new("http://example.com".parse()?)),
    };
    let multi_create = MultiCommunity::create(pool, &form).await?;
    assert_eq!(form.owner_id, multi_create.owner_id);
    assert_eq!(form.name, multi_create.name);
    assert_eq!(form.ap_id, multi_create.ap_id);

    let multi_read_empty = MultiCommunity::read(pool, ReadParams::Id(multi_create.id)).await?;
    assert_eq!(multi_read_empty.multi.owner_id, multi_create.owner_id);
    assert!(multi_read_empty.entries.is_empty());

    let multi_entries = vec![community.id];
    MultiCommunity::update(pool, multi_create.id, &multi_entries).await?;

    let multi_read = MultiCommunity::read(pool, ReadParams::Id(multi_create.id)).await?;
    assert_eq!(multi_read.multi.owner_id, multi_create.owner_id);
    assert_eq!(multi_entries, multi_read.entries);

    let multi_read_apub = MultiCommunity::read_apub(pool, &bobby.name, &multi_create.name).await?;
    assert_eq!(multi_read.multi.owner_id, multi_create.owner_id);
    assert_eq!(vec![community.ap_id], multi_read_apub.entries);

    Ok(())
  }
}
