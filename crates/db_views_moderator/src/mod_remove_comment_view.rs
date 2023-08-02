use crate::structs::{ModRemoveCommentView, ModlogListParams};
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  IntoSql,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{comment, community, mod_remove_comment, person, post},
  source::{
    comment::Comment,
    community::Community,
    moderator::ModRemoveComment,
    person::Person,
    post::Post,
  },
  traits::JoinView,
  utils::{get_conn, limit_and_offset, DbPool},
};

type ModRemoveCommentViewTuple = (
  ModRemoveComment,
  Option<Person>,
  Comment,
  Person,
  Post,
  Community,
);

impl ModRemoveCommentView {
  pub async fn list(pool: &mut DbPool<'_>, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let person_alias_1 = diesel::alias!(lemmy_db_schema::schema::person as person1);
    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = mod_remove_comment::mod_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));
    let mut query = mod_remove_comment::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(comment::table)
      .inner_join(person_alias_1.on(comment::creator_id.eq(person_alias_1.field(person::id))))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .select((
        mod_remove_comment::all_columns,
        person::all_columns.nullable(),
        comment::all_columns,
        person_alias_1.fields(person::all_columns),
        post::all_columns,
        community::all_columns,
      ))
      .into_boxed();

    if let Some(community_id) = params.community_id {
      query = query.filter(post::community_id.eq(community_id));
    };

    if let Some(mod_person_id) = params.mod_person_id {
      query = query.filter(mod_remove_comment::mod_person_id.eq(mod_person_id));
    };

    if let Some(other_person_id) = params.other_person_id {
      query = query.filter(person_alias_1.field(person::id).eq(other_person_id));
    };

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_remove_comment::when_.desc())
      .load::<ModRemoveCommentViewTuple>(conn)
      .await?;

    let results = res.into_iter().map(Self::from_tuple).collect();
    Ok(results)
  }
}

impl JoinView for ModRemoveCommentView {
  type JoinTuple = ModRemoveCommentViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      mod_remove_comment: a.0,
      moderator: a.1,
      comment: a.2,
      commenter: a.3,
      post: a.4,
      community: a.5,
    }
  }
}
