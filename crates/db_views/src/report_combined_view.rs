use crate::structs::{
  LocalUserView,
  PostOrCommentReportViewTemp,
  PostReportView,
  ReportCombinedView,
};
use diesel::{
  pg::Pg,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases::{self, creator_community_actions},
  newtypes::{CommunityId, PersonId, PostReportId},
  schema::{
    comment_report,
    community,
    community_actions,
    local_user,
    person,
    person_actions,
    post,
    post_actions,
    post_aggregates,
    post_report,
    report_combined,
  },
  source::community::CommunityFollower,
  utils::{
    actions,
    actions_alias,
    functions::coalesce,
    get_conn,
    limit_and_offset,
    DbConn,
    DbPool,
    ListFn,
    Queries,
    ReadFn,
  },
};
use lemmy_utils::error::LemmyResult;

impl ReportCombinedView {
  /// returns the current unresolved report count for the communities you mod
  pub async fn get_report_count(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    admin: bool,
    community_id: Option<CommunityId>,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;
    let mut query = post_report::table
      .inner_join(post::table)
      .filter(post_report::resolved.eq(false))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id))
    }

    // If its not an admin, get only the ones you mod
    if !admin {
      query
        .inner_join(
          community_actions::table.on(
            community_actions::community_id
              .eq(post::community_id)
              .and(community_actions::person_id.eq(my_person_id))
              .and(community_actions::became_moderator.is_not_null()),
          ),
        )
        .select(count(post_report::id))
        .first::<i64>(conn)
        .await
    } else {
      query
        .select(count(post_report::id))
        .first::<i64>(conn)
        .await
    }
  }
}

#[derive(Default)]
pub struct ReportCombinedQuery {
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unresolved_only: bool,
}

impl ReportCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> LemmyResult<Vec<PostOrCommentReportViewTemp>> {
    let options = self;
    let conn = &mut get_conn(pool).await?;
    let mut query = report_combined::table
      .left_join(post_report::table)
      .left_join(comment_report::table)
      // .inner_join(post::table)
      // .inner_join(community::table.on(post::community_id.eq(community::id)))
      .left_join(
        person::table.on(
          post_report::creator_id
            .eq(person::id)
            .or(comment_report::creator_id.eq(person::id)),
        ),
      )
      // .inner_join(aliases::person1.on(post::creator_id.eq(aliases::person1.field(person::id))))
      // .left_join(actions_alias(
      //   creator_community_actions,
      //   post::creator_id,
      //   post::community_id,
      // ))
      // .left_join(actions(
      //   community_actions::table,
      //   Some(my_person_id),
      //   post::community_id,
      // ))
      // .left_join(
      //   local_user::table.on(
      //     post::creator_id
      //       .eq(local_user::person_id)
      //       .and(local_user::admin.eq(true)),
      //   ),
      // )
      // .left_join(actions(post_actions::table, Some(my_person_id), post::id))
      // .left_join(actions(
      //   person_actions::table,
      //   Some(my_person_id),
      //   post::creator_id,
      // ))
      // .inner_join(post_aggregates::table.on(post_report::post_id.eq(post_aggregates::post_id)))
      // .left_join(
      //   aliases::person2
      //     .on(post_report::resolver_id.eq(aliases::person2.field(person::id).nullable())),
      // )
      .select((
        post_report::all_columns.nullable(),
        comment_report::all_columns.nullable(),
        // post::all_columns,
        // community::all_columns,
        person::all_columns.nullable(),
        // aliases::person1.fields(person::all_columns),
        // creator_community_actions
        //   .field(community_actions::received_ban)
        //   .nullable()
        //   .is_not_null(),
        // creator_community_actions
        //   .field(community_actions::became_moderator)
        //   .nullable()
        //   .is_not_null(),
        // local_user::admin.nullable().is_not_null(),
        // CommunityFollower::select_subscribed_type(),
        // post_actions::saved.nullable().is_not_null(),
        // post_actions::read.nullable().is_not_null(),
        // post_actions::hidden.nullable().is_not_null(),
        // person_actions::blocked.nullable().is_not_null(),
        // post_actions::like_score.nullable(),
        // coalesce(
        //   post_aggregates::comments.nullable() - post_actions::read_comments_amount.nullable(),
        //   post_aggregates::comments,
        // ),
        // post_aggregates::all_columns,
        // aliases::person2.fields(person::all_columns.nullable()),
      ))
      .into_boxed();

    // if let Some(community_id) = options.community_id {
    //   query = query.filter(post::community_id.eq(community_id));
    // }

    // if let Some(post_id) = options.post_id {
    //   query = query.filter(post::id.eq(post_id));
    // }

    // If viewing all reports, order by newest, but if viewing unresolved only, show the oldest
    // first (FIFO)
    // if options.unresolved_only {
    //   query = query
    //     .filter(post_report::resolved.eq(false))
    //     .order_by(post_report::published.asc());
    // } else {
    //   query = query.order_by(post_report::published.desc());
    // }

    // If its not an admin, get only the ones you mod
    // if !user.local_user.admin {
    //   query = query.filter(community_actions::became_moderator.is_not_null());
    // }

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query = query.limit(limit).offset(offset);

    let res = query.load::<ReportCombinedView>(conn).await?;
    let out = res
      .iter()
      .filter_map(map_to_post_or_comment_view_tmp)
      .collect();

    Ok(out)
  }
}

fn map_to_post_or_comment_view_tmp(
  view: &ReportCombinedView,
) -> Option<PostOrCommentReportViewTemp> {
  // If it has post_report, you know the other fields are defined
  if let (Some(post_report), Some(post_creator)) = (view.post_report.clone(), view.creator.clone())
  {
    Some(PostOrCommentReportViewTemp::Post {
      post_report,
      post_creator,
    })
  } else if let (Some(comment_report), Some(comment_creator)) =
    (view.comment_report.clone(), view.creator.clone())
  {
    Some(PostOrCommentReportViewTemp::Comment {
      comment_report,
      comment_creator,
    })
  } else {
    None
  }
}

// TODO add tests
