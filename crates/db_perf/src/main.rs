mod series;

use crate::series::ValuesFromSeries;
use anyhow::Context;
use clap::Parser;
use diesel::{
  dsl::{self, sql},
  sql_types,
  ExpressionMethods,
  IntoSql,
};
use diesel_async::{RunQueryDsl, SimpleAsyncConnection};
use lemmy_db_schema::{
  schema::post,
  source::{
    community::{Community, CommunityInsertForm},
    instance::Instance,
    person::{Person, PersonInsertForm},
  },
  traits::Crud,
  utils::{build_db_pool, get_conn, now},
  SortType,
};
use lemmy_db_views::{post_view::PostQuery, structs::PaginationCursor};
use lemmy_utils::error::{LemmyErrorExt2, LemmyResult};
use std::num::NonZeroU32;

#[derive(Parser, Debug)]
struct CmdArgs {
  #[arg(long, default_value_t = 3.try_into().unwrap())]
  communities: NonZeroU32,
  #[arg(long, default_value_t = 3.try_into().unwrap())]
  people: NonZeroU32,
  #[arg(long, default_value_t = 100000.try_into().unwrap())]
  posts: NonZeroU32,
  #[arg(long, default_value_t = 0)]
  read_post_pages: u32,
  #[arg(long)]
  explain_insertions: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut result = try_main().await.into_anyhow();
  if let Ok(path) = std::env::var("PGDATA") {
    result = result.with_context(|| {
      format!("Failed to run lemmy_db_perf (more details might be available in {path}/log)")
    });
  }
  result
}

async fn try_main() -> LemmyResult<()> {
  let args = CmdArgs::parse();
  let pool = &build_db_pool().await?;
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

  let instance = Instance::read_or_create(&mut conn.into(), "reddit.com".to_owned()).await?;

  println!("🫃 creating {} people", args.people);
  let mut person_ids = vec![];
  for i in 0..args.people.get() {
    let form = PersonInsertForm::builder()
      .name(format!("p{i}"))
      .public_key("pubkey".to_owned())
      .instance_id(instance.id)
      .build();
    person_ids.push(Person::create(&mut conn.into(), &form).await?.id);
  }

  println!("🌍 creating {} communities", args.communities);
  let mut community_ids = vec![];
  for i in 0..args.communities.get() {
    let form = CommunityInsertForm::builder()
      .name(format!("c{i}"))
      .title(i.to_string())
      .instance_id(instance.id)
      .build();
    community_ids.push(Community::create(&mut conn.into(), &form).await?.id);
  }

  let post_batches = args.people.get() * args.communities.get();
  let posts_per_batch = args.posts.get() / post_batches;
  let num_posts = post_batches * posts_per_batch;
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
          post::published,
        ))
        .execute(conn)
        .await?;
      num_inserted_posts += n;
    }
  }
  // Make sure the println above shows the correct amount
  assert_eq!(num_inserted_posts, num_posts as usize);

  // Enable auto_explain
  conn
    .batch_execute(
      "SET auto_explain.log_min_duration = 0; SET auto_explain.log_nested_statements = off;",
    )
    .await?;

  // TODO: show execution duration stats
  let mut page_after = None;
  for page_num in 1..=args.read_post_pages {
    println!(
      "👀 getting page {page_num} of posts (pagination cursor used: {})",
      page_after.is_some()
    );

    // TODO: include local_user
    let post_views = PostQuery {
      community_id: community_ids.as_slice().first().cloned(),
      sort: Some(SortType::New),
      limit: Some(20),
      page_after,
      ..Default::default()
    }
    .list(&mut conn.into())
    .await?;

    if let Some(post_view) = post_views.into_iter().last() {
      println!("👀 getting pagination cursor data for next page");
      let cursor_data = PaginationCursor::after_post(&post_view)
        .read(&mut conn.into())
        .await?;
      page_after = Some(cursor_data);
    } else {
      println!("👀 reached empty page");
      break;
    }
  }

  if let Ok(path) = std::env::var("PGDATA") {
    println!("🪵 query plans written in {path}/log");
  }

  Ok(())
}
