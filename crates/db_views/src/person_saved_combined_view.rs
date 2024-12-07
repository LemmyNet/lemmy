// use crate::{
//   structs::{
//     CommentView,
//     LocalUserView,
//     PostView,
//     ProfileCombinedPaginationCursor,
//     PersonContentCombinedView,
//     PersonContentViewInternal,
//   },
//   InternalToCombinedView,
// };
// use diesel::{
//   result::Error,
//   BoolExpressionMethods,
//   ExpressionMethods,
//   JoinOnDsl,
//   NullableExpressionMethods,
//   QueryDsl,
//   SelectableHelper,
// };
// use diesel_async::RunQueryDsl;
// use i_love_jesus::PaginatedQueryBuilder;
// use lemmy_db_schema::{
//   aliases::creator_community_actions,
//   newtypes::{CommunityId, PersonId},
//   schema::{
//     comment,
//     comment_actions,
//     comment_aggregates,
//     community,
//     community_actions,
//     image_details,
//     local_user,
//     person,
//     person_actions,
//     post,
//     post_actions,
//     post_aggregates,
//     profile_combined,
//   },
//   source::{
//     combined::profile::{profile_combined_keys as key, ProfileCombined},
//     community::CommunityFollower,
//   },
//   utils::{actions, actions_alias, functions::coalesce, get_conn, DbPool, ReverseTimestampKey},
// };
// use lemmy_utils::error::LemmyResult;

// impl ProfileCombinedPaginationCursor {
//   // get cursor for page that starts immediately after the given post
//   pub fn after_post(view: &PersonContentCombinedView) -> ProfileCombinedPaginationCursor {
//     let (prefix, id) = match view {
//       PersonContentCombinedView::Comment(v) => ('C', v.comment.id.0),
//       PersonContentCombinedView::Post(v) => ('P', v.post.id.0),
//     };
//     // hex encoding to prevent ossification
//     ProfileCombinedPaginationCursor(format!("{prefix}{id:x}"))
//   }

//   pub async fn read(&self, pool: &mut DbPool<'_>) -> Result<PaginationCursorData, Error> {
//     let err_msg = || Error::QueryBuilderError("Could not parse pagination token".into());
//     let mut query = profile_combined::table
//       .select(ProfileCombined::as_select())
//       .into_boxed();
//     let (prefix, id_str) = self.0.split_at_checked(1).ok_or_else(err_msg)?;
//     let id = i32::from_str_radix(id_str, 16).map_err(|_err| err_msg())?;
//     query = match prefix {
//       "C" => query.filter(profile_combined::comment_id.eq(id)),
//       "P" => query.filter(profile_combined::post_id.eq(id)),
//       _ => return Err(err_msg()),
//     };
//     let token = query.first(&mut get_conn(pool).await?).await?;

//     Ok(PaginationCursorData(token))
//   }
// }

// #[derive(Clone)]
// pub struct PaginationCursorData(ProfileCombined);

// #[derive(Default)]
// pub struct ProfileCombinedQuery {
//   pub creator_id: PersonId,
//   pub page_after: Option<PaginationCursorData>,
//   pub page_back: Option<bool>,
// }

// impl ProfileCombinedQuery {
//   pub async fn list(
//     self,
//     pool: &mut DbPool<'_>,
//     user: &Option<LocalUserView>,
//   ) -> LemmyResult<Vec<PersonContentCombinedView>> {
//     let my_person_id = user
//       .as_ref()
//       .map(|u| u.local_user.person_id)
//       .unwrap_or(PersonId(-1));
//     let item_creator = person::id;

//     let conn = &mut get_conn(pool).await?;

