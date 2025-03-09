use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{context::LemmyContext, post::BlockKeywordForPost, SuccessResponse};
use lemmy_db_schema::source::user_post_keyword_block::UserPostKeywordBlock;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn user_block_keyword_for_posts(
  data: Json<BlockKeywordForPost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  for keyword in data.keywords_to_block.iter() {
    let trimmed = keyword.trim();
    let length = trimmed.len();
    if length < 3 {
        return Err(LemmyErrorType::BlockKeywordToShort.into());
    } else if length > 50 {
        return Err(LemmyErrorType::BlockKeywordToLong.into());
    }
  }
  if data.keywords_to_block.len() >= 15 {
    Err(LemmyErrorType::BlockKeywordLimitReached)?;
  }
  let person_id = local_user_view.person.id;
  UserPostKeywordBlock::update(
    &mut context.pool(),
    person_id,
    data.keywords_to_block.clone(),
  )
  .await?;
   Ok(Json(SuccessResponse::default()))
  }
