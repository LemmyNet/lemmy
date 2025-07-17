use crate::{
  diesel::ExpressionMethods,
  newtypes::{CommentId, CommunityId, PersonId, PostId},
  source::{
    combined::search::{
      SearchCombined,
      SearchCombinedCommentInsertForm,
      SearchCombinedCommunityInsertForm,
      SearchCombinedPersonInsertForm,
      SearchCombinedPostInsertForm,
    },
    history_status::{HistoryStatus, HistoryStatusUpdateForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::{DateTime, Utc};
use diesel::{dsl::count_star, insert_into, upsert::excluded, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use lemmy_db_schema_file::schema::{comment, community, person, post, search_combined};
use lemmy_utils::{error::LemmyResult, DB_BATCH_SIZE};
use tracing::info;

impl SearchCombined {
  pub async fn fill_post_history(pool: &mut DbPool<'_>) -> LemmyResult<()> {
    info!("Filling post history into search_combined...");

    // Get the total count of post rows, to show progress
    // Since you aren't deleting any rows, you need to use < last_scanned_id
    let history_key = || ("post".into(), "search_combined".into());
    let mut last_scanned_id = PostId(
      HistoryStatus::read(pool, history_key())
        .await?
        .last_scanned_id
        .unwrap_or(i32::MAX),
    );

    let conn = &mut get_conn(pool).await?;
    let post_count = post::table
      .select(count_star())
      .filter(post::id.lt(last_scanned_id))
      .first::<i64>(conn)
      .await?;

    let mut processed_rows = 0;

    while processed_rows < post_count {
      let (rows_inserted, last_scanned_id_out) = conn
        .run_transaction(|conn| {
          async move {
            // Select and map into comment like forms
            let forms = post::table
              .select((post::published_at, post::score, post::id))
              .filter(post::id.lt(last_scanned_id))
              .order_by(post::id.desc())
              .limit(DB_BATCH_SIZE.try_into()?)
              .get_results::<(DateTime<Utc>, i32, PostId)>(conn)
              .await?
              .iter()
              .map(|cl| SearchCombinedPostInsertForm {
                published_at: cl.0,
                score: cl.1,
                post_id: cl.2,
              })
              .collect::<Vec<SearchCombinedPostInsertForm>>();

            // Can't reuse this, as it gets moved internally. Need to return it and assign outside
            let last_scanned_id_internal =
              forms.last().map(|f| f.post_id).unwrap_or(PostId(i32::MAX));

            let inserted_count = insert_into(search_combined::table)
              .values(forms)
              .on_conflict(search_combined::post_id)
              .do_update()
              .set((
                search_combined::published_at.eq(excluded(search_combined::published_at)),
                search_combined::score.eq(excluded(search_combined::score)),
              ))
              .execute(conn)
              .await?;

            // Update the history status
            let history_form = HistoryStatusUpdateForm {
              last_scanned_timestamp: None,
              last_scanned_id: Some(Some(last_scanned_id_internal.0)),
            };
            HistoryStatus::update_conn(conn, history_key(), &history_form).await?;

            Ok((inserted_count, last_scanned_id_internal))
          }
          .scope_boxed()
        })
        .await?;

      last_scanned_id = last_scanned_id_out;
      processed_rows += i64::try_from(rows_inserted)?;
      let pct_complete = processed_rows * 100 / post_count;
      info!("post -> search_combined: {processed_rows} / {post_count} , {pct_complete}% complete");
    }

    info!("Finished filling post history into search_combined.");
    Ok(())
  }

  pub async fn fill_comment_history(pool: &mut DbPool<'_>) -> LemmyResult<()> {
    info!("Filling comment history into search_combined...");

    // Get the total count of comment rows, to show progress
    // Since you aren't deleting any rows, you need to use < last_scanned_id
    let history_key = || ("comment".into(), "search_combined".into());
    let mut last_scanned_id = CommentId(
      HistoryStatus::read(pool, history_key())
        .await?
        .last_scanned_id
        .unwrap_or(i32::MAX),
    );

    let conn = &mut get_conn(pool).await?;
    let comment_count = comment::table
      .select(count_star())
      .filter(comment::id.lt(last_scanned_id))
      .first::<i64>(conn)
      .await?;

    let mut processed_rows = 0;

    while processed_rows < comment_count {
      let (rows_inserted, last_scanned_id_out) = conn
        .run_transaction(|conn| {
          async move {
            // Select and map into comment like forms
            let forms = comment::table
              .select((comment::published_at, comment::score, comment::id))
              .filter(comment::id.lt(last_scanned_id))
              .order_by(comment::id.desc())
              .limit(DB_BATCH_SIZE.try_into()?)
              .get_results::<(DateTime<Utc>, i32, CommentId)>(conn)
              .await?
              .iter()
              .map(|cl| SearchCombinedCommentInsertForm {
                published_at: cl.0,
                score: cl.1,
                comment_id: cl.2,
              })
              .collect::<Vec<SearchCombinedCommentInsertForm>>();

            // Can't reuse this, as it gets moved internally. Need to return it and assign outside
            let last_scanned_id_internal = forms
              .last()
              .map(|f| f.comment_id)
              .unwrap_or(CommentId(i32::MAX));

            let inserted_count = insert_into(search_combined::table)
              .values(forms)
              .on_conflict(search_combined::comment_id)
              .do_update()
              .set((
                search_combined::published_at.eq(excluded(search_combined::published_at)),
                search_combined::score.eq(excluded(search_combined::score)),
              ))
              .execute(conn)
              .await?;

            // Update the history status
            let history_form = HistoryStatusUpdateForm {
              last_scanned_timestamp: None,
              last_scanned_id: Some(Some(last_scanned_id_internal.0)),
            };
            HistoryStatus::update_conn(conn, history_key(), &history_form).await?;

            Ok((inserted_count, last_scanned_id_internal))
          }
          .scope_boxed()
        })
        .await?;

      last_scanned_id = last_scanned_id_out;
      processed_rows += i64::try_from(rows_inserted)?;
      let pct_complete = processed_rows * 100 / comment_count;
      info!(
        "comment -> search_combined: {processed_rows} / {comment_count} , {pct_complete}% complete"
      );
    }

    info!("Finished filling comment history into search_combined.");
    Ok(())
  }

  pub async fn fill_community_history(pool: &mut DbPool<'_>) -> LemmyResult<()> {
    info!("Filling community history into search_combined...");

    // Get the total count of community rows, to show progress
    // Since you aren't deleting any rows, you need to use < last_scanned_id
    let history_key = || ("community".into(), "search_combined".into());
    let mut last_scanned_id = CommunityId(
      HistoryStatus::read(pool, history_key())
        .await?
        .last_scanned_id
        .unwrap_or(i32::MAX),
    );

    let conn = &mut get_conn(pool).await?;
    let community_count = community::table
      .select(count_star())
      .filter(community::id.lt(last_scanned_id))
      .first::<i64>(conn)
      .await?;

    let mut processed_rows = 0;

    while processed_rows < community_count {
      let (rows_inserted, last_scanned_id_out) = conn
        .run_transaction(|conn| {
          async move {
            // Select and map into community like forms
            let forms = community::table
              .select((
                community::published_at,
                community::users_active_month,
                community::id,
              ))
              .filter(community::id.lt(last_scanned_id))
              .order_by(community::id.desc())
              .limit(DB_BATCH_SIZE.try_into()?)
              .get_results::<(DateTime<Utc>, i32, CommunityId)>(conn)
              .await?
              .iter()
              .map(|cl| SearchCombinedCommunityInsertForm {
                published_at: cl.0,
                score: cl.1,
                community_id: cl.2,
              })
              .collect::<Vec<SearchCombinedCommunityInsertForm>>();

            // Can't reuse this, as it gets moved internally. Need to return it and assign outside
            let last_scanned_id_internal = forms
              .last()
              .map(|f| f.community_id)
              .unwrap_or(CommunityId(i32::MAX));

            let inserted_count = insert_into(search_combined::table)
              .values(forms)
              .on_conflict(search_combined::community_id)
              .do_update()
              .set((
                search_combined::published_at.eq(excluded(search_combined::published_at)),
                search_combined::score.eq(excluded(search_combined::score)),
              ))
              .execute(conn)
              .await?;

            // Update the history status
            let history_form = HistoryStatusUpdateForm {
              last_scanned_timestamp: None,
              last_scanned_id: Some(Some(last_scanned_id_internal.0)),
            };
            HistoryStatus::update_conn(conn, history_key(), &history_form).await?;

            Ok((inserted_count, last_scanned_id_internal))
          }
          .scope_boxed()
        })
        .await?;

      last_scanned_id = last_scanned_id_out;
      processed_rows += i64::try_from(rows_inserted)?;
      let pct_complete = processed_rows * 100 / community_count;
      info!(
        "community -> search_combined: {processed_rows} / {community_count} , {pct_complete}% complete"
      );
    }

    info!("Finished filling community history into search_combined.");
    Ok(())
  }

  pub async fn fill_person_history(pool: &mut DbPool<'_>) -> LemmyResult<()> {
    info!("Filling person history into search_combined...");

    // Get the total count of person rows, to show progress
    // Since you aren't deleting any rows, you need to use < last_scanned_id
    let history_key = || ("person".into(), "search_combined".into());
    let mut last_scanned_id = PersonId(
      HistoryStatus::read(pool, history_key())
        .await?
        .last_scanned_id
        .unwrap_or(i32::MAX),
    );

    let conn = &mut get_conn(pool).await?;
    let person_count = person::table
      .select(count_star())
      .filter(person::id.lt(last_scanned_id))
      .first::<i64>(conn)
      .await?;

    let mut processed_rows = 0;

    while processed_rows < person_count {
      let (rows_inserted, last_scanned_id_out) = conn
        .run_transaction(|conn| {
          async move {
            // Select and map into person like forms
            let forms = person::table
              .select((person::published_at, person::post_score, person::id))
              .filter(person::id.lt(last_scanned_id))
              .order_by(person::id.desc())
              .limit(DB_BATCH_SIZE.try_into()?)
              .get_results::<(DateTime<Utc>, i32, PersonId)>(conn)
              .await?
              .iter()
              .map(|cl| SearchCombinedPersonInsertForm {
                published_at: cl.0,
                score: cl.1,
                person_id: cl.2,
              })
              .collect::<Vec<SearchCombinedPersonInsertForm>>();

            // Can't reuse this, as it gets moved internally. Need to return it and assign outside
            let last_scanned_id_internal = forms
              .last()
              .map(|f| f.person_id)
              .unwrap_or(PersonId(i32::MAX));

            let inserted_count = insert_into(search_combined::table)
              .values(forms)
              .on_conflict(search_combined::person_id)
              .do_update()
              .set((
                search_combined::published_at.eq(excluded(search_combined::published_at)),
                search_combined::score.eq(excluded(search_combined::score)),
              ))
              .execute(conn)
              .await?;

            // Update the history status
            let history_form = HistoryStatusUpdateForm {
              last_scanned_timestamp: None,
              last_scanned_id: Some(Some(last_scanned_id_internal.0)),
            };
            HistoryStatus::update_conn(conn, history_key(), &history_form).await?;

            Ok((inserted_count, last_scanned_id_internal))
          }
          .scope_boxed()
        })
        .await?;

      last_scanned_id = last_scanned_id_out;
      processed_rows += i64::try_from(rows_inserted)?;
      let pct_complete = processed_rows * 100 / person_count;
      info!(
        "person -> search_combined: {processed_rows} / {person_count} , {pct_complete}% complete"
      );
    }

    info!("Finished filling person history into search_combined.");
    Ok(())
  }
}