//     // Notes: since the post_id and comment_id are optional columns,
//     // many joins must use an OR condition.
//     // For example, the creator must be the person table joined to either:
//     // - post.creator_id
//     // - comment.creator_id
//     let mut query = profile_combined::table
//       // The comment
//       .left_join(comment::table.on(profile_combined::comment_id.eq(comment::id.nullable())))
//       // The post
//       .inner_join(
//         post::table.on(
//           profile_combined::post_id
//             .eq(post::id.nullable())
//             .or(comment::post_id.nullable().eq(profile_combined::post_id)),
//         ),
//       )
//       // The item creator
//       .inner_join(
//         person::table.on(
//           comment::creator_id
//             .eq(person::id)
//             .or(post::creator_id.eq(person::id)),
//         ),
//       )
//       // The community
//       .inner_join(community::table.on(post::community_id.eq(community::id)))
//       .left_join(actions_alias(
//         creator_community_actions,
//         item_creator,
//         post::community_id,
//       ))
//       .left_join(
//         local_user::table.on(
//           item_creator
//             .eq(local_user::person_id)
//             .and(local_user::admin.eq(true)),
//         ),
//       )
//       .left_join(actions(
//         community_actions::table,
//         Some(my_person_id),
//         post::community_id,
//       ))
//       .left_join(actions(post_actions::table, Some(my_person_id), post::id))
//       .left_join(actions(
//         person_actions::table,
//         Some(my_person_id),
//         item_creator,
//       ))
//       .inner_join(post_aggregates::table.on(post::id.eq(post_aggregates::post_id)))
//       .left_join(
//         comment_aggregates::table
//           .on(profile_combined::comment_id.eq(comment_aggregates::comment_id.nullable())),
//       )
//       .left_join(actions(
//         comment_actions::table,
//         Some(my_person_id),
//         comment::id,
//       ))
//       .left_join(image_details::table.on(post::thumbnail_url.eq(image_details::link.nullable())))
//       // The creator id filter
//       .filter(item_creator.eq(self.creator_id))
//       .select((
//         // Post-specific
//         post_aggregates::all_columns,
//         coalesce(
//           post_aggregates::comments.nullable() - post_actions::read_comments_amount.nullable(),
//           post_aggregates::comments,
//         ),
//         post_actions::saved.nullable().is_not_null(),
//         post_actions::read.nullable().is_not_null(),
//         post_actions::hidden.nullable().is_not_null(),
//         post_actions::like_score.nullable(),
//         image_details::all_columns.nullable(),
//         // Comment-specific
//         comment::all_columns.nullable(),
//         comment_aggregates::all_columns.nullable(),
//         comment_actions::saved.nullable().is_not_null(),
//         comment_actions::like_score.nullable(),
//         // Shared
//         post::all_columns,
//         community::all_columns,
//         person::all_columns,
//         CommunityFollower::select_subscribed_type(),
//         local_user::admin.nullable().is_not_null(),
//         creator_community_actions
//           .field(community_actions::became_moderator)
//           .nullable()
//           .is_not_null(),
//         creator_community_actions
//           .field(community_actions::received_ban)
//           .nullable()
//           .is_not_null(),
//         person_actions::blocked.nullable().is_not_null(),
//         community_actions::received_ban.nullable().is_not_null(),
//       ))
//       .into_boxed();

//     let mut query = PaginatedQueryBuilder::new(query);

//     let page_after = self.page_after.map(|c| c.0);

//     if self.page_back.unwrap_or_default() {
//       query = query.before(page_after).limit_and_offset_from_end();
//     } else {
//       query = query.after(page_after);
//     }

//     // Sorting by published
//     query = query
//       .then_desc(ReverseTimestampKey(key::published))
//       // Tie breaker
//       .then_desc(key::id);

//     let res = query.load::<PersonContentViewInternal>(conn).await?;

//     // Map the query results to the enum
//     let out = res.into_iter().filter_map(|u| u.map_to_enum()).collect();

//     Ok(out)
//   }
// }

// impl InternalToCombinedView for PersonContentViewInternal {
//   type CombinedView = PersonContentCombinedView;

//   fn map_to_enum(&self) -> Option<Self::CombinedView> {
//     // Use for a short alias
//     let v = self.clone();

//     if let (Some(comment), Some(counts)) = (v.comment, v.comment_counts) {
//       Some(PersonContentCombinedView::Comment(CommentView {
//         comment,
//         counts,
//         post: v.post,
//         community: v.community,
//         creator: v.item_creator,
//         creator_banned_from_community: v.item_creator_banned_from_community,
//         creator_is_moderator: v.item_creator_is_moderator,
//         creator_is_admin: v.item_creator_is_admin,
//         creator_blocked: v.item_creator_blocked,
//         subscribed: v.subscribed,
//         saved: v.comment_saved,
//         my_vote: v.my_comment_vote,
//         banned_from_community: v.banned_from_community,
//       }))
//     } else {
//       Some(PersonContentCombinedView::Post(PostView {
//         post: v.post,
//         community: v.community,
//         unread_comments: v.post_unread_comments,
//         counts: v.post_counts,
//         creator: v.item_creator,
//         creator_banned_from_community: v.item_creator_banned_from_community,
//         creator_is_moderator: v.item_creator_is_moderator,
//         creator_is_admin: v.item_creator_is_admin,
//         creator_blocked: v.item_creator_blocked,
//         subscribed: v.subscribed,
//         saved: v.post_saved,
//         read: v.post_read,
//         hidden: v.post_hidden,
//         my_vote: v.my_post_vote,
//         image_details: v.image_details,
//         banned_from_community: v.banned_from_community,
//       }))
//     }
//   }
// }

// #[cfg(test)]
// #[expect(clippy::indexing_slicing)]
// mod tests {

