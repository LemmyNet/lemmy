use crate::structs::{
  VoteAnalyticsByCommunity,
  VoteAnalyticsByPerson,
  VoteAnalyticsGivenByPersonView,
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::exists,
  result::{Error, Error::QueryBuilderError},
  select,
  sql_query,
  sql_types::{BigInt, Double, Integer, Nullable, Text, Timestamptz},
  QueryDsl,
  QueryableByName,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::person,
  source::{community::Community, person::Person},
  utils::{get_conn, DbPool, FETCH_LIMIT_MAX},
};
use std::collections::HashMap;

const VOTE_FETCH_LIMIT_DEFAULT: i64 = 20;
const VOTE_FETCH_LIMIT_MAX: i64 = FETCH_LIMIT_MAX * 2;

fn fetch_limit(limit: Option<i64>) -> Result<i64, Error> {
  Ok(match limit {
    Some(limit) => {
      if !(1..=VOTE_FETCH_LIMIT_MAX).contains(&limit) {
        return Err(QueryBuilderError(
          format!("Vote fetch limit is > {VOTE_FETCH_LIMIT_MAX}").into(),
        ));
      }
      limit
    }
    None => VOTE_FETCH_LIMIT_DEFAULT,
  })
}

fn create_person_votes_view(
  result: &VotesByTargetResult,
  persons: &HashMap<PersonId, Person>,
) -> Result<VoteAnalyticsByPerson, Error> {
  if let Some(person_id) = result.target {
    return Ok(VoteAnalyticsByPerson {
      creator: persons
        .get(&PersonId(person_id))
        .ok_or_else(|| Error::NotFound)?
        .clone(),
      total_votes: result.total_votes,
      upvotes: result.upvotes,
      downvotes: result.downvotes,
      upvote_percentage: result.upvote_percentage,
    });
  }
  Err(Error::NotFound)
}

fn create_community_votes_view(
  result: &VotesByTargetResult,
  communities: &HashMap<CommunityId, Community>,
) -> Result<VoteAnalyticsByCommunity, Error> {
  if let Some(community_id) = result.target {
    return Ok(VoteAnalyticsByCommunity {
      community: communities
        .get(&CommunityId(community_id))
        .ok_or_else(|| Error::NotFound)?
        .clone(),
      total_votes: result.total_votes,
      upvotes: result.upvotes,
      downvotes: result.downvotes,
      upvote_percentage: result.upvote_percentage,
    });
  }
  Err(Error::NotFound)
}

fn extract_person_ids(results: Vec<&VotesByTargetResult>) -> Result<Vec<PersonId>, Error> {
  // it's possible that this contains duplicates, but that will get deduplicated by postgres
  let person_ids = results
    .iter()
    .map(|&x| x.target.ok_or_else(|| Error::NotFound).map(PersonId))
    .collect::<Result<Vec<_>, _>>()?;
  Ok(person_ids)
}

fn extract_community_ids(results: Vec<&VotesByTargetResult>) -> Result<Vec<CommunityId>, Error> {
  // it's possible that this contains duplicates, but that will get deduplicated by postgres
  let community_ids = results
    .iter()
    .map(|&x| x.target.ok_or_else(|| Error::NotFound).map(CommunityId))
    .collect::<Result<Vec<_>, _>>()?;
  Ok(community_ids)
}

#[derive(QueryableByName)]
struct VotesByTargetResult {
  #[diesel(sql_type = Text)]
  target_type: String,
  #[diesel(sql_type = Nullable<Integer>)]
  target: Option<i32>,
  #[diesel(sql_type = BigInt)]
  total_votes: i64,
  #[diesel(sql_type = BigInt)]
  upvotes: i64,
  #[diesel(sql_type = BigInt)]
  downvotes: i64,
  #[diesel(sql_type = Double)]
  upvote_percentage: f64,
}

