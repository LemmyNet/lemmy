use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_private_instance,
  get_local_user_view_from_jwt_opt,
  site::{Search, SearchResponse},
};
use lemmy_apub::{fetcher::resolve_actor_identifier, objects::community::ApubCommunity};
use lemmy_db_schema::{
  from_opt_str_to_opt_enum,
  source::community::Community,
  traits::DeleteableOrRemoveable,
  ListingType,
  SearchType,
  SortType,
};
use lemmy_db_views::{comment_view::CommentQueryBuilder, post_view::PostQueryBuilder};
use lemmy_db_views_actor::{
  community_view::CommunityQueryBuilder,
  person_view::PersonQueryBuilder,
};
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for Search {
  type Response = SearchResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<SearchResponse, LemmyError> {
    let data: &Search = self;

    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;

    check_private_instance(&local_user_view, context.pool()).await?;

    let show_nsfw = local_user_view.as_ref().map(|t| t.local_user.show_nsfw);
    let show_bot_accounts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_bot_accounts);
    let show_read_posts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_read_posts);

    let person_id = local_user_view.map(|u| u.person.id);

    let mut posts = Vec::new();
    let mut comments = Vec::new();
    let mut communities = Vec::new();
    let mut users = Vec::new();

    // TODO no clean / non-nsfw searching rn

    let q = data.q.to_owned();
    let page = data.page;
    let limit = data.limit;
    let sort: Option<SortType> = from_opt_str_to_opt_enum(&data.sort);
    let listing_type: Option<ListingType> = from_opt_str_to_opt_enum(&data.listing_type);
    let search_type: SearchType = from_opt_str_to_opt_enum(&data.type_).unwrap_or(SearchType::All);
    let community_id = data.community_id;
    let community_actor_id = if let Some(name) = &data.community_name {
      resolve_actor_identifier::<ApubCommunity, Community>(name, context)
        .await
        .ok()
        .map(|c| c.actor_id)
    } else {
      None
    };
    let creator_id = data.creator_id;
    match search_type {
      SearchType::Posts => {
        posts = blocking(context.pool(), move |conn| {
          PostQueryBuilder::create(conn)
            .sort(sort)
            .show_nsfw(show_nsfw)
            .show_bot_accounts(show_bot_accounts)
            .show_read_posts(show_read_posts)
            .listing_type(listing_type)
            .community_id(community_id)
            .community_actor_id(community_actor_id)
            .creator_id(creator_id)
            .my_person_id(person_id)
            .search_term(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::Comments => {
        comments = blocking(context.pool(), move |conn| {
          CommentQueryBuilder::create(conn)
            .sort(sort)
            .listing_type(listing_type)
            .search_term(q)
            .show_bot_accounts(show_bot_accounts)
            .community_id(community_id)
            .community_actor_id(community_actor_id)
            .creator_id(creator_id)
            .my_person_id(person_id)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::Communities => {
        communities = blocking(context.pool(), move |conn| {
          CommunityQueryBuilder::create(conn)
            .sort(sort)
            .listing_type(listing_type)
            .search_term(q)
            .my_person_id(person_id)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::Users => {
        users = blocking(context.pool(), move |conn| {
          PersonQueryBuilder::create(conn)
            .sort(sort)
            .search_term(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::All => {
        // If the community or creator is included, dont search communities or users
        let community_or_creator_included =
          data.community_id.is_some() || data.community_name.is_some() || data.creator_id.is_some();
        let community_actor_id_2 = community_actor_id.to_owned();

        posts = blocking(context.pool(), move |conn| {
          PostQueryBuilder::create(conn)
            .sort(sort)
            .show_nsfw(show_nsfw)
            .show_bot_accounts(show_bot_accounts)
            .show_read_posts(show_read_posts)
            .listing_type(listing_type)
            .community_id(community_id)
            .community_actor_id(community_actor_id_2)
            .creator_id(creator_id)
            .my_person_id(person_id)
            .search_term(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;

        let q = data.q.to_owned();
        let community_actor_id = community_actor_id.to_owned();

        comments = blocking(context.pool(), move |conn| {
          CommentQueryBuilder::create(conn)
            .sort(sort)
            .listing_type(listing_type)
            .search_term(q)
            .show_bot_accounts(show_bot_accounts)
            .community_id(community_id)
            .community_actor_id(community_actor_id)
            .creator_id(creator_id)
            .my_person_id(person_id)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;

        let q = data.q.to_owned();

        communities = if community_or_creator_included {
          vec![]
        } else {
          blocking(context.pool(), move |conn| {
            CommunityQueryBuilder::create(conn)
              .sort(sort)
              .listing_type(listing_type)
              .search_term(q)
              .my_person_id(person_id)
              .page(page)
              .limit(limit)
              .list()
          })
          .await??
        };

        let q = data.q.to_owned();

        users = if community_or_creator_included {
          vec![]
        } else {
          blocking(context.pool(), move |conn| {
            PersonQueryBuilder::create(conn)
              .sort(sort)
              .search_term(q)
              .page(page)
              .limit(limit)
              .list()
          })
          .await??
        };
      }
      SearchType::Url => {
        posts = blocking(context.pool(), move |conn| {
          PostQueryBuilder::create(conn)
            .sort(sort)
            .show_nsfw(show_nsfw)
            .show_bot_accounts(show_bot_accounts)
            .show_read_posts(show_read_posts)
            .listing_type(listing_type)
            .my_person_id(person_id)
            .community_id(community_id)
            .community_actor_id(community_actor_id)
            .creator_id(creator_id)
            .url_search(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
    };

    // Blank out deleted or removed info for non logged in users
    if person_id.is_none() {
      for cv in communities
        .iter_mut()
        .filter(|cv| cv.community.deleted || cv.community.removed)
      {
        cv.community = cv.to_owned().community.blank_out_deleted_or_removed_info();
      }

      for pv in posts
        .iter_mut()
        .filter(|p| p.post.deleted || p.post.removed)
      {
        pv.post = pv.to_owned().post.blank_out_deleted_or_removed_info();
      }

      for cv in comments
        .iter_mut()
        .filter(|cv| cv.comment.deleted || cv.comment.removed)
      {
        cv.comment = cv.to_owned().comment.blank_out_deleted_or_removed_info();
      }
    }

    // Return the jwt
    Ok(SearchResponse {
      type_: search_type.to_string(),
      comments,
      posts,
      communities,
      users,
    })
  }
}
