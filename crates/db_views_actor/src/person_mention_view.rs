use crate::structs::PersonMentionView;
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{PersonId, PersonMentionId},
  schema::{comment, person, person_mention, post},
  source::{comment::Comment, person::Person, person_mention::PersonMention, post::Post},
  traits::JoinView,
  utils::{get_conn, limit_and_offset, DbPool},
};
use typed_builder::TypedBuilder;

type PersonMentionViewTuple = (PersonMention, Option<Comment>, Option<Post>, Person, Person);

impl PersonMentionView {
  pub async fn read(pool: &DbPool, person_mention_id: PersonMentionId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_alias_1 = diesel::alias!(person as person1);

    let (person_mention, comment, post, creator, recipient) = person_mention::table
      .find(person_mention_id)
      .left_join(comment::table)
      .left_join(post::table)
      .inner_join(
        person::table.on(
          comment::creator_id
            .eq(person::id)
            .or(post::creator_id.eq(person::id)),
        ),
      )
      .inner_join(person_alias_1)
      .select((
        person_mention::all_columns,
        comment::all_columns.nullable(),
        post::all_columns.nullable(),
        person::all_columns,
        person_alias_1.fields(person::all_columns),
      ))
      .first::<PersonMentionViewTuple>(conn)
      .await?;

    Ok(PersonMentionView {
      person_mention,
      comment,
      post,
      creator,
      recipient,
    })
  }

  /// Gets the number of unread mentions
  pub async fn get_unread_mentions(pool: &DbPool, my_person_id: PersonId) -> Result<i64, Error> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;

    person_mention::table
      .left_join(comment::table)
      .left_join(post::table)
      .filter(person_mention::recipient_id.eq(my_person_id))
      .filter(person_mention::read.eq(false))
      // TODO check to make sure these filters work. You might need to move them up to the joins
      .filter(comment::deleted.eq(false))
      .filter(comment::removed.eq(false))
      .filter(post::deleted.eq(false))
      .filter(post::removed.eq(false))
      .select(count(person_mention::id))
      .first::<i64>(conn)
      .await
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PersonMentionQuery<'a> {
  #[builder(!default)]
  pool: &'a DbPool,
  recipient_id: Option<PersonId>,
  unread_only: Option<bool>,
  show_bot_accounts: Option<bool>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PersonMentionQuery<'a> {
  pub async fn list(self) -> Result<Vec<PersonMentionView>, Error> {
    let conn = &mut get_conn(self.pool).await?;

    let person_alias_1 = diesel::alias!(person as person1);

    let mut query = person_mention::table
      .left_join(comment::table)
      .left_join(post::table)
      .inner_join(
        person::table.on(
          comment::creator_id
            .eq(person::id)
            .or(post::creator_id.eq(person::id)),
        ),
      )
      .inner_join(person_alias_1)
      .select((
        person_mention::all_columns,
        comment::all_columns.nullable(),
        post::all_columns.nullable(),
        person::all_columns,
        person_alias_1.fields(person::all_columns),
      ))
      .into_boxed();

    if let Some(recipient_id) = self.recipient_id {
      query = query.filter(person_mention::recipient_id.eq(recipient_id));
    }

    if self.unread_only.unwrap_or(false) {
      query = query.filter(person_mention::read.eq(false));
    }

    if !self.show_bot_accounts.unwrap_or(true) {
      query = query.filter(person::bot_account.eq(false));
    };

    query = query.order_by(person_mention::published.desc());

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .load::<PersonMentionViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(PersonMentionView::from_tuple).collect())
  }
}

impl JoinView for PersonMentionView {
  type JoinTuple = PersonMentionViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      person_mention: a.0,
      comment: a.1,
      post: a.2,
      creator: a.3,
      recipient: a.4,
    }
  }
}
