use actix_web::web::{Data, Json};
use lemmy_api_common::{
    context::LemmyContext,
    person::{GetRepliesResponse, MarkAllAsRead},
    utils::local_user_view_from_jwt,
};
use lemmy_db_schema::source::{
    comment_reply::CommentReply, person_mention::PersonMention, private_message::PrivateMessage,
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn mark_all_notifications_read(
    data: Json<MarkAllAsRead>,
    context: Data<LemmyContext>,
) -> Result<Json<GetRepliesResponse>, LemmyError> {
    let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;
    let person_id = local_user_view.person.id;

    // Mark all comment_replies as read
    CommentReply::mark_all_as_read(&mut context.pool(), person_id)
        .await
        .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

    // Mark all user mentions as read
    PersonMention::mark_all_as_read(&mut context.pool(), person_id)
        .await
        .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

    // Mark all private_messages as read
    PrivateMessage::mark_all_as_read(&mut context.pool(), person_id)
        .await
        .with_lemmy_type(LemmyErrorType::CouldntUpdatePrivateMessage)?;

    Ok(Json(GetRepliesResponse { replies: vec![] }))
}
