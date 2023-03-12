use crate::structs::ReviewCommentView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  schema::{comment, community, person, post, review_comment},
  source::{
    comment::Comment,
    community::Community,
    person::Person,
    post::Post,
    review_comment::ReviewComment,
  },
  traits::JoinView,
  utils::{get_conn, limit_and_offset, DbPool},
};
use typed_builder::TypedBuilder;

type ReviewCommentViewTuple = (
  ReviewComment,
  Comment,
  Post,
  Community,
  Person,
  Option<Person>,
);

impl ReviewCommentView {
  /// Returns the current unresolved review count
  pub async fn get_review_count(pool: &DbPool) -> Result<i64, Error> {
    use diesel::dsl::count;

    let conn = &mut get_conn(pool).await?;

    review_comment::table
      .inner_join(comment::table)
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .filter(review_comment::approved.eq(false))
      .select(count(review_comment::id))
      .first::<i64>(conn)
      .await
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct ReviewCommentQuery<'a> {
  #[builder(!default)]
  pool: &'a DbPool,
  page: Option<i64>,
  limit: Option<i64>,
  unapproved_only: Option<bool>,
}

impl<'a> ReviewCommentQuery<'a> {
  pub async fn list(self) -> Result<Vec<ReviewCommentView>, Error> {
    let conn = &mut get_conn(self.pool).await?;

    let person_alias_1 = diesel::alias!(person as person1);

    let mut query = review_comment::table
      .inner_join(comment::table)
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .left_join(
        person_alias_1
          .on(review_comment::approver_id.eq(person_alias_1.field(person::id).nullable())),
      )
      .select((
        review_comment::all_columns,
        comment::all_columns,
        post::all_columns,
        community::all_columns,
        person::all_columns,
        person_alias_1.fields(person::all_columns).nullable(),
      ))
      .into_boxed();

    if self.unapproved_only.unwrap_or(true) {
      query = query.filter(review_comment::approved.eq(false));
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;

    query = query
      .order_by(review_comment::published.desc())
      .limit(limit)
      .offset(offset);

    let res = query.load::<ReviewCommentViewTuple>(conn).await?;

    Ok(res.into_iter().map(ReviewCommentView::from_tuple).collect())
  }
}

impl JoinView for ReviewCommentView {
  type JoinTuple = ReviewCommentViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      review_comment: a.0,
      comment: a.1,
      post: a.2,
      community: a.3,
      comment_creator: a.4,
      resolver: a.5,
    }
  }
}
