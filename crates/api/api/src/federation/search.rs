use crate::federation::{
  fetcher::{resolve_community_identifier, resolve_person_identifier},
  resolve_object::resolve_object_internal,
};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use itertools::Itertools;
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
use lemmy_diesel_utils::pagination::PaginationCursor;
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
  let search_type = data.type_.unwrap_or_default();
  let listing_type = Some(ListingType::All);
  let search_title_only = data.title_only;
  let time_range_seconds = data.time_range_seconds;
  let search_url_only = data.post_url_only;
  let show_nsfw = data.show_nsfw;
  let page_cursors = from_single_cursor(data.page_cursor, search_type);
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
    page_cursor: page_cursors[0].clone(),
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
    sort: Some(CommentSortType::New),
    page_cursor: page_cursors[1].clone(),
    limit,
    ..Default::default()
  };

  let persons_query = PersonQuery {
    search_term: search_term.clone(),
    search_title_only,
    local_user,
    listing_type: Some(PersonListingType::All),
    sort: Some(PersonSortType::New),
    page_cursor: page_cursors[2].clone(),
    limit,
  };

  let communities_query = CommunityQuery {
    search_term: search_term.clone(),
    search_title_only,
    local_user,
    listing_type,
    time_range_seconds,
    show_nsfw,
    sort: Some(CommunitySortType::New),
    page_cursor: page_cursors[3].clone(),
    limit,
    ..Default::default()
  };

  let multi_communities_query = MultiCommunityQuery {
    search_term,
    search_title_only,
    creator_id,
    local_user,
    time_range_seconds,
    listing_type: Some(MultiCommunityListingType::All),
    sort: Some(MultiCommunitySortType::New),
    page_cursor: page_cursors[4].clone(),
    limit,
    ..Default::default()
  };

  let mut posts = Vec::new();
  let mut comments = Vec::new();
  let mut communities = Vec::new();
  let mut persons = Vec::new();
  let mut multi_communities = Vec::new();

  let mut next_page: [_; 5] = Default::default();
  let mut prev_page: [_; 5] = Default::default();

  let search_all = search_type == SearchType::All;

  // If the community or creator is included and it's All search, only search posts and comments
  let community_or_creator_included =
    data.community_id.is_some() || data.community_name.is_some() || data.creator_id.is_some();
  let search_all_no_community_or_creator = search_all && !community_or_creator_included;

  if (search_type == SearchType::Posts || search_all)
    && let Ok(x) = posts_query
      .list(&mut context.pool(), &site, &local_site)
      .await
  {
    posts = x.items;
    next_page[0] = x.next_page;
    prev_page[0] = x.prev_page;
  }
  if (search_type == SearchType::Comments || search_all)
    && let Ok(x) = comments_query
      .list(&mut context.pool(), &site, &local_site)
      .await
  {
    comments = x.items;
    next_page[1] = x.next_page;
    prev_page[1] = x.prev_page;
  }
  if (search_type == SearchType::Communities || search_all_no_community_or_creator)
    && let Ok(x) = communities_query
      .list(&mut context.pool(), &site, &local_site)
      .await
  {
    communities = x.items;
    next_page[2] = x.next_page;
    prev_page[2] = x.prev_page;
  }
  if (search_type == SearchType::Users || search_all_no_community_or_creator)
    && let Ok(x) = persons_query.list(&site, &mut context.pool()).await
  {
    persons = x.items;
    next_page[3] = x.next_page;
    prev_page[3] = x.prev_page;
  }
  if (search_type == SearchType::MultiCommunities || search_all_no_community_or_creator)
    && let Ok(x) = multi_communities_query.list(&mut context.pool()).await
  {
    multi_communities = x.items;
    next_page[4] = x.next_page;
    prev_page[4] = x.prev_page;
  }

  let res = SearchResponse {
    resolve,
    comments,
    posts,
    communities,
    multi_communities,
    persons,
    prev_page: to_single_cursor(prev_page, search_type),
    next_page: to_single_cursor(next_page, search_type),
  };

  Ok(Json(res))
}

