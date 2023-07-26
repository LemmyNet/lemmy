use crate::{fetcher::resolve_actor_identifier, objects::community::ApubCommunity};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{Search, SearchResponse},
  utils::{check_private_instance, is_admin, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::{
  source::{community::Community, local_site::LocalSite},
  utils::{post_to_comment_sort_type, post_to_person_sort_type},
  SearchType,
};
use lemmy_db_views::{comment_view::CommentQuery, post_view::PostQuery};
use lemmy_db_views_actor::{community_view::CommunityQuery, person_view::PersonQuery};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn search(
  data: Query<Search>,
  context: Data<LemmyContext>,
) -> Result<Json<SearchResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), &context).await;
  let local_site = LocalSite::read(context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  let is_admin = local_user_view.as_ref().map(|luv| is_admin(luv).is_ok());

  let mut posts = Vec::new();
  let mut comments = Vec::new();
  let mut communities = Vec::new();
  let mut users = Vec::new();

  // TODO no clean / non-nsfw searching rn

  let q = data.q.clone();
  let page = data.page;
  let limit = data.limit;
  let sort = data.sort;
  let listing_type = data.listing_type;
  let search_type = data.type_.unwrap_or(SearchType::All);
  let community_id = if let Some(name) = &data.community_name {
    Some(
      resolve_actor_identifier::<ApubCommunity, Community>(name, &context, &local_user_view, false)
        .await?,
    )
    .map(|c| c.id)
  } else {
    data.community_id
  };
  let creator_id = data.creator_id;
  let local_user = local_user_view.map(|l| l.local_user);
  match search_type {
    SearchType::Posts => {
      posts = PostQuery::builder()
        .pool(context.pool())
        .sort(sort)
        .listing_type(listing_type)
        .community_id(community_id)
        .creator_id(creator_id)
        .local_user(local_user.as_ref())
        .search_term(Some(q))
        .is_mod_or_admin(is_admin)
        .page(page)
        .limit(limit)
        .build()
        .list()
        .await?;
    }
    SearchType::Comments => {
      comments = CommentQuery::builder()
        .pool(context.pool())
        .sort(sort.map(post_to_comment_sort_type))
        .listing_type(listing_type)
        .search_term(Some(q))
        .community_id(community_id)
        .creator_id(creator_id)
        .local_user(local_user.as_ref())
        .page(page)
        .limit(limit)
        .build()
        .list()
        .await?;
    }
    SearchType::Communities => {
      communities = CommunityQuery::builder()
        .pool(context.pool())
        .sort(sort)
        .listing_type(listing_type)
        .search_term(Some(q))
        .local_user(local_user.as_ref())
        .is_mod_or_admin(is_admin)
        .page(page)
        .limit(limit)
        .build()
        .list()
        .await?;
    }
    SearchType::Users => {
      users = PersonQuery::builder()
        .pool(context.pool())
        .sort(sort.map(post_to_person_sort_type))
        .search_term(Some(q))
        .page(page)
        .limit(limit)
        .build()
        .list()
        .await?;
    }
    SearchType::All => {
      // If the community or creator is included, dont search communities or users
      let community_or_creator_included =
        data.community_id.is_some() || data.community_name.is_some() || data.creator_id.is_some();

      let local_user_ = local_user.clone();
      posts = PostQuery::builder()
        .pool(context.pool())
        .sort(sort)
        .listing_type(listing_type)
        .community_id(community_id)
        .creator_id(creator_id)
        .local_user(local_user_.as_ref())
        .search_term(Some(q))
        .is_mod_or_admin(is_admin)
        .page(page)
        .limit(limit)
        .build()
        .list()
        .await?;

      let q = data.q.clone();

      let local_user_ = local_user.clone();
      comments = CommentQuery::builder()
        .pool(context.pool())
        .sort(sort.map(post_to_comment_sort_type))
        .listing_type(listing_type)
        .search_term(Some(q))
        .community_id(community_id)
        .creator_id(creator_id)
        .local_user(local_user_.as_ref())
        .page(page)
        .limit(limit)
        .build()
        .list()
        .await?;

      let q = data.q.clone();

      communities = if community_or_creator_included {
        vec![]
      } else {
        CommunityQuery::builder()
          .pool(context.pool())
          .sort(sort)
          .listing_type(listing_type)
          .search_term(Some(q))
          .local_user(local_user.as_ref())
          .is_mod_or_admin(is_admin)
          .page(page)
          .limit(limit)
          .build()
          .list()
          .await?
      };

      let q = data.q.clone();

      users = if community_or_creator_included {
        vec![]
      } else {
        PersonQuery::builder()
          .pool(context.pool())
          .sort(sort.map(post_to_person_sort_type))
          .search_term(Some(q))
          .page(page)
          .limit(limit)
          .build()
          .list()
          .await?
      };
    }
    SearchType::Url => {
      posts = PostQuery::builder()
        .pool(context.pool())
        .sort(sort)
        .listing_type(listing_type)
        .community_id(community_id)
        .creator_id(creator_id)
        .url_search(Some(q))
        .is_mod_or_admin(is_admin)
        .page(page)
        .limit(limit)
        .build()
        .list()
        .await?;
    }
  };

  // Return the jwt
  Ok(Json(SearchResponse {
    type_: search_type,
    comments,
    posts,
    communities,
    users,
  }))
}
