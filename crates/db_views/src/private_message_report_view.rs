use crate::structs::PrivateMessageReportView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PrivateMessageReportId,
  schema::{person, private_message, private_message_report},
  source::{
    person::{Person, PersonSafe},
    private_message::PrivateMessage,
    private_message_report::PrivateMessageReport,
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};
use typed_builder::TypedBuilder;

type PrivateMessageReportViewTuple = (
  PrivateMessageReport,
  PrivateMessage,
  PersonSafe,
  PersonSafe,
  Option<PersonSafe>,
);

impl PrivateMessageReportView {
  /// returns the PrivateMessageReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub fn read(conn: &mut PgConnection, report_id: PrivateMessageReportId) -> Result<Self, Error> {
    let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);

    let (private_message_report, private_message, private_message_creator, creator, resolver) =
      private_message_report::table
        .find(report_id)
        .inner_join(private_message::table)
        .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
        .inner_join(
          person_alias_1
            .on(private_message_report::creator_id.eq(person_alias_1.field(person::id))),
        )
        .left_join(
          person_alias_2.on(
            private_message_report::resolver_id.eq(person_alias_2.field(person::id).nullable()),
          ),
        )
        .select((
          private_message_report::all_columns,
          private_message::all_columns,
          Person::safe_columns_tuple(),
          person_alias_1.fields(Person::safe_columns_tuple()),
          person_alias_2
            .fields(Person::safe_columns_tuple())
            .nullable(),
        ))
        .first::<PrivateMessageReportViewTuple>(conn)?;

    Ok(Self {
      private_message_report,
      private_message,
      private_message_creator,
      creator,
      resolver,
    })
  }

  /// Returns the current unresolved post report count for the communities you mod
  pub fn get_report_count(conn: &mut PgConnection) -> Result<i64, Error> {
    use diesel::dsl::*;

    private_message_report::table
      .inner_join(private_message::table)
      .filter(private_message_report::resolved.eq(false))
      .into_boxed()
      .select(count(private_message_report::id))
      .first::<i64>(conn)
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PrivateMessageReportQuery<'a> {
  #[builder(!default)]
  conn: &'a mut PgConnection,
  page: Option<i64>,
  limit: Option<i64>,
  unresolved_only: Option<bool>,
}

impl<'a> PrivateMessageReportQuery<'a> {
  pub fn list(self) -> Result<Vec<PrivateMessageReportView>, Error> {
    let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);

    let mut query = private_message_report::table
      .inner_join(private_message::table)
      .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
      .inner_join(
        person_alias_1.on(private_message_report::creator_id.eq(person_alias_1.field(person::id))),
      )
      .left_join(
        person_alias_2
          .on(private_message_report::resolver_id.eq(person_alias_2.field(person::id).nullable())),
      )
      .select((
        private_message_report::all_columns,
        private_message::all_columns,
        Person::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
        person_alias_2
          .fields(Person::safe_columns_tuple())
          .nullable(),
      ))
      .into_boxed();

    if self.unresolved_only.unwrap_or(true) {
      query = query.filter(private_message_report::resolved.eq(false));
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;

    query = query
      .order_by(private_message::published.desc())
      .limit(limit)
      .offset(offset);

    let res = query.load::<PrivateMessageReportViewTuple>(self.conn)?;

    Ok(PrivateMessageReportView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for PrivateMessageReportView {
  type DbTuple = PrivateMessageReportViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        private_message_report: a.0,
        private_message: a.1,
        private_message_creator: a.2,
        creator: a.3,
        resolver: a.4,
      })
      .collect::<Vec<Self>>()
  }
}

#[cfg(test)]
mod tests {
  use crate::private_message_report_view::PrivateMessageReportQuery;
  use lemmy_db_schema::{
    source::{
      person::{Person, PersonForm},
      private_message::{PrivateMessage, PrivateMessageForm},
      private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
    },
    traits::{Crud, Reportable},
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();

    let new_person_1 = PersonForm {
      name: "timmy_mrv".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };
    let inserted_timmy = Person::create(conn, &new_person_1).unwrap();

    let new_person_2 = PersonForm {
      name: "jessica_mrv".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };
    let inserted_jessica = Person::create(conn, &new_person_2).unwrap();

    // timmy sends private message to jessica
    let pm_form = PrivateMessageForm {
      creator_id: inserted_timmy.id,
      recipient_id: inserted_jessica.id,
      content: "something offensive".to_string(),
      ..Default::default()
    };
    let pm = PrivateMessage::create(conn, &pm_form).unwrap();

    // jessica reports private message
    let pm_report_form = PrivateMessageReportForm {
      creator_id: inserted_jessica.id,
      original_pm_text: pm.content.clone(),
      private_message_id: pm.id,
      reason: "its offensive".to_string(),
    };
    let pm_report = PrivateMessageReport::report(conn, &pm_report_form).unwrap();

    let reports = PrivateMessageReportQuery::builder()
      .conn(conn)
      .build()
      .list()
      .unwrap();
    assert_eq!(1, reports.len());
    assert!(!reports[0].private_message_report.resolved);
    assert_eq!(inserted_timmy.name, reports[0].private_message_creator.name);
    assert_eq!(inserted_jessica.name, reports[0].creator.name);
    assert_eq!(pm_report.reason, reports[0].private_message_report.reason);
    assert_eq!(pm.content, reports[0].private_message.content);

    let new_person_3 = PersonForm {
      name: "admin_mrv".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };
    let inserted_admin = Person::create(conn, &new_person_3).unwrap();

    // admin resolves the report (after taking appropriate action)
    PrivateMessageReport::resolve(conn, pm_report.id, inserted_admin.id).unwrap();

    let reports = PrivateMessageReportQuery::builder()
      .conn(conn)
      .unresolved_only(Some(false))
      .build()
      .list()
      .unwrap();
    assert_eq!(1, reports.len());
    assert!(reports[0].private_message_report.resolved);
    assert!(reports[0].resolver.is_some());
    assert_eq!(
      inserted_admin.name,
      reports[0].resolver.as_ref().unwrap().name
    );
  }
}
