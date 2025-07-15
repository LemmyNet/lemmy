use crate::{
  diesel::ExpressionMethods,
  newtypes::{CommentId, PostId},
  source::{
    combined::person_content::{
      PersonContentCombined,
      PersonContentCombinedCommentInsertForm,
      PersonContentCombinedPostInsertForm,
    },
    history_status::{HistoryStatus, HistoryStatusUpdateForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::{DateTime, Utc};
use diesel::{dsl::count_star, insert_into, upsert::excluded, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, RunQueryDsl};
use lemmy_db_schema_file::schema::{comment, person_content_combined, post};
use lemmy_utils::{error::LemmyResult, DB_BATCH_SIZE};
use tracing::info;

impl PersonContentCombined {
  pub async fn fill_post_history(pool: &mut DbPool<'_>) -> LemmyResult<()> {
    info!("Filling post history into person_content_combined...");

    // Get the total count of post rows, to show progress
    // Since you aren't deleting any rows, you need to use < last_scanned_id
    let history_key = || ("post".into(), "person_content_combined".into());
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
              .select((post::id, post::published_at))
              .filter(post::id.lt(last_scanned_id))
              .order_by(post::id.desc())
              .limit(DB_BATCH_SIZE.try_into()?)
              .get_results::<(PostId, DateTime<Utc>)>(conn)
              .await?
              .iter()
              .map(|cl| PersonContentCombinedPostInsertForm {
                post_id: cl.0,
                published_at: cl.1,
              })
              .collect::<Vec<PersonContentCombinedPostInsertForm>>();

            // Can't reuse this, as it gets moved internally. Need to return it and assign outside
            let last_scanned_id_internal =
              forms.last().map(|f| f.post_id).unwrap_or(PostId(i32::MAX));

            let inserted_count = insert_into(person_content_combined::table)
              .values(forms)
              .on_conflict(person_content_combined::post_id)
              .do_update()
              .set(
                person_content_combined::published_at
                  .eq(excluded(person_content_combined::published_at)),
              )
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
      info!(
        "post -> person_content_combined: {processed_rows} / {post_count} , {pct_complete}% complete"
      );
    }

    info!("Finished filling post history into person_content_combined.");
    Ok(())
  }

  pub async fn fill_comment_history(pool: &mut DbPool<'_>) -> LemmyResult<()> {
    info!("Filling comment history into person_content_combined...");

    // Get the total count of rows, to show progress
    // Since you aren't deleting any rows, you need to use < last_scanned_id
    let history_key = || ("comment".into(), "person_content_combined".into());
    let mut last_scanned_id = CommentId(
      HistoryStatus::read(pool, history_key())
        .await?
        .last_scanned_id
        .unwrap_or(i32::MAX),
    );

    let conn = &mut get_conn(pool).await?;

    // Get the total count of comment rows, to show progress
    let comment_count = comment::table
      .filter(comment::id.lt(last_scanned_id))
      .select(count_star())
      .first::<i64>(conn)
      .await?;

    let mut processed_rows = 0;

    while processed_rows < comment_count {
      let (rows_inserted, last_scanned_id_out) = conn
        .run_transaction(|conn| {
          async move {
            // Select and map into comment like forms
            let forms = comment::table
              .select((comment::id, comment::published_at))
              .filter(comment::id.lt(last_scanned_id))
              .order_by(comment::id.desc())
              .limit(DB_BATCH_SIZE.try_into()?)
              .get_results::<(CommentId, DateTime<Utc>)>(conn)
              .await?
              .iter()
              .map(|cl| PersonContentCombinedCommentInsertForm {
                comment_id: cl.0,
                published_at: cl.1,
              })
              .collect::<Vec<PersonContentCombinedCommentInsertForm>>();

            // Can't reuse this, as it gets moved internally. Need to return it and assign outside
            let last_scanned_id_internal = forms
              .last()
              .map(|f| f.comment_id)
              .unwrap_or(CommentId(i32::MAX));

            let inserted_count = insert_into(person_content_combined::table)
              .values(forms)
              .on_conflict(person_content_combined::comment_id)
              .do_update()
              .set(
                person_content_combined::published_at
                  .eq(excluded(person_content_combined::published_at)),
              )
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
        "comment -> person_content_combined: {processed_rows} / {comment_count} , {pct_complete}% complete"
      );
    }

    info!("Finished filling comment history into person_content_combined.");
    Ok(())
  }
}