impl VoteAnalyticsGivenByPersonView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    limit: Option<i64>,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    // Ensure person exists, as the other queries do not necessarily return rows that would indicate
    // the existence of a user.
    let person_exists: bool = select(exists(person::table.find(&person_id)))
      .get_result(conn)
      .await?;
    if !person_exists {
      Err(Error::NotFound)?
    }

    let limit = fetch_limit(limit)?;

    // This is a rather dangerous workaround; this number must be one above than the highest
    // parameter used in the statements below without leaving any space. It could probably be
    // improved by implementing QueryFragments.
    let mut sql_dynamic_parameter_binding_index = 3u8;
    let (sql_since_post, sql_since_comment) = start_time
      .map(|_| {
        let (s_post, s_comment) = (
          format!("AND post_like.published >= ${sql_dynamic_parameter_binding_index}"),
          format!("AND comment_like.published >= ${sql_dynamic_parameter_binding_index}"),
        );
        sql_dynamic_parameter_binding_index += 1;
        (s_post, s_comment)
      })
      .unwrap_or_default();
    let (sql_until_post, sql_until_comment) = end_time
      .map(|_| {
        let (s_post, s_comment) = (
          format!("AND post_like.published <= ${sql_dynamic_parameter_binding_index}"),
          format!("AND comment_like.published <= ${sql_dynamic_parameter_binding_index}"),
        );
        sql_dynamic_parameter_binding_index += 1;
        (s_post, s_comment)
      })
      .unwrap_or_default();

    let mut post_votes_by_target_query = sql_query(format!(
            r#"
WITH post_likes_by_voter AS (
    SELECT post_like.score,
        creator.id AS creator,
        community.id AS community
    FROM person voter
        JOIN post_like ON post_like.person_id = voter.id
        JOIN post ON post.id = post_like.post_id
        JOIN person creator ON creator.id = post.creator_id
        JOIN community ON community.id = post.community_id
    WHERE voter.id = $1
        AND post_like.score != 0
        AND creator.id != voter.id
        {since}
        {until}
), post_likes_by_recipient AS (
    SELECT 'person' AS target_type,
        creator AS target,
        COUNT(*) AS total_votes,
        COUNT(score = 1 OR NULL) AS upvotes,
        COUNT(score = -1 OR NULL) AS downvotes,
        CASE WHEN COUNT(*) > 0 THEN 100::float * COUNT(score = 1 OR NULL) / COUNT(*) ELSE 0::float END AS upvote_percentage
    FROM post_likes_by_voter
    GROUP BY creator
    ORDER BY
        total_votes DESC,
        creator ASC
    LIMIT $2
), post_likes_by_community AS (
    SELECT 'community' AS target_type,
        community AS target,
        COUNT(*) AS total_votes,
        COUNT(score = 1 OR NULL) AS upvotes,
        COUNT(score = -1 OR NULL) AS downvotes,
        CASE WHEN COUNT(*) > 0 THEN 100::float * COUNT(score = 1 OR NULL) / COUNT(*) ELSE 0::float END AS upvote_percentage
    FROM post_likes_by_voter
    GROUP BY community
    ORDER BY
        total_votes DESC,
        community ASC
    LIMIT $2
)

SELECT 'total' AS target_type,
    NULL AS target,
    COUNT(*) AS total_votes,
    COUNT(score = 1 OR NULL) AS upvotes,
    COUNT(score = -1 OR NULL) AS downvotes,
    CASE WHEN COUNT(*) > 0 THEN 100::float * COUNT(score = 1 OR NULL) / COUNT(*) ELSE 0::float END AS upvote_percentage
FROM post_likes_by_voter

UNION ALL
SELECT * FROM post_likes_by_recipient
UNION ALL
SELECT * FROM post_likes_by_community
      "#,
            since = sql_since_post,
            until = sql_until_post,
        )).into_boxed()
            .bind::<Integer, _>(&person_id.0)
            .bind::<BigInt, _>(limit);
    // this order must match the order in which the dynamic parameter binding index was generated
    if let Some(t) = start_time {
      post_votes_by_target_query = post_votes_by_target_query.bind::<Timestamptz, _>(t);
    }
    if let Some(t) = end_time {
      post_votes_by_target_query = post_votes_by_target_query.bind::<Timestamptz, _>(t);
    }
    let post_votes_by_target: Vec<VotesByTargetResult> =
      post_votes_by_target_query.get_results(conn).await?;

    let mut comment_votes_by_target_query = sql_query(format!(
            r#"
WITH comment_likes_by_voter AS (
    SELECT comment_like.score,
        creator.id AS creator,
        community.id AS community
    FROM person voter
        JOIN comment_like ON comment_like.person_id = voter.id
        JOIN comment on comment.id = comment_like.comment_id
        JOIN person creator ON creator.id = comment.creator_id
        JOIN post ON post.id = comment.post_id
        JOIN community ON community.id = post.community_id
    WHERE voter.id = $1
        AND comment_like.score != 0
        AND creator.id != voter.id
        {since}
        {until}
), comment_likes_by_recipient AS (
    SELECT 'person' AS target_type,
        creator AS target,
        COUNT(*) AS total_votes,
        COUNT(score = 1 OR NULL) AS upvotes,
        COUNT(score = -1 OR NULL) AS downvotes,
        CASE WHEN COUNT(*) > 0 THEN 100::float * COUNT(score = 1 OR NULL) / COUNT(*) ELSE 0::float END AS upvote_percentage
    FROM comment_likes_by_voter
    GROUP BY creator
    ORDER BY
        total_votes DESC,
        creator ASC
    LIMIT $2
), comment_likes_by_community AS (
    SELECT 'community' AS target_type,
        community AS target,
        COUNT(*) AS total_votes,
        COUNT(score = 1 OR NULL) AS upvotes,
        COUNT(score = -1 OR NULL) AS downvotes,
        CASE WHEN COUNT(*) > 0 THEN 100::float * COUNT(score = 1 OR NULL) / COUNT(*) ELSE 0::float END AS upvote_percentage
    FROM comment_likes_by_voter
    GROUP BY community
    ORDER BY
        total_votes DESC,
        community ASC
    LIMIT $2
)

SELECT 'total' AS target_type,
    NULL AS target,
    COUNT(*) AS total_votes,
    COUNT(score = 1 OR NULL) AS upvotes,
    COUNT(score = -1 OR NULL) AS downvotes,
    CASE WHEN COUNT(*) > 0 THEN 100::float * COUNT(score = 1 OR NULL) / COUNT(*) ELSE 0::float END AS upvote_percentage
FROM comment_likes_by_voter

UNION ALL
SELECT * FROM comment_likes_by_recipient
UNION ALL
SELECT * FROM comment_likes_by_community
      "#,
            since = sql_since_comment,
            until = sql_until_comment,
        )).into_boxed()
            .bind::<Integer, _>(&person_id.0)
            .bind::<BigInt, _>(limit);
    // this order must match the order in which the dynamic parameter binding index was generated
    if let Some(t) = start_time {
      comment_votes_by_target_query = comment_votes_by_target_query.bind::<Timestamptz, _>(t);
    }
    if let Some(t) = end_time {
      comment_votes_by_target_query = comment_votes_by_target_query.bind::<Timestamptz, _>(t);
    }
    let comment_votes_by_target: Vec<VotesByTargetResult> =
      comment_votes_by_target_query.get_results(conn).await?;

    let person_type = "person".to_string();
    let post_votes_by_target_person: Vec<_> = post_votes_by_target
      .iter()
      .filter(|&x| x.target_type.eq(&person_type))
      .collect();
    let comment_votes_by_target_person: Vec<_> = comment_votes_by_target
      .iter()
      .filter(|&x| x.target_type.eq(&person_type))
      .collect();

    let combined_votes_by_target_person: Vec<&VotesByTargetResult> = post_votes_by_target_person
      .clone()
      .into_iter()
      .chain(comment_votes_by_target_person.clone())
      .collect();

    let person_ids: Vec<PersonId> = extract_person_ids(combined_votes_by_target_person)?;
    let persons = Person::read_many(pool, &person_ids, true).await?;

    let post_votes_by_target_person_resolved: Vec<VoteAnalyticsByPerson> =
      post_votes_by_target_person
        .iter()
        .map(|person| create_person_votes_view(person, &persons))
        .collect::<Result<_, _>>()?;
    let comment_votes_by_target_person_resolved: Vec<VoteAnalyticsByPerson> =
      comment_votes_by_target_person
        .iter()
        .map(|person| create_person_votes_view(person, &persons))
        .collect::<Result<_, _>>()?;

    let community_type = "community".to_string();
    let post_votes_by_target_community: Vec<_> = post_votes_by_target
      .iter()
      .filter(|&x| x.target_type.eq(&community_type))
      .collect();
    let comment_votes_by_target_community: Vec<_> = comment_votes_by_target
      .iter()
      .filter(|&x| x.target_type.eq(&community_type))
      .collect();

    let combined_votes_by_target_community: Vec<&VotesByTargetResult> =
      post_votes_by_target_community
        .clone()
        .into_iter()
        .chain(comment_votes_by_target_community.clone())
        .collect();

    let community_ids = extract_community_ids(combined_votes_by_target_community)?;
    let communities = Community::read_many(pool, &community_ids, true).await?;

    let post_votes_by_target_community_resolved: Vec<VoteAnalyticsByCommunity> =
      post_votes_by_target_community
        .iter()
        .map(|community| create_community_votes_view(community, &communities))
        .collect::<Result<_, _>>()?;
    let comment_votes_by_target_community_resolved: Vec<VoteAnalyticsByCommunity> =
      comment_votes_by_target_community
        .iter()
        .map(|community| create_community_votes_view(community, &communities))
        .collect::<Result<_, _>>()?;

    let total_type = "total".to_string();
    let post_totals = post_votes_by_target
      .iter()
      .find(|&x| x.target_type.eq(&total_type))
      .ok_or(Error::NotFound)?;
    let comment_totals = comment_votes_by_target
      .iter()
      .find(|&x| x.target_type.eq(&total_type))
      .ok_or(Error::NotFound)?;

    Ok(VoteAnalyticsGivenByPersonView {
      post_votes_total_votes: post_totals.total_votes,
      post_votes_total_upvotes: post_totals.upvotes,
      post_votes_total_downvotes: post_totals.downvotes,
      post_votes_total_upvote_percentage: post_totals.upvote_percentage,
      post_votes_by_target_user: post_votes_by_target_person_resolved,
      post_votes_by_target_community: post_votes_by_target_community_resolved,
      comment_votes_total_votes: comment_totals.total_votes,
      comment_votes_total_upvotes: comment_totals.upvotes,
      comment_votes_total_downvotes: comment_totals.downvotes,
      comment_votes_total_upvote_percentage: comment_totals.upvote_percentage,
      comment_votes_by_target_user: comment_votes_by_target_person_resolved,
      comment_votes_by_target_community: comment_votes_by_target_community_resolved,
    })
  }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod test {
  use crate::structs::VoteAnalyticsGivenByPersonView;
  use diesel::result::Error;
  use lemmy_db_schema::{
    assert_length,
    newtypes::PersonId,
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm, PostLike, PostLikeForm},
    },
    traits::{Crud, Likeable},
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::error::LemmyResult;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_vote_analytics() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;
    let community_form = CommunityInsertForm::builder()
      .name("vote_test".to_string())
      .title("vote_test".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let community = Community::create(pool, &community_form).await?;

    let alice_form = PersonInsertForm {
      ..PersonInsertForm::test_form(inserted_instance.id, "alice")
    };
    let alice = Person::create(pool, &alice_form).await?;
    let mut alice_posts: Vec<Post> = vec![];
    for _ in 0..=9 {
      let post_form = PostInsertForm::builder()
        .name("A test post".into())
        .creator_id(alice.id)
        .community_id(community.id)
        .build();

      let post = Post::create(pool, &post_form).await?;
      alice_posts.push(post);
    }

    let bob_form = PersonInsertForm {
      ..PersonInsertForm::test_form(inserted_instance.id, "bob")
    };
    let bob = Person::create(pool, &bob_form).await?;
    let bob_post_form = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(bob.id)
      .community_id(community.id)
      .build();
    let bob_post = Post::create(pool, &bob_post_form).await?;

    // readability
    #[allow(clippy::needless_range_loop)]
    for i in 0..=9 {
      let post = alice_posts[i].clone();
      // 2 votes without score, 3 upvotes, 5 downvotes
      let score = if i < 2 {
        0
      } else if i < 5 {
        1
      } else {
        -1
      };
      let like_form = PostLikeForm {
        post_id: post.id,
        person_id: bob.id,
        score,
      };
      PostLike::like(pool, &like_form).await?;
    }
    let like_form = PostLikeForm {
      post_id: bob_post.id,
      person_id: bob.id,
      score: 1,
    };
    PostLike::like(pool, &like_form).await?;

    // Test for non-existing person
    let invalid_person_id = PersonId(-1);
    let view =
      VoteAnalyticsGivenByPersonView::read(pool, invalid_person_id, None, None, None).await;
    assert!(
      view.is_err_and(|e| e == Error::NotFound),
      "query should not match a person",
    );

    // alice exists but hasn't voted on anything
    let view = VoteAnalyticsGivenByPersonView::read(pool, alice.id, None, None, None).await?;
    assert_eq!(0, view.post_votes_total_votes);
    assert_eq!(0, view.post_votes_total_upvotes);
    assert_eq!(0, view.post_votes_total_downvotes);
    assert_eq!(0.0, view.post_votes_total_upvote_percentage);
    assert_length!(0, view.post_votes_by_target_user);
    assert_length!(0, view.post_votes_by_target_community);

    let view = VoteAnalyticsGivenByPersonView::read(pool, bob.id, None, None, None).await?;

    assert_eq!(8, view.post_votes_total_votes);
    assert_eq!(3, view.post_votes_total_upvotes);
    assert_eq!(5, view.post_votes_total_downvotes);
    assert_eq!(37.5, view.post_votes_total_upvote_percentage);
    assert_length!(1, view.post_votes_by_target_user);
    assert_length!(1, view.post_votes_by_target_community);
    assert_eq!(alice.id, view.post_votes_by_target_user[0].creator.id);
    assert_eq!(
      community.id,
      view.post_votes_by_target_community[0].community.id
    );

    // TODO: test limits, multiple users, multiple communities, time ranges, comments

    Person::delete(pool, alice.id).await?;
    Person::delete(pool, bob.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