fn to_single_cursor(
  cursors: [Option<PaginationCursor>; 5],
  search_type: SearchType,
) -> Option<String> {
  if search_type == SearchType::All {
    Some(
      cursors
        .into_iter()
        .map(|c| c.map(|c| c.0).unwrap_or("none".to_string()))
        .join(","),
    )
  } else {
    cursors
      .into_iter()
      .find(Option::is_some)
      .flatten()
      .map(|c| c.0)
  }
}

fn from_single_cursor(
  cursor: Option<String>,
  search_type: SearchType,
) -> [Option<PaginationCursor>; 5] {
  use SearchType::*;
  let mut res = [None, None, None, None, None];
  if cursor.is_none() {
    return res;
  };

  match search_type {
    All => {
      let vec = cursor
        .iter()
        .flat_map(|c| c.split(","))
        .map(|c| {
          if c == "none" {
            None
          } else {
            Some(PaginationCursor(c.to_string()))
          }
        })
        .collect::<Vec<Option<PaginationCursor>>>();
      let v: Result<[Option<PaginationCursor>; 5], _> = vec.try_into();
      if let Ok(v) = v {
        res = v;
      }
    }
    Posts => res[0] = cursor.map(Into::into),
    Comments => res[1] = cursor.map(Into::into),
    Users => res[2] = cursor.map(Into::into),
    Communities => res[3] = cursor.map(Into::into),
    MultiCommunities => res[4] = cursor.map(Into::into),
  };
  res
}

#[test]
fn test_from_single_cursor() {
  let a = None;
  let a_res = [None, None, None, None, None];
  assert_eq!(a_res, from_single_cursor(a, SearchType::All));

  let b = Some("a,b,c,d,e".to_string());
  let b_res = [
    Some(PaginationCursor("a".to_string())),
    Some(PaginationCursor("b".to_string())),
    Some(PaginationCursor("c".to_string())),
    Some(PaginationCursor("d".to_string())),
    Some(PaginationCursor("e".to_string())),
  ];
  assert_eq!(b_res, from_single_cursor(b, SearchType::All));

  let c = Some("none,b,none,none,none".to_string());
  let c_res = [
    None,
    Some(PaginationCursor("b".to_string())),
    None,
    None,
    None,
  ];
  assert_eq!(c_res, from_single_cursor(c, SearchType::All));

  let d = Some("p".to_string());
  let d_res = [
    Some(PaginationCursor("p".to_string())),
    None,
    None,
    None,
    None,
  ];
  assert_eq!(d_res, from_single_cursor(d, SearchType::Posts));
}

#[test]
fn test_to_single_cursor() {
  let a = [None, None, None, None, None];
  let a_res = Some("none,none,none,none,none".to_string());
  assert_eq!(a_res, to_single_cursor(a, SearchType::All));

  let b = [
    Some(PaginationCursor("a".to_string())),
    Some(PaginationCursor("b".to_string())),
    Some(PaginationCursor("c".to_string())),
    Some(PaginationCursor("d".to_string())),
    Some(PaginationCursor("e".to_string())),
  ];
  let b_res = Some("a,b,c,d,e".to_string());
  assert_eq!(b_res, to_single_cursor(b, SearchType::All));

  let c = [
    None,
    Some(PaginationCursor("b".to_string())),
    None,
    None,
    None,
  ];
  let c_res = Some("none,b,none,none,none".to_string());
  assert_eq!(c_res, to_single_cursor(c, SearchType::All));

  let d = [
    Some(PaginationCursor("p".to_string())),
    None,
    None,
    None,
    None,
  ];
  let d_res = Some("p".to_string());
  assert_eq!(d_res, to_single_cursor(d, SearchType::Posts));
}
