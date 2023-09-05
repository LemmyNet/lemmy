use crate::{
    diesel::OptionalExtension,
    newtypes::DbUrl,
    source::activity::{ReceivedActivity, SentActivity, SentActivityForm},
    utils::{get_conn, DbPool},
};
use diesel::{
    dsl::insert_into,
    result::{DatabaseErrorKind, Error, Error::DatabaseError},
    ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;

impl SentActivity {
    pub async fn create(pool: &mut DbPool<'_>, form: SentActivityForm) -> Result<Self, Error> {
        use crate::schema::sent_activity::dsl::sent_activity;
        let conn = &mut get_conn(pool).await?;
        insert_into(sent_activity)
            .values(form)
            .get_result::<Self>(conn)
            .await
    }

    pub async fn read_from_apub_id(
        pool: &mut DbPool<'_>,
        object_id: &DbUrl,
    ) -> Result<Self, Error> {
        use crate::schema::sent_activity::dsl::{ap_id, sent_activity};
        let conn = &mut get_conn(pool).await?;
        sent_activity
            .filter(ap_id.eq(object_id))
            .first::<Self>(conn)
            .await
    }
}

impl ReceivedActivity {
    pub async fn create(pool: &mut DbPool<'_>, ap_id_: &DbUrl) -> Result<(), Error> {
        use crate::schema::received_activity::dsl::{ap_id, id, received_activity};
        let conn = &mut get_conn(pool).await?;
        let res = insert_into(received_activity)
            .values(ap_id.eq(ap_id_))
            .on_conflict_do_nothing()
            .returning(id)
            .get_result::<i64>(conn)
            .await
            .optional()?;
        if res.is_some() {
            // new activity inserted successfully
            Ok(())
        } else {
            // duplicate activity
            Err(DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                Box::<String>::default(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::indexing_slicing)]

    use super::*;
    use crate::utils::build_db_pool_for_tests;
    use serde_json::json;
    use serial_test::serial;
    use url::Url;

    #[tokio::test]
    #[serial]
    async fn receive_activity_duplicate() {
        let pool = &build_db_pool_for_tests().await;
        let pool = &mut pool.into();
        let ap_id: DbUrl = Url::parse("http://example.com/activity/531")
            .unwrap()
            .into();

        // inserting activity for first time
        let res = ReceivedActivity::create(pool, &ap_id).await;
        assert!(res.is_ok());

        let res = ReceivedActivity::create(pool, &ap_id).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn sent_activity_write_read() {
        let pool = &build_db_pool_for_tests().await;
        let pool = &mut pool.into();
        let ap_id: DbUrl = Url::parse("http://example.com/activity/412")
            .unwrap()
            .into();
        let data = json!({
            "key1": "0xF9BA143B95FF6D82",
            "key2": "42",
        });
        let sensitive = false;

        let form = SentActivityForm {
            ap_id: ap_id.clone(),
            data: data.clone(),
            sensitive,
        };

        SentActivity::create(pool, form).await.unwrap();

        let res = SentActivity::read_from_apub_id(pool, &ap_id).await.unwrap();
        assert_eq!(res.ap_id, ap_id);
        assert_eq!(res.data, data);
        assert_eq!(res.sensitive, sensitive);
    }
}
