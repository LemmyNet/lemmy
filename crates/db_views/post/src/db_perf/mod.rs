mod series;

use crate::{db_perf::series::ValuesFromSeries, impls::PostQuery};
use diesel::{
  ExpressionMethods,
  IntoSql,
  dsl::{self, sql},
  sql_types,
};
use diesel_async::{RunQueryDsl, SimpleAsyncConnection};
use lemmy_db_schema::source::{
  community::{Community, CommunityInsertForm},
  instance::Instance,
  person::{Person, PersonInsertForm},
  site::Site,
};
use lemmy_db_schema_file::{enums::PostSortType, schema::post};
use lemmy_diesel_utils::{
  connection::{build_db_pool, get_conn},
  traits::Crud,
  utils::now,
};
use lemmy_utils::error::LemmyResult;
use serial_test::serial;
use std::{fmt::Display, num::NonZeroU32, str::FromStr};
use url::Url;

#[derive(Debug)]
struct CmdArgs {
  communities: NonZeroU32,
  people: NonZeroU32,
  posts: NonZeroU32,
  read_post_pages: u32,
  explain_insertions: bool,
}

fn get_option<T: FromStr + Display>(suffix: &str, default: T) -> Result<T, T::Err> {
  let name = format!("LEMMY_{suffix}");
  if let Some(value) = std::env::var_os(&name) {
    value.to_string_lossy().parse()
  } else {
    println!("🔧 using default env var {name}={default}");
    Ok(default)
  }
}

#[tokio::test]
#[serial]
async fn db_perf() -> LemmyResult<()> {
  let args = CmdArgs {
    communities: get_option("COMMUNITIES", 3.try_into()?)?,
    people: get_option("PEOPLE", 3.try_into()?)?,
    posts: get_option("POSTS", 100000.try_into()?)?,
    read_post_pages: get_option("READ_POST_PAGES", 0)?,
    explain_insertions: get_option("EXPLAIN_INSERTIONS", false)?,
  };
  let pool = &build_db_pool()?;
  let pool = &mut pool.into();
  let conn = &mut get_conn(pool).await?;

  if args.explain_insertions {
    // log_nested_statements is enabled to log trigger execution
    conn
      .batch_execute(
        "SET auto_explain.log_min_duration = 0; SET auto_explain.log_nested_statements = on;",
      )
      .await?;
  }

  let instance = Instance::read_or_create(&mut conn.into(), "reddit.com").await?;

  println!("🫃 creating {} people", args.people);
  let mut person_ids = vec![];
  for i in 0..args.people.get() {
    let form = PersonInsertForm::test_form(instance.id, &format!("p{i}"));
    person_ids.push(Person::create(&mut conn.into(), &form).await?.id);
  }

  println!("🌍 creating {} communities", args.communities);
  let mut community_ids = vec![];
  for i in 0..args.communities.get() {
    let form = CommunityInsertForm::new(
      instance.id,
      format!("c{i}"),
      i.to_string(),
      "pubkey".to_string(),
    );
    community_ids.push(Community::create(&mut conn.into(), &form).await?.id);
  }

  let post_batches = args.people.get() * args.communities.get();
  let posts_per_batch = args.posts.get() / post_batches;
  let num_posts: usize = (post_batches * posts_per_batch).try_into()?;
  println!(
    "📜 creating {} posts ({} featured in community)",
    num_posts, post_batches
  );
  let mut num_inserted_posts = 0;
  // TODO: progress bar
  for person_id in &person_ids {
    for community_id in &community_ids {
      let n = dsl::insert_into(post::table)
        .values(ValuesFromSeries {
          start: 1,
          stop: posts_per_batch.into(),
          selection: (
            "AAAAAAAAAAA".into_sql::<sql_types::Text>(),
            person_id.into_sql::<sql_types::Integer>(),
            community_id.into_sql::<sql_types::Integer>(),
            series::current_value.eq(1),
            now()
              - sql::<sql_types::Interval>("make_interval(secs => ")
                .bind::<sql_types::BigInt, _>(series::current_value)
                .sql(")"),
          ),
        })
        .into_columns((
          post::name,
          post::creator_id,
          post::community_id,
          post::featured_community,
          post::published_at,
        ))
        .execute(conn)
        .await?;
      num_inserted_posts += n;
    }
  }
  // Make sure the println above shows the correct amount
  assert_eq!(num_inserted_posts, num_posts);

  // Manually trigger and wait for a statistics update to ensure consistent and high amount of
  // accuracy in the statistics used for query planning
  println!("🧮 updating database statistics");
  conn.batch_execute("ANALYZE;").await?;

  // Enable auto_explain
  conn
    .batch_execute(
      "SET auto_explain.log_min_duration = 0; SET auto_explain.log_nested_statements = off;",
    )
    .await?;

  // TODO: show execution duration stats
  let mut page_cursor = None;
  for page_num in 1..=args.read_post_pages {
    println!(
      "👀 getting page {page_num} of posts (pagination cursor used: {})",
      page_cursor.is_some()
    );

    // TODO: include local_user
    let post_views = PostQuery {
      community_id: community_ids.as_slice().first().cloned(),
      sort: Some(PostSortType::New),
      limit: Some(20),
      page_cursor,
      ..Default::default()
    }
    .list(&site()?, &mut conn.into())
    .await?;

    if let Some(cursor) = post_views.next_page {
      println!("👀 getting pagination cursor data for next page");
      page_cursor = Some(cursor);
    } else {
      println!("👀 reached empty page");
      break;
    }
  }

  // Delete everything, which might prevent problems if this is not run using scripts/db_perf.sh
  Instance::delete(&mut conn.into(), instance.id).await?;

  if let Ok(path) = std::env::var("PGDATA") {
    println!("🪵 query plans written in {path}/log");
  }

  Ok(())
}

fn site() -> LemmyResult<Site> {
  Ok(Site {
    id: Default::default(),
    name: String::new(),
    sidebar: None,
    published_at: Default::default(),
    updated_at: None,
    icon: None,
    banner: None,
    summary: None,
    ap_id: Url::parse("http://example.com")?.into(),
    last_refreshed_at: Default::default(),
    inbox_url: Url::parse("http://example.com")?.into(),
    private_key: None,
    public_key: String::new(),
    instance_id: Default::default(),
    content_warning: None,
  })
}
