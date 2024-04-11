use crate::{fetcher::resolve_actor_identifier, objects::community::ApubCommunity};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{Search, SearchResponse},
  utils::{check_private_instance, is_admin},
};
use lemmy_db_schema::{source::community::Community, utils::post_to_comment_sort_type, SearchType};
use lemmy_db_views::{
  comment_view::CommentQuery,
  post_view::PostQuery,
  structs::{LocalUserView, SiteView},
};
use lemmy_db_views_actor::{community_view::CommunityQuery, person_view::PersonQuery};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn search(
  data: Query<Search>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<SearchResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site.local_site)?;

  let is_admin = local_user_view
    .as_ref()
    .map(|luv| is_admin(luv).is_ok())
    .unwrap_or_default();

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
  let local_user = local_user_view.as_ref().map(|luv| &luv.local_user);

  match search_type {
    SearchType::Posts => {
      posts = PostQuery {
        sort: (sort),
        listing_type: (listing_type),
        community_id: (community_id),
        creator_id: (creator_id),
        local_user: (local_user_view.as_ref()),
        search_term: (Some(q)),
        page: (page),
        limit: (limit),
        ..Default::default()
      }
      .list(&local_site.site, &mut context.pool())
      .await?;
    }
    SearchType::Comments => {
      comments = CommentQuery {
        sort: (sort.map(post_to_comment_sort_type)),
        listing_type: (listing_type),
        search_term: (Some(q)),
        community_id: (community_id),
        creator_id: (creator_id),
        local_user: (local_user_view.as_ref()),
        page: (page),
        limit: (limit),
        ..Default::default()
      }
      .list(&mut context.pool())
      .await?;
    }
    SearchType::Communities => {
      communities = CommunityQuery {
        sort: (sort),
        listing_type: (listing_type),
        search_term: (Some(q)),
        local_user,
        is_mod_or_admin: (is_admin),
        page: (page),
        limit: (limit),
        ..Default::default()
      }
      .list(&local_site.site, &mut context.pool())
      .await?;
    }
    SearchType::Users => {
      users = PersonQuery {
        sort,
        search_term: (Some(q)),
        listing_type: (listing_type),
        page: (page),
        limit: (limit),
      }
      .list(&mut context.pool())
      .await?;
    }
    SearchType::All => {
      // If the community or creator is included, dont search communities or users
      let community_or_creator_included =
        data.community_id.is_some() || data.community_name.is_some() || data.creator_id.is_some();

      let q = data.q.clone();

      posts = PostQuery {
        sort: (sort),
        listing_type: (listing_type),
        community_id: (community_id),
        creator_id: (creator_id),
        local_user: (local_user_view.as_ref()),
        search_term: (Some(q)),
        page: (page),
        limit: (limit),
        ..Default::default()
      }
      .list(&local_site.site, &mut context.pool())
      .await?;

      let q = data.q.clone();

      comments = CommentQuery {
        sort: (sort.map(post_to_comment_sort_type)),
        listing_type: (listing_type),
        search_term: (Some(q)),
        community_id: (community_id),
        creator_id: (creator_id),
        local_user: (local_user_view.as_ref()),
        page: (page),
        limit: (limit),
        ..Default::default()
      }
      .list(&mut context.pool())
      .await?;

      let q = data.q.clone();

      communities = if community_or_creator_included {
        vec![]
      } else {
        CommunityQuery {
          sort: (sort),
          listing_type: (listing_type),
          search_term: (Some(q)),
          local_user,
          is_mod_or_admin: (is_admin),
          page: (page),
          limit: (limit),
          ..Default::default()
        }
        .list(&local_site.site, &mut context.pool())
        .await?
      };

      let q = data.q.clone();

      users = if community_or_creator_included {
        vec![]
      } else {
        PersonQuery {
          sort,
          search_term: (Some(q)),
          listing_type: (listing_type),
          page: (page),
          limit: (limit),
        }
        .list(&mut context.pool())
        .await?
      };
    }
    SearchType::Url => {
      posts = PostQuery {
        sort: (sort),
        listing_type: (listing_type),
        community_id: (community_id),
        creator_id: (creator_id),
        url_search: (Some(q)),
        page: (page),
        limit: (limit),
        ..Default::default()
      }
      .list(&local_site.site, &mut context.pool())
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