//   use crate::{
//     profile_combined_view::ProfileCombinedQuery,
//     report_combined_view::ReportCombinedQuery,
//     structs::{
//       CommentReportView,
//       LocalUserView,
//       PostReportView,
//       PersonContentCombinedView,
//       ReportCombinedView,
//       ReportCombinedViewInternal,
//     },
//   };
//   use lemmy_db_schema::{
//     aggregates::structs::{CommentAggregates, PostAggregates},
//     assert_length,
//     source::{
//       comment::{Comment, CommentInsertForm, CommentSaved, CommentSavedForm},
//       comment_report::{CommentReport, CommentReportForm},
//       community::{Community, CommunityInsertForm, CommunityModerator, CommunityModeratorForm},
//       instance::Instance,
//       local_user::{LocalUser, LocalUserInsertForm},
//       local_user_vote_display_mode::LocalUserVoteDisplayMode,
//       person::{Person, PersonInsertForm},
//       post::{Post, PostInsertForm},
//       post_report::{PostReport, PostReportForm},
//       private_message::{PrivateMessage, PrivateMessageInsertForm},
//       private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
//     },
//     traits::{Crud, Joinable, Reportable, Saveable},
//     utils::{build_db_pool_for_tests, DbPool},
//   };
//   use lemmy_utils::error::LemmyResult;
//   use pretty_assertions::assert_eq;
//   use serial_test::serial;

//   struct Data {
//     instance: Instance,
//     timmy: Person,
//     sara: Person,
//     timmy_view: LocalUserView,
//     community: Community,
//     timmy_post: Post,
//     timmy_post_2: Post,
//     sara_post: Post,
//     timmy_comment: Comment,
//     sara_comment: Comment,
//     sara_comment_2: Comment,
//   }

//   async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
//     let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

//     let timmy_form = PersonInsertForm::test_form(inserted_instance.id, "timmy_pcv");
//     let inserted_timmy = Person::create(pool, &timmy_form).await?;
//     let timmy_local_user_form = LocalUserInsertForm::test_form(inserted_timmy.id);
//     let timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;
//     let timmy_view = LocalUserView {
//       local_user: timmy_local_user,
//       local_user_vote_display_mode: LocalUserVoteDisplayMode::default(),
//       person: inserted_timmy.clone(),
//       counts: Default::default(),
//     };

//     let sara_form = PersonInsertForm::test_form(inserted_instance.id, "sara_pcv");
//     let inserted_sara = Person::create(pool, &sara_form).await?;

//     let community_form = CommunityInsertForm::new(
//       inserted_instance.id,
//       "test community pcv".to_string(),
//       "nada".to_owned(),
//       "pubkey".to_string(),
//     );
//     let inserted_community = Community::create(pool, &community_form).await?;

//     let timmy_post_form = PostInsertForm::new(
//       "timmy post prv".into(),
//       inserted_timmy.id,
//       inserted_community.id,
//     );
//     let timmy_post = Post::create(pool, &timmy_post_form).await?;

//     let timmy_post_form_2 = PostInsertForm::new(
//       "timmy post prv 2".into(),
//       inserted_timmy.id,
//       inserted_community.id,
//     );
//     let timmy_post_2 = Post::create(pool, &timmy_post_form_2).await?;

//     let sara_post_form = PostInsertForm::new(
//       "sara post prv".into(),
//       inserted_sara.id,
//       inserted_community.id,
//     );
//     let sara_post = Post::create(pool, &sara_post_form).await?;

//     let timmy_comment_form =
//       CommentInsertForm::new(inserted_timmy.id, timmy_post.id, "timmy comment prv".into());
//     let timmy_comment = Comment::create(pool, &timmy_comment_form, None).await?;

//     let sara_comment_form =
//       CommentInsertForm::new(inserted_sara.id, timmy_post.id, "sara comment prv".into());
//     let sara_comment = Comment::create(pool, &sara_comment_form, None).await?;

//     let sara_comment_form_2 = CommentInsertForm::new(
//       inserted_sara.id,
//       timmy_post_2.id,
//       "sara comment prv 2".into(),
//     );
//     let sara_comment_2 = Comment::create(pool, &sara_comment_form_2, None).await?;

//     Ok(Data {
//       instance: inserted_instance,
//       timmy: inserted_timmy,
//       sara: inserted_sara,
//       timmy_view,
//       community: inserted_community,
//       timmy_post,
//       timmy_post_2,
//       sara_post,
//       timmy_comment,
//       sara_comment,
//       sara_comment_2,
//     })
//   }

//   async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
//     Instance::delete(pool, data.instance.id).await?;

//     Ok(())
//   }

//   #[tokio::test]
//   #[serial]
//   async fn test_combined() -> LemmyResult<()> {
//     let pool = &build_db_pool_for_tests();
//     let pool = &mut pool.into();
//     let data = init_data(pool).await?;

