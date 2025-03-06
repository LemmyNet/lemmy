use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{context::LemmyContext, post::BlockKeywordForPost, SuccessResponse};
use lemmy_db_schema::source::post_keyword_block::{PostKeywordBlock, PostKeywordBlockForm};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn user_block_keyword_for_posts(
  data: Json<BlockKeywordForPost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  if data.keyword.trim().len() < 3 {
    Err(LemmyErrorType::BlockKeywordToShort)?;
  }
  let person_id = local_user_view.person.id;
  let post_keyword_block_form = PostKeywordBlockForm {
    person_id,
    keyword: data.keyword.clone(),
  };
  let blocked_keywords = PostKeywordBlock::for_person(&mut context.pool(), person_id).await?;
  if data.block {
    //Get already blocked keywords and check if the limit is reached and also check if the keyword
    // is already blocked
    if blocked_keywords.iter().any(|k| k.keyword == data.keyword) {
      Err(LemmyErrorType::BlockKeywordAlreadyBlocked)?;
    } else if blocked_keywords.len() >= 15 {
      Err(LemmyErrorType::BlockKeywordLimitReached)?;
    }
    PostKeywordBlock::block_keyword(&mut context.pool(), &post_keyword_block_form).await?;
  } else {
    if !blocked_keywords.iter().any(|k| k.keyword == data.keyword) {
      Err(LemmyErrorType::BlockKeywordNotExisting)?;
    }
    PostKeywordBlock::unblock_keyword(&mut context.pool(), &post_keyword_block_form).await?;
  }
  Ok(Json(SuccessResponse::default()))
}
