use lemmy_api_common::{post::{BlockKeywordForPost}, context::LemmyContext, SuccessResponse};
use lemmy_db_views::structs::LocalUserView;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use lemmy_db_schema::source::post_keyword_block::{PostKeywordBlock, PostKeywordBlockForm};

pub async fn user_block_keyword_for_posts(
    data: Json<BlockKeywordForPost>,
    context: Data<LemmyContext>,
    local_user_view: LocalUserView
) -> LemmyResult<Json<SuccessResponse>>{
    if data.keyword.trim().len() < 3 {
        Err(LemmyErrorType::BlockKeywordToShort)?;
    }
    let person_id = local_user_view.person.id;
    let post_block_keyword_form = PostKeywordBlockForm {
        person_id,
        keyword: data.keyword.clone(),
    };
    if data.block {
        PostKeywordBlock::block_keyword(&mut context.pool(), &post_block_keyword_form).await?;
    } else {
        PostKeywordBlock::unblock_keyword(&mut context.pool(), &post_block_keyword_form).await?;
    }
    Ok(Json(SuccessResponse::default()))
}