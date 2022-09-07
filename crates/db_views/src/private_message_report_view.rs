use crate::structs::PrivateMessageReportView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PrivateMessageReportId,
  schema::{person, person_alias_1, person_alias_2, private_message, private_message_report},
  source::{
    person::{Person, PersonAlias1, PersonAlias2, PersonSafe, PersonSafeAlias1, PersonSafeAlias2},
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
  PersonSafeAlias1,
  Option<PersonSafeAlias2>,
);

impl PrivateMessageReportView {
  /// returns the PrivateMessageReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub fn read(conn: &PgConnection, report_id: PrivateMessageReportId) -> Result<Self, Error> {
    let (private_message_report, private_message, private_message_creator, creator, resolver) =
      private_message_report::table
        .find(report_id)
        .inner_join(private_message::table)
        .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
        .inner_join(
          person_alias_1::table.on(private_message_report::creator_id.eq(person_alias_1::id)),
        )
        .left_join(
          person_alias_2::table
            .on(private_message_report::resolver_id.eq(person_alias_2::id.nullable())),
        )
        .select((
          private_message_report::all_columns,
          private_message::all_columns,
          Person::safe_columns_tuple(),
          PersonAlias1::safe_columns_tuple(),
          PersonAlias2::safe_columns_tuple().nullable(),
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
  pub fn get_report_count(conn: &PgConnection) -> Result<i64, Error> {
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
  conn: &'a PgConnection,
  page: Option<i64>,
  limit: Option<i64>,
  unresolved_only: Option<bool>,
}

impl<'a> PrivateMessageReportQuery<'a> {
  pub fn list(self) -> Result<Vec<PrivateMessageReportView>, Error> {
    let mut query = private_message_report::table
      .inner_join(private_message::table)
      .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
      .inner_join(person_alias_1::table.on(private_message::creator_id.eq(person_alias_1::id)))
      .left_join(
        person_alias_2::table
          .on(private_message_report::resolver_id.eq(person_alias_2::id.nullable())),
      )
      .select((
        private_message_report::all_columns,
        private_message::all_columns,
        Person::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
        PersonAlias2::safe_columns_tuple().nullable(),
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
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    todo!()
  }
}
