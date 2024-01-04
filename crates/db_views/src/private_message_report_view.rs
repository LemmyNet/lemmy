use crate::structs::PrivateMessageReportView;
use diesel::{
  pg::Pg,
  result::Error,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  newtypes::PrivateMessageReportId,
  schema::{person, private_message, private_message_report},
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
};

fn queries<'a>() -> Queries<
  impl ReadFn<'a, PrivateMessageReportView, PrivateMessageReportId>,
  impl ListFn<'a, PrivateMessageReportView, PrivateMessageReportQuery>,
> {
  let all_joins =
    |query: private_message_report::BoxedQuery<'a, Pg>| {
      query
        .inner_join(private_message::table)
        .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
        .inner_join(
          aliases::person1
            .on(private_message_report::creator_id.eq(aliases::person1.field(person::id))),
        )
        .left_join(aliases::person2.on(
          private_message_report::resolver_id.eq(aliases::person2.field(person::id).nullable()),
        ))
        .select((
          private_message_report::all_columns,
          private_message::all_columns,
          person::all_columns,
          aliases::person1.fields(person::all_columns),
          aliases::person2.fields(person::all_columns).nullable(),
        ))
    };

  let read = move |mut conn: DbConn<'a>, report_id: PrivateMessageReportId| async move {
    all_joins(private_message_report::table.find(report_id).into_boxed())
      .first::<PrivateMessageReportView>(&mut conn)
      .await
  };

  let list = move |mut conn: DbConn<'a>, options: PrivateMessageReportQuery| async move {
    let mut query = all_joins(private_message_report::table.into_boxed());

    if options.unresolved_only {
      query = query.filter(private_message_report::resolved.eq(false));
    }

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query
      .order_by(private_message::published.asc())
      .limit(limit)
      .offset(offset)
      .load::<PrivateMessageReportView>(&mut conn)
      .await
  };

  Queries::new(read, list)
}

impl PrivateMessageReportView {
  /// returns the PrivateMessageReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: PrivateMessageReportId,
  ) -> Result<Self, Error> {
    queries().read(pool, report_id).await
  }

  /// Returns the current unresolved post report count for the communities you mod
  pub async fn get_report_count(pool: &mut DbPool<'_>) -> Result<i64, Error> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

    private_message_report::table
      .inner_join(private_message::table)
      .filter(private_message_report::resolved.eq(false))
      .into_boxed()
      .select(count(private_message_report::id))
      .first::<i64>(conn)
      .await
  }
}

#[derive(Default)]
pub struct PrivateMessageReportQuery {
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unresolved_only: bool,
}

impl PrivateMessageReportQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<PrivateMessageReportView>, Error> {
    queries().list(pool, self).await
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::private_message_report_view::PrivateMessageReportQuery;
  use lemmy_db_schema::{
    assert_length,
    source::{
      instance::Instance,
      person::{Person, PersonInsertForm},
      private_message::{PrivateMessage, PrivateMessageInsertForm},
      private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
    },
    traits::{Crud, Reportable},
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person_1 = PersonInsertForm::builder()
      .name("timmy_mrv".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let inserted_timmy = Person::create(pool, &new_person_1).await.unwrap();

    let new_person_2 = PersonInsertForm::builder()
      .name("jessica_mrv".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let inserted_jessica = Person::create(pool, &new_person_2).await.unwrap();

    // timmy sends private message to jessica
    let pm_form = PrivateMessageInsertForm::builder()
      .creator_id(inserted_timmy.id)
      .recipient_id(inserted_jessica.id)
      .content("something offensive".to_string())
      .build();
    let pm = PrivateMessage::create(pool, &pm_form).await.unwrap();

    // jessica reports private message
    let pm_report_form = PrivateMessageReportForm {
      creator_id: inserted_jessica.id,
      original_pm_text: pm.content.clone(),
      private_message_id: pm.id,
      reason: "its offensive".to_string(),
    };
    let pm_report = PrivateMessageReport::report(pool, &pm_report_form)
      .await
      .unwrap();

    let reports = PrivateMessageReportQuery::default()
      .list(pool)
      .await
      .unwrap();
    assert_length!(1, reports);
    assert!(!reports[0].private_message_report.resolved);
    assert_eq!(inserted_timmy.name, reports[0].private_message_creator.name);
    assert_eq!(inserted_jessica.name, reports[0].creator.name);
    assert_eq!(pm_report.reason, reports[0].private_message_report.reason);
    assert_eq!(pm.content, reports[0].private_message.content);

    let new_person_3 = PersonInsertForm::builder()
      .name("admin_mrv".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let inserted_admin = Person::create(pool, &new_person_3).await.unwrap();

    // admin resolves the report (after taking appropriate action)
    PrivateMessageReport::resolve(pool, pm_report.id, inserted_admin.id)
      .await
      .unwrap();

    let reports = PrivateMessageReportQuery {
      unresolved_only: (false),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_length!(1, reports);
    assert!(reports[0].private_message_report.resolved);
    assert!(reports[0].resolver.is_some());
    assert_eq!(
      inserted_admin.name,
      reports[0].resolver.as_ref().unwrap().name
    );

    Instance::delete(pool, inserted_instance.id).await.unwrap();
  }
}
