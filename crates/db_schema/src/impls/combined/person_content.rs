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
    let conn = &mut get_conn(pool).await?;

    info!("Filling post history into person_content_combined...");

    // Get the total count of post rows, to show progress
    let post_count = post::table.select(count_star()).first::<i64>(conn).await?;

    let mut processed_rows = 0;

    while processed_rows < post_count {
      let rows_inserted = conn
        .run_transaction(|conn| {
          async move {
            // Select and map into comment like forms
            let forms = post::table
              .select((post::id, post::published_at))
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

            // When this is None, the scanning is complete
            let last_scanned_id = forms.last().map(|f| f.post_id);

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

            if let Some(last_scanned_id) = last_scanned_id {
              // Update the history status
              let history_form = HistoryStatusUpdateForm {
                last_scanned_timestamp: None,
                last_scanned_id: Some(Some(last_scanned_id.0)),
              };
              HistoryStatus::update_conn(
                conn,
                ("post".into(), "person_content_combined".into()),
                &history_form,
              )
              .await?;
            }

            Ok(inserted_count)
          }
          .scope_boxed()
        })
        .await?;

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
    let conn = &mut get_conn(pool).await?;

    info!("Filling comment history into person_content_combined...");

    // Get the total count of comment rows, to show progress
    let comment_count = comment::table
      .select(count_star())
      .first::<i64>(conn)
      .await?;

    let mut processed_rows = 0;

    while processed_rows < comment_count {
      let rows_inserted = conn
        .run_transaction(|conn| {
          async move {
            // Select and map into comment like forms
            let forms = comment::table
              .select((comment::id, comment::published_at))
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

            // When this is None, the scanning is complete
            let last_scanned_id = forms.last().map(|f| f.comment_id);

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

            if let Some(last_scanned_id) = last_scanned_id {
              // Update the history status
              let history_form = HistoryStatusUpdateForm {
                last_scanned_timestamp: None,
                last_scanned_id: Some(Some(last_scanned_id.0)),
              };
              HistoryStatus::update_conn(
                conn,
                ("comment".into(), "person_content_combined".into()),
                &history_form,
              )
              .await?;
            }

            Ok(inserted_count)
          }
          .scope_boxed()
        })
        .await?;

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
