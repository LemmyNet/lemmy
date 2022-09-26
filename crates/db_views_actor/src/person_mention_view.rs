use crate::structs::PersonMentionView;
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  aggregates::structs::CommentAggregates,
  newtypes::{PersonId, PersonMentionId},
  schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_saved,
    community,
    community_follower,
    community_person_ban,
    person,
    person_block,
    person_mention,
    post,
  },
  source::{
    comment::{Comment, CommentSaved},
    community::{Community, CommunityFollower, CommunityPersonBan, CommunitySafe},
    person::{Person, PersonSafe},
    person_block::PersonBlock,
    person_mention::PersonMention,
    post::Post,
  },
  traits::{ToSafe, ViewToVec},
  utils::{functions::hot_rank, limit_and_offset},
  CommentSortType,
};
use typed_builder::TypedBuilder;

type PersonMentionViewTuple = (
  PersonMention,
  Comment,
  PersonSafe,
  Post,
  CommunitySafe,
  PersonSafe,
  CommentAggregates,
  Option<CommunityPersonBan>,
  Option<CommunityFollower>,
  Option<CommentSaved>,
  Option<PersonBlock>,
  Option<i16>,
);

impl PersonMentionView {
  pub fn read(
    conn: &mut PgConnection,
    person_mention_id: PersonMentionId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    let person_alias_1 = diesel::alias!(person as person1);

    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    let (
      person_mention,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community,
      follower,
      saved,
      creator_blocked,
      my_vote,
    ) = person_mention::table
      .find(person_mention_id)
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person_alias_1)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          comment::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(person_id_join)),
        ),
      )
      .select((
        person_mention::all_columns,
        comment::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .first::<PersonMentionViewTuple>(conn)?;

    Ok(PersonMentionView {
      person_mention,
      comment,
      creator,
      post,
      community,
      recipient,
      counts,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      subscribed: CommunityFollower::to_subscribed_type(&follower),
      saved: saved.is_some(),
      creator_blocked: creator_blocked.is_some(),
      my_vote,
    })
  }

  /// Gets the number of unread mentions
  pub fn get_unread_mentions(
    conn: &mut PgConnection,
    my_person_id: PersonId,
  ) -> Result<i64, Error> {
    use diesel::dsl::*;

    person_mention::table
      .filter(person_mention::recipient_id.eq(my_person_id))
      .filter(person_mention::read.eq(false))
      .select(count(person_mention::id))
      .first::<i64>(conn)
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PersonMentionQuery<'a> {
  #[builder(!default)]
  conn: &'a mut PgConnection,
  my_person_id: Option<PersonId>,
  recipient_id: Option<PersonId>,
  sort: Option<CommentSortType>,
  unread_only: Option<bool>,
  show_bot_accounts: Option<bool>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PersonMentionQuery<'a> {
  pub fn list(self) -> Result<Vec<PersonMentionView>, Error> {
    use diesel::dsl::*;

    let person_alias_1 = diesel::alias!(person as person1);

    // The left join below will return None in this case
    let person_id_join = self.my_person_id.unwrap_or(PersonId(-1));

    let mut query = person_mention::table
      .inner_join(comment::table)
      .inner_join(person::table.on(comment::creator_id.eq(person::id)))
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person_alias_1)
      .inner_join(comment_aggregates::table.on(comment::id.eq(comment_aggregates::comment_id)))
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          comment::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(person_id_join)),
        ),
      )
      .select((
        person_mention::all_columns,
        comment::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
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

    query = match self.sort.unwrap_or(CommentSortType::Hot) {
      CommentSortType::Hot => query
        .then_order_by(hot_rank(comment_aggregates::score, comment_aggregates::published).desc())
        .then_order_by(comment_aggregates::published.desc()),
      CommentSortType::New => query.then_order_by(comment::published.desc()),
      CommentSortType::Old => query.then_order_by(comment::published.asc()),
      CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;

    let res = query
      .limit(limit)
      .offset(offset)
      .load::<PersonMentionViewTuple>(self.conn)?;

    Ok(PersonMentionView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for PersonMentionView {
  type DbTuple = PersonMentionViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        person_mention: a.0,
        comment: a.1,
        creator: a.2,
        post: a.3,
        community: a.4,
        recipient: a.5,
        counts: a.6,
        creator_banned_from_community: a.7.is_some(),
        subscribed: CommunityFollower::to_subscribed_type(&a.8),
        saved: a.9.is_some(),
        creator_blocked: a.10.is_some(),
        my_vote: a.11,
      })
      .collect::<Vec<Self>>()
  }
}
