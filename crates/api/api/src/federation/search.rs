use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::{context::LemmyContext, utils::check_private_instance};
use lemmy_db_schema::{
  CommunitySortType,
  MultiCommunityListingType,
  MultiCommunitySortType,
  PersonListingType,
  PersonSortType,
};
use lemmy_db_schema_file::enums::{CommentSortType, ListingType, PostSortType};
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

  let search_term = Some(data.search_term);
  let listing_type = Some(ListingType::All);
  let search_title_only = data.search_title_only;

  check_private_instance(&local_user_view, &local_site)?;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);

  let posts = PostQuery {
    search_term: search_term.clone(),
    search_title_only,
    local_user,
    listing_type,
    sort: Some(PostSortType::New),
    ..Default::default()
  }
  .list(&mut context.pool(), &site, &local_site)
  .await?
  .items;

  let comments = CommentQuery {
    search_term: search_term.clone(),
    local_user,
    listing_type,
    sort: Some(CommentSortType::New),
    ..Default::default()
  }
  .list(&site, &mut context.pool())
  .await?
  .items;

  let persons = PersonQuery {
    search_term: search_term.clone(),
    search_title_only,
    local_user,
    listing_type: Some(PersonListingType::All),
    sort: Some(PersonSortType::New),
    ..Default::default()
  }
  .list(&site, &mut context.pool())
  .await?
  .items;

  let communities = CommunityQuery {
    search_term: search_term.clone(),
    search_title_only,
    local_user,
    listing_type,
    sort: Some(CommunitySortType::New),
    ..Default::default()
  }
  .list(&site, &mut context.pool())
  .await?
  .items;

  let multi_communities = MultiCommunityQuery {
    search_term,
    search_title_only,
    local_user,
    listing_type: Some(MultiCommunityListingType::All),
    sort: Some(MultiCommunitySortType::New),
    ..Default::default()
  }
  .list(&mut context.pool())
  .await?
  .items;

  let res = SearchResponse {
    comments,
    posts,
    communities,
    multi_communities,
    persons,
  };

  Ok(Json(res))
}