//     // Do a batch read of timmy
//     let timmy_content = ProfileCombinedQuery::default().list(pool, &None).await?;
//     assert_eq!(3, timmy_content.len());

//     // Make sure the report types are correct
//     if let PersonContentCombinedView::Comment(v) = &timmy_content[0] {
//       assert_eq!(data.timmy_comment.id, v.comment.id);
//       assert_eq!(data.timmy.id, v.creator.id);
//     } else {
//       panic!("wrong type");
//     }
//     if let PersonContentCombinedView::Post(v) = &timmy_content[1] {
//       assert_eq!(data.timmy_post_2.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }
//     if let PersonContentCombinedView::Post(v) = &timmy_content[2] {
//       assert_eq!(data.timmy_post.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }

//     // Do a batch read of sara
//     let sara_content = ProfileCombinedQuery::default().list(pool, &None).await?;
//     assert_eq!(3, sara_content.len());

//     // Make sure the report types are correct
//     if let PersonContentCombinedView::Comment(v) = &sara_content[0] {
//       assert_eq!(data.sara_comment_2.id, v.comment.id);
//       assert_eq!(data.sara.id, v.creator.id);
//       // This one was to timmy_post_2
//       assert_eq!(data.timmy_post_2.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }
//     if let PersonContentCombinedView::Comment(v) = &sara_content[1] {
//       assert_eq!(data.sara_comment.id, v.comment.id);
//       assert_eq!(data.sara.id, v.creator.id);
//       assert_eq!(data.timmy_post.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }
//     if let PersonContentCombinedView::Post(v) = &sara_content[2] {
//       assert_eq!(data.timmy_post.id, v.post.id);
//       assert_eq!(data.timmy.id, v.post.creator_id);
//     } else {
//       panic!("wrong type");
//     }

//     // Timmy saves sara's comment, and his 2nd post
//     let save_comment_0_form = CommentSavedForm {
//       person_id: data.timmy.id,
//       comment_id: data.sara_comment.id,
//     };
//     CommentSaved::save(pool, &save_comment_0_form).await?;

//     // Timmy saves sara's comment, and his 2nd post
//     let save_comment_0_form = CommentSavedForm {
//       person_id: data.timmy.id,
//       comment_id: data.sara_comment.id,
//     };
//     CommentSaved::save(pool, &save_comment_0_form).await?;

//     // Do a saved_only query
//     let timmy_content_saved_only = ProfileCombinedQuery {}.list(pool, &None).await?;

//     cleanup(data, pool).await?;

//     Ok(())
//   }
// }
// #[tokio::test]
// #[serial]
// async fn test_saved_order() -> LemmyResult<()> {
//   let pool = &build_db_pool_for_tests();
//   let pool = &mut pool.into();
//   let data = init_data(pool).await?;

//   // Save two comments
//   let save_comment_0_form = CommentSavedForm {
//     person_id: data.timmy_local_user_view.person.id,
//     comment_id: data.inserted_comment_0.id,
//   };
//   CommentSaved::save(pool, &save_comment_0_form).await?;

//   let save_comment_2_form = CommentSavedForm {
//     person_id: data.timmy_local_user_view.person.id,
//     comment_id: data.inserted_comment_2.id,
//   };
//   CommentSaved::save(pool, &save_comment_2_form).await?;

//   // Fetch the saved comments
//   let comments = CommentQuery {
//     local_user: Some(&data.timmy_local_user_view.local_user),
//     saved_only: Some(true),
//     ..Default::default()
//   }
//   .list(&data.site, pool)
//   .await?;

//   // There should only be two comments
//   assert_eq!(2, comments.len());

//   // The first comment, should be the last one saved (descending order)
//   assert_eq!(comments[0].comment.id, data.inserted_comment_2.id);

//   // The second comment, should be the first one saved
//   assert_eq!(comments[1].comment.id, data.inserted_comment_0.id);

//   cleanup(data, pool).await
// }
// #[tokio::test]
// #[serial]
// async fn post_listing_saved_only() -> LemmyResult<()> {
//   let pool = &build_db_pool()?;
//   let pool = &mut pool.into();
//   let data = init_data(pool).await?;

//   // Save only the bot post
//   // The saved_only should only show the bot post
//   let post_save_form =
//     PostSavedForm::new(data.inserted_bot_post.id, data.local_user_view.person.id);
//   PostSaved::save(pool, &post_save_form).await?;

//   // Read the saved only
//   let read_saved_post_listing = PostQuery {
//     community_id: Some(data.inserted_community.id),
//     saved_only: Some(true),
//     ..data.default_post_query()
//   }
//   .list(&data.site, pool)
//   .await?;

//   // This should only include the bot post, not the one you created
//   assert_eq!(vec![POST_BY_BOT], names(&read_saved_post_listing));

//   cleanup(data, pool).await
// }
