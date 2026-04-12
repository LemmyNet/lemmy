use crate::federation::{
  fetcher::{resolve_community_identifier, resolve_person_identifier},
  resolve_object::resolve_object_internal,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_schema::{
  CommunitySortType,
  MultiCommunityListingType,
  MultiCommunitySortType,
  PersonListingType,
  PersonSortType,
  SearchType,
};
use lemmy_db_schema_file::enums::{CommentSortType, ListingType};
use lemmy_db_views_comment::impls::CommentQuery;
use lemmy_db_views_community::impls::{CommunityQuery, MultiCommunityQuery};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::impls::PersonQuery;
use lemmy_db_views_post::impls::PostQuery;
use lemmy_db_views_site::{
  SiteView,
  api::{Search, SearchResponse},
};
use lemmy_utils::error::LemmyResult;

pub async fn search(
  Query(data): Query<Search>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<SearchResponse>> {
  let SiteView {
    local_site, site, ..
  } = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  let resolve = resolve_object_internal(&data.search_term, &local_user_view, &context)
    .await
    .ok();

  let search_term = Some(data.search_term);
  let listing_type = Some(ListingType::All);
  let search_title_only = data.title_only;
  let time_range_seconds = data.time_range_seconds;
  let search_url_only = data.post_url_only;
  let show_nsfw = data.show_nsfw;
  let limit = data.limit;

  let community_id = resolve_community_identifier(
    &data.community_name,
    data.community_id,
    &context,
    &local_user_view,
  )
  .await?;

  let creator_id = resolve_person_identifier(
    data.creator_id,
    &data.creator_username,
    &context,
    &local_user_view,
  )
  .await?;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);

  let posts_query = PostQuery {
    search_term: search_term.clone(),
    search_title_only,
    local_user,
    listing_type,
    community_id,
    creator_id,
    time_range_seconds,
    search_url_only,
    show_nsfw,
    limit,
    ..Default::default()
  };

  let comments_query = CommentQuery {
    search_term: search_term.clone(),
    local_user,
    listing_type,
    community_id,
    creator_id,
    time_range_seconds,
    limit,
    sort: Some(CommentSortType::New),
    ..Default::default()
  };

  let persons_query = PersonQuery {
    search_term: search_term.clone(),
    search_title_only,
    local_user,
    limit,
    listing_type: Some(PersonListingType::All),
    sort: Some(PersonSortType::New),
    ..Default::default()
  };

  let communities_query = CommunityQuery {
    search_term: search_term.clone(),
    search_title_only,
    local_user,
    listing_type,
    time_range_seconds,
    show_nsfw,
    limit,
    sort: Some(CommunitySortType::New),
    ..Default::default()
  };

  let multi_communities_query = MultiCommunityQuery {
    search_term,
    search_title_only,
    creator_id,
    local_user,
    time_range_seconds,
    limit,
    listing_type: Some(MultiCommunityListingType::All),
    sort: Some(MultiCommunitySortType::New),
    ..Default::default()
  };

  let mut posts = Vec::new();
  let mut comments = Vec::new();
  let mut communities = Vec::new();
  let mut persons = Vec::new();
  let mut multi_communities = Vec::new();

  match data.type_.unwrap_or_default() {
    SearchType::Posts => {
      posts = posts_query
        .list(&mut context.pool(), &site, &local_site)
        .await?
        .items;
    }
    SearchType::Comments => {
      comments = comments_query.list(&site, &mut context.pool()).await?.items;
    }
    SearchType::Communities => {
      communities = communities_query
        .list(&site, &mut context.pool())
        .await?
        .items;
    }
    SearchType::Users => {
      persons = persons_query.list(&site, &mut context.pool()).await?.items;
    }
    SearchType::MultiCommunities => {
      multi_communities = multi_communities_query
        .list(&mut context.pool())
        .await?
        .items;
    }
    SearchType::All => {
      // If the community or creator is included, dont search communities or users
      let community_or_creator_included =
        data.community_id.is_some() || data.community_name.is_some(); // || data.creator_id.is_some();

      posts = posts_query
        .list(&mut context.pool(), &site, &local_site)
        .await?
        .items;

      comments = comments_query.list(&site, &mut context.pool()).await?.items;

      if !community_or_creator_included {
        communities = communities_query
          .list(&site, &mut context.pool())
          .await?
          .items;
        persons = persons_query.list(&site, &mut context.pool()).await?.items;
      };
    }
  }

  let res = SearchResponse {
    resolve,
    comments,
    posts,
    communities,
    multi_communities,
    persons,
  };

  Ok(Json(res))
}
