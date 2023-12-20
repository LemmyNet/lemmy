use clap::{Parser, Subcommand};
use diesel::{dsl::{self, sql}, sql_query, sql_types, ExpressionMethods, IntoSql};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  schema::post,
  source::{
    community::{Community, CommunityInsertForm},
    instance::Instance,
    person::{Person, PersonInsertForm},
  },
  traits::Crud,
  utils::{
    build_db_pool,
    get_conn,
    series::{self, ValuesFromSeries}, DbConn, DbPool, now,
  },
  SortType,
};
use lemmy_db_views::{
  post_view::{PaginationCursorData, PostQuery},
  structs::PaginationCursor,
};
use lemmy_utils::error::LemmyResult;
use std::num::NonZeroU32;
use diesel::pg::expression::dsl::IntervalDsl;

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
async fn main() -> LemmyResult<()> {
  let args = CmdArgs::parse();
  let pool = &build_db_pool().await?;
  let pool = &mut pool.into();

  let conn = &mut get_conn(pool).await?;
  if args.explain_insertions {
    sql_query("SET auto_explain.log_min_duration = 0")
      .execute(conn)
      .await?;
  }
  let pool = &mut conn.into();

  let instance = Instance::read_or_create(pool, "reddit.com".to_owned()).await?;

  println!("🫃 creating {} people", args.people);
  let mut person_ids = vec![];
  for i in 0..args.people.get() {
    person_ids.push(
      Person::create(
        pool,
        &PersonInsertForm::builder()
          .name(format!("p{i}"))
          .public_key("pubkey".to_owned())
          .instance_id(instance.id)
          .build(),
      )
      .await?
      .id,
    );
  }

  println!("🏠 creating {} communities", args.communities);
  let mut community_ids = vec![];
  for i in 0..args.communities.get() {
    community_ids.push(
      Community::create(
        pool,
        &CommunityInsertForm::builder()
          .name(format!("c{i}"))
          .title(i.to_string())
          .instance_id(instance.id)
          .build(),
      )
      .await?
      .id,
    );
  }

  let post_batches = args.people.get() * args.communities.get();
  let posts_per_batch = args.posts.get() / post_batches;
  let num_posts = post_batches * posts_per_batch;
  println!(
    "📢 creating {} posts ({} featured in community)",
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
            now() - sql::<sql_types::Interval>("make_interval(secs => ").bind::<sql_types::BigInt, _>(series::current_value).sql(")"),
          ),
        })
        .into_columns((
          post::name,
          post::creator_id,
          post::community_id,
          post::featured_community,
          post::published,
        ))
        .execute(&mut get_conn(pool).await?)
        .await?;
      num_inserted_posts += n;
    }
  }

  // Lie detector for the println above
  assert_eq!(num_inserted_posts, num_posts as usize);

  // Enable auto_explain
  let conn = &mut get_conn(pool).await?;
  sql_query("SET auto_explain.log_min_duration = 0")
    .execute(conn)
    .await?;
  let pool = &mut conn.into();
  
  {
    let mut page_after = None;
    for page_num in 1..=args.read_post_pages {
      println!(
        "👀 getting page {page_num} of posts (pagination cursor used: {})",
        page_after.is_some()
      );
      // TODO: include local_user
      let post_views = PostQuery {
        community_id: community_ids.get(0).cloned(),
        sort: Some(SortType::New),
        limit: Some(20),
        page_after,
        ..Default::default()
      }
      .list(pool)
      .await?;
    if let Some(post_view) = post_views.into_iter().last() {
      println!("👀 getting pagination cursor data for next page");
      let cursor_data = PaginationCursor::after_post(&post_view).read(pool).await?;
      page_after = Some(cursor_data);
    } else {
      println!("🚫 reached empty page");
      break;
    }
    }
  }
  
  // TODO show this path when there's an error
  if let Ok(path) = std::env::var("PGDATA") {
    println!("🪵 query plans written in {path}/log");
  }
  
  Ok(())
}

async fn conn_with_auto_explain<'a, 'b: 'a>(pool: &'a mut DbPool<'b>) -> LemmyResult<DbConn<'a>> {
  let mut conn = get_conn(pool).await?;

  sql_query("SET auto_explain.log_min_duration = 0")
    .execute(&mut conn)
    .await?;

  Ok(conn)
}
