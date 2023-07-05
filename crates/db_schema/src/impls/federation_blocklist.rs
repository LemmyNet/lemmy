use crate::{
  schema::federation_blocklist,
  source::{
    federation_blocklist::{FederationBlockList, FederationBlockListForm},
    instance::Instance,
  },
  utils::GetConn,
};
use diesel::{dsl::insert_into, result::Error};
use lemmy_db_schema::utils::RunQueryDsl;

impl FederationBlockList {
  pub async fn replace(mut conn: impl GetConn, list_opt: Option<Vec<String>>) -> Result<(), Error> {
    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          if let Some(list) = list_opt {
            Self::clear(conn).await?;

            for domain in list {
              // Upsert all of these as instances
              let instance = Instance::read_or_create_with_conn(conn, domain).await?;

              let form = FederationBlockListForm {
                instance_id: instance.id,
                updated: None,
              };
              insert_into(federation_blocklist::table)
                .values(form)
                .get_result::<Self>(conn)
                .await?;
            }
            Ok(())
          } else {
            Ok(())
          }
        }) as _
      })
      .await
  }

  async fn clear(mut conn: impl GetConn) -> Result<usize, Error> {
    diesel::delete(federation_blocklist::table)
      .execute(conn)
      .await
  }
}
