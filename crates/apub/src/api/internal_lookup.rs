use activitypub_federation::config::Data;
use lemmy_api_common::{
    internal_lookup::{
        InternalLookupRequest, 
        InternalLookupResponse, 
        InternalLookupType
    }, 
    context::LemmyContext, 
    utils::{
        local_user_view_from_jwt_opt, 
        check_private_instance
    }
};
use actix_web::web::{Json, Query};
use lemmy_db_schema::source::{
    comment::Comment,
    post::Post, 
    local_site::LocalSite
};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn internal_lookup(
  data: Query<InternalLookupRequest>,
  context: Data<LemmyContext>,
) -> Result<Json<InternalLookupResponse>, LemmyError> {

    let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), &context).await;
    let local_site = LocalSite::read(&mut context.pool()).await?;

    check_private_instance(&local_user_view, &local_site)?;

    let actor_id = (*data.actor_id).to_owned();
    let lookup_type = data.lookup_type.to_owned();

    // In theory we could use the URL itself to determine the actor's type, but the client should already
    // know this before making the request.  Also, as different software comes online that each have their
    // own unique ids for posts and comments, those would have to  be added here.
    let internal_id = match lookup_type {
        InternalLookupType::Comment => {
            // Is there a better way to do this?
            Comment::read_from_apub_id(&mut context.pool(), actor_id)
                .await?
                .map(|c| {
                    c.id.0
                })
        },
        InternalLookupType::Post => {
            Post::read_from_apub_id(&mut context.pool(), actor_id)
                .await?
                .map(|p| {
                    p.id.0
                })
        }
    };

    // Do we still want to return the internal_ids for deleted posts?

    Ok(Json(InternalLookupResponse { internal_id }))
}
