jest.setTimeout(180000);

import { PostResponse } from "lemmy-js-client/dist/types/PostResponse";
import {
  alpha,
  beta,
  gamma,
  setupLogins,
  createPost,
  getPost,
  resolveComment,
  likeComment,
  followBeta,
  resolveBetaCommunity,
  createComment,
  editComment,
  deleteComment,
  removeComment,
  resolvePost,
  unfollowRemotes,
  createCommunity,
  registerUser,
  reportComment,
  randomString,
  unfollows,
  getComment,
  getComments,
  getCommentParentId,
  resolveCommunity,
  waitUntil,
  waitForPost,
  alphaUrl,
  betaUrl,
  followCommunity,
  blockCommunity,
  saveUserSettings,
  listReports,
  listPersonContent,
  listNotifications,
  lockComment,
  statusNotFound,
  statusBadRequest,
  jestLemmyError,
  getUnreadCounts,
  expectSuccess,
  expectFailure,
  waitUntilSuccess,
} from "./shared";
import {
  CommentReportView,
  CommentResponse,
  CommentView,
  CommunityView,
  DistinguishComment,
  LemmyError,
  ReportCombinedView,
  SaveUserSettings,
} from "lemmy-js-client";

let betaCommunity: CommunityView | undefined;
let postOnAlphaRes: PostResponse;

beforeAll(async () => {
  await setupLogins();
  await Promise.allSettled([followBeta(alpha), followBeta(gamma)]);
  betaCommunity = await resolveBetaCommunity(alpha);
  if (betaCommunity) {
    postOnAlphaRes = await createPost(alpha, betaCommunity.community.id).then(
      expectSuccess,
    );
  }
});

afterAll(unfollows);

function assertCommentFederation(
  commentOne?: CommentView,
  commentTwo?: CommentView,
) {
  expect(commentOne?.comment.ap_id).toBe(commentTwo?.comment.ap_id);
  expect(commentOne?.comment.content).toBe(commentTwo?.comment.content);
  expect(commentOne?.creator.name).toBe(commentTwo?.creator.name);
  expect(commentOne?.community.ap_id).toBe(commentTwo?.community.ap_id);
  expect(commentOne?.comment.published_at).toBe(
    commentTwo?.comment.published_at,
  );
  expect(commentOne?.comment.updated_at).toBe(commentOne?.comment.updated_at);
  expect(commentOne?.comment.deleted).toBe(commentOne?.comment.deleted);
  expect(commentOne?.comment.removed).toBe(commentOne?.comment.removed);
}

test("Create a comment", async () => {
  const commentRes = await createComment(
    alpha,
    postOnAlphaRes.post_view.post.id,
  ).then(expectSuccess);
  expect(commentRes.comment_view.comment.content).toBeDefined();
  expect(commentRes.comment_view.community.local).toBe(false);
  expect(commentRes.comment_view.creator.local).toBe(true);
  expect(commentRes.comment_view.comment.score).toBe(1);

  // Make sure that comment is liked on beta
  const betaComment = await waitUntil(
    () => resolveComment(beta, commentRes.comment_view.comment),
    c => c?.comment.score === 1,
  );
  expect(betaComment).toBeDefined();
  expect(betaComment?.community.local).toBe(true);
  expect(betaComment?.creator.local).toBe(false);
  expect(betaComment?.comment.score).toBe(1);
  assertCommentFederation(betaComment, commentRes.comment_view);
});

test("Create a comment in a non-existent post", async () => {
  await jestLemmyError(
    () => createComment(alpha, -1).then(expectFailure),
    new LemmyError("not_found", statusNotFound),
  );
});

test("Update a comment", async () => {
  const commentRes = await createComment(
    alpha,
    postOnAlphaRes.post_view.post.id,
  ).then(expectSuccess);
  // Federate the comment first
  const betaComment = await resolveComment(
    beta,
    commentRes.comment_view.comment,
  );
  assertCommentFederation(betaComment, commentRes.comment_view);

  const updateCommentRes = await editComment(
    alpha,
    commentRes.comment_view.comment.id,
  ).then(expectSuccess);
  expect(updateCommentRes.comment_view.comment.content).toBe(
    "A jest test federated comment update",
  );
  expect(updateCommentRes.comment_view.community.local).toBe(false);
  expect(updateCommentRes.comment_view.creator.local).toBe(true);

  // Make sure that post is updated on beta
  const betaCommentUpdated = await waitUntil(
    () => resolveComment(beta, commentRes.comment_view.comment),
    c => c?.comment.content === "A jest test federated comment update",
  );
  assertCommentFederation(betaCommentUpdated, updateCommentRes.comment_view);
});

test("Delete a comment", async () => {
  const post = await createPost(alpha, betaCommunity!.community.id).then(
    expectSuccess,
  );
  // creating a comment on alpha (remote from home of community)
  const commentRes = await createComment(alpha, post.post_view.post.id).then(
    expectSuccess,
  );

  // Find the comment on beta (home of community)
  const betaComment = await resolveComment(
    beta,
    commentRes.comment_view.comment,
  );
  if (!betaComment) {
    throw new Error("Missing beta comment before delete");
  }

  // Find the comment on remote instance gamma
  const gammaComment = (
    await waitUntil(
      () => resolveComment(gamma, commentRes.comment_view.comment),
      r => !!r,
    )
  )?.comment;
  if (!gammaComment) {
    throw new Error(
      "Missing gamma comment (remote-home-remote replication) before delete",
    );
  }

  const deleteCommentRes = await deleteComment(
    alpha,
    true,
    commentRes.comment_view.comment.id,
  ).then(expectSuccess);
  expect(deleteCommentRes.comment_view.comment.deleted).toBe(true);

  // Make sure that comment is deleted on beta
  await waitUntil(
    () => resolveComment(beta, commentRes.comment_view.comment),
    c => c?.comment.deleted === true,
  );

  // Make sure that comment is deleted on gamma after delete
  await waitUntil(
    () => resolveComment(gamma, commentRes.comment_view.comment),
    c => c?.comment.deleted === true,
  );

  // Test undeleting the comment
  const undeleteCommentRes = await deleteComment(
    alpha,
    false,
    commentRes.comment_view.comment.id,
  ).then(expectSuccess);
  expect(undeleteCommentRes.comment_view.comment.deleted).toBe(false);

  // Make sure that comment is undeleted on beta
  const betaComment2 = await waitUntil(
    () => resolveComment(beta, commentRes.comment_view.comment),
    c => c?.comment.deleted === false,
  );
  assertCommentFederation(betaComment2, undeleteCommentRes.comment_view);
});

test.skip("Remove a comment from admin and community on the same instance", async () => {
  const commentRes = await createComment(
    alpha,
    postOnAlphaRes.post_view.post.id,
  ).then(expectSuccess);

  // Get the id for beta
  const betaCommentId = (
    await resolveComment(beta, commentRes.comment_view.comment)
  )?.comment.id;

  if (!betaCommentId) {
    throw new Error("beta comment id is missing");
  }

  // The beta admin removes it (the community lives on beta)
  const removeCommentRes = await removeComment(beta, true, betaCommentId).then(
    expectSuccess,
  );
  expect(removeCommentRes.comment_view.comment.removed).toBe(true);

  // Make sure that comment is removed on alpha (it gets pushed since an admin from beta removed it)
  const refetchedPostComments = await listPersonContent(
    alpha,
    commentRes.comment_view.comment.creator_id,
    "comments",
  ).then(expectSuccess);
  const firstRefetchedComment = refetchedPostComments.items[0] as CommentView;
  expect(firstRefetchedComment.comment.removed).toBe(true);

  // beta will unremove the comment
  const unremoveCommentRes = await removeComment(
    beta,
    false,
    betaCommentId,
  ).then(expectSuccess);
  expect(unremoveCommentRes.comment_view.comment.removed).toBe(false);

  // Make sure that comment is unremoved on alpha
  const refetchedPostComments2 = await getComments(
    alpha,
    postOnAlphaRes.post_view.post.id,
  ).then(expectSuccess);
  expect(refetchedPostComments2.items[0].comment.removed).toBe(false);
  assertCommentFederation(
    refetchedPostComments2.items[0],
    unremoveCommentRes.comment_view,
  );
});

test("Remove a comment from admin and community on different instance", async () => {
  const newAlphaApi = await registerUser(alpha, alphaUrl);

  // New alpha user creates a community, post, and comment.
  const newCommunity = await createCommunity(newAlphaApi).then(expectSuccess);
  const newPost = await createPost(
    newAlphaApi,
    newCommunity.community_view.community.id,
  ).then(expectSuccess);
  const commentRes = await createComment(
    newAlphaApi,
    newPost.post_view.post.id,
  ).then(expectSuccess);
  expect(commentRes.comment_view.comment.content).toBeDefined();

  // Beta searches that to cache it, then removes it
  const betaComment = await waitUntil(
    () => resolveComment(beta, commentRes.comment_view.comment),
    c => c?.comment !== undefined,
  );

  if (!betaComment) {
    throw new Error("beta comment missing");
  }

  const removeCommentRes = await removeComment(
    beta,
    true,
    betaComment.comment.id,
  ).then(expectSuccess);
  expect(removeCommentRes.comment_view.comment.removed).toBe(true);

  // Comment text is also hidden from list
  const listComments = await getComments(
    beta,
    removeCommentRes.comment_view.post.id,
  ).then(expectSuccess);
  expect(listComments.items.length).toBe(1);
  expect(listComments.items[0].comment.removed).toBe(true);

  // Make sure its not removed on alpha
  const refetchedPostComments = await getComments(
    alpha,
    newPost.post_view.post.id,
  ).then(expectSuccess);
  expect(refetchedPostComments.items[0].comment.removed).toBe(false);
  assertCommentFederation(
    refetchedPostComments.items[0],
    commentRes.comment_view,
  );
});

test("Unlike a comment", async () => {
  const commentRes = await createComment(
    alpha,
    postOnAlphaRes.post_view.post.id,
  ).then(expectSuccess);

  // Lemmy automatically creates 1 like (vote) by author of comment.
  // Make sure that comment is liked (voted up) on gamma, downstream peer
  // This is testing replication from remote-home-remote (alpha-beta-gamma)

  const gammaComment1 = await waitUntil(
    () => resolveComment(gamma, commentRes.comment_view.comment),
    c => c?.comment.score === 1,
  );
  expect(gammaComment1).toBeDefined();
  expect(gammaComment1?.community.local).toBe(false);
  expect(gammaComment1?.creator.local).toBe(false);
  expect(gammaComment1?.comment.score).toBe(1);

  const unlike = await likeComment(
    alpha,
    undefined,
    commentRes.comment_view.comment,
  ).then(expectSuccess);
  expect(unlike.comment_view.comment.score).toBe(0);

  // Make sure that comment is unliked on beta
  const betaComment = await waitUntil(
    () => resolveComment(beta, commentRes.comment_view.comment),
    c => c?.comment.score === 0,
  );
  expect(betaComment).toBeDefined();
  expect(betaComment?.community.local).toBe(true);
  expect(betaComment?.creator.local).toBe(false);
  expect(betaComment?.comment.score).toBe(0);

  // Make sure that comment is unliked on gamma, downstream peer
  // This is testing replication from remote-home-remote (alpha-beta-gamma)
  const gammaComment = await waitUntil(
    () => resolveComment(gamma, commentRes.comment_view.comment),
    c => c?.comment.score === 0,
  );
  expect(gammaComment).toBeDefined();
  expect(gammaComment?.community.local).toBe(false);
  expect(gammaComment?.creator.local).toBe(false);
  expect(gammaComment?.comment.score).toBe(0);
});

test("Federated comment like", async () => {
  const commentRes = await createComment(
    alpha,
    postOnAlphaRes.post_view.post.id,
  ).then(expectSuccess);
  await waitUntil(
    () => resolveComment(beta, commentRes.comment_view.comment),
    c => c?.comment.score === 1,
  );
  // Find the comment on beta
  const betaComment = await resolveComment(
    beta,
    commentRes.comment_view.comment,
  );

  if (!betaComment) {
    throw new Error("Missing beta comment");
  }

  const like = await likeComment(beta, true, betaComment.comment).then(
    expectSuccess,
  );
  expect(like.comment_view.comment.score).toBe(2);

  // Get the post from alpha, check the likes
  const postComments = await waitUntilSuccess(
    () => getComments(alpha, postOnAlphaRes.post_view.post.id),
    c => c.items[0].comment.score === 2,
  );
  expect(postComments.items[0].comment.score).toBe(2);
});

test("Reply to a comment from another instance, get notification", async () => {
  await alpha.markAllNotificationsAsRead();

  const betaCommunity = await waitUntil(
    () => resolveBetaCommunity(alpha),
    c => !!c?.community.instance_id,
  );
  if (!betaCommunity) {
    throw new Error("Missing beta community");
  }

  const postOnAlphaRes = await createPost(
    alpha,
    betaCommunity.community.id,
  ).then(expectSuccess);

  // Create a root-level trunk-branch comment on alpha
  const commentRes = await createComment(
    alpha,
    postOnAlphaRes.post_view.post.id,
  ).then(expectSuccess);
  // find that comment id on beta
  const betaComment = await waitUntil(
    () => resolveComment(beta, commentRes.comment_view.comment),
    c => c?.comment.score === 1,
  );

  if (!betaComment) {
    throw new Error("Missing beta comment");
  }

  // Reply from beta, extending the branch
  const replyRes = await createComment(
    beta,
    betaComment.post.id,
    betaComment.comment.id,
  ).then(expectSuccess);
  expect(replyRes.comment_view.comment.content).toBeDefined();
  expect(replyRes.comment_view.community.local).toBe(true);
  expect(replyRes.comment_view.creator.local).toBe(true);
  expect(getCommentParentId(replyRes.comment_view.comment)).toBe(
    betaComment.comment.id,
  );
  expect(replyRes.comment_view.comment.score).toBe(1);

  // Make sure that reply comment is seen on alpha
  const commentSearch = await waitUntil(
    () => resolveComment(alpha, replyRes.comment_view.comment),
    c => c?.comment.score === 1,
  );
  const alphaComment = commentSearch!;
  const postComments = await waitUntilSuccess(
    () => getComments(alpha, postOnAlphaRes.post_view.post.id),
    pc => pc.items.length >= 2,
  );
  // Note: this test fails when run twice and this count will differ
  expect(postComments.items.length).toBeGreaterThanOrEqual(2);
  expect(alphaComment.comment.content).toBeDefined();

  expect(getCommentParentId(alphaComment.comment)).toBe(
    postComments.items[1].comment.id,
  );
  expect(alphaComment.community.local).toBe(false);
  expect(alphaComment.creator.local).toBe(false);
  expect(alphaComment.comment.score).toBe(1);
  assertCommentFederation(alphaComment, replyRes.comment_view);

  // Did alpha get notified of the reply from beta?
  const alphaUnreadCountRes = await waitUntilSuccess(
    () => getUnreadCounts(alpha),
    e => e.notification_count >= 1,
  );
  expect(alphaUnreadCountRes.notification_count).toBeGreaterThanOrEqual(1);

  // check inbox of replies on alpha, fetching read/unread both
  const alphaRepliesRes = await waitUntilSuccess(
    () => listNotifications(alpha, "reply"),
    r => r.items.length > 0,
  );
  const alphaReply = alphaRepliesRes.items.find(
    r =>
      r.data.type_ == "comment" &&
      r.data.comment.id === alphaComment.comment.id,
  );
  expect(alphaReply).toBeDefined();
  if (!alphaReply) throw Error();
  const alphaReplyData = alphaReply.data as CommentView;
  expect(alphaReplyData.comment.content).toBeDefined();
  expect(alphaReplyData.community.local).toBe(false);
  expect(alphaReplyData.creator.local).toBe(false);
  expect(alphaReplyData.comment.score).toBe(1);
  // ToDo: interesting alphaRepliesRes.replies[0].comment_reply.id is 1, meaning? how did that come about?
  expect(alphaReplyData.comment.id).toBe(alphaComment.comment.id);
  // this is a new notification, getReplies fetch was for read/unread both, confirm it is unread.
  expect(alphaReply.notification.read).toBe(false);
});

test("Bot reply notifications are filtered when bots are hidden", async () => {
  const newAlphaBot = await registerUser(alpha, alphaUrl);
  let form: SaveUserSettings = {
    bot_account: true,
  };
  await saveUserSettings(newAlphaBot, form);

  const alphaCommunity = await resolveCommunity(
    alpha,
    "!main@lemmy-alpha:8541",
  );

  if (!alphaCommunity) {
    throw new Error("Missing alpha community");
  }

  await alpha.markAllNotificationsAsRead();
  form = {
    show_bot_accounts: false,
  };
  await saveUserSettings(alpha, form);
  const postOnAlphaRes = await createPost(
    alpha,
    alphaCommunity.community.id,
  ).then(expectSuccess);

  // Bot reply to alpha's post
  const commentRes = await createComment(
    newAlphaBot,
    postOnAlphaRes.post_view.post.id,
  ).then(expectSuccess);
  expect(commentRes).toBeDefined();

  let alphaUnreadCountRes = await getUnreadCounts(alpha).then(expectSuccess);
  expect(alphaUnreadCountRes.notification_count).toBe(0);

  // This both restores the original state that may be expected by other tests
  // implicitly and is used by the next steps to ensure replies are still
  // returned when a user later decides to show bot accounts again.
  form = {
    show_bot_accounts: true,
  };
  await saveUserSettings(alpha, form);

  alphaUnreadCountRes = await getUnreadCounts(alpha).then(expectSuccess);
  expect(alphaUnreadCountRes.notification_count).toBe(1);

  const alphaUnreadRepliesRes = await listNotifications(
    alpha,
    "reply",
    true,
  ).then(expectSuccess);
  expect(alphaUnreadRepliesRes.items.length).toBe(1);
  expect(alphaUnreadRepliesRes.items[0].notification.comment_id).toBe(
    commentRes.comment_view.comment.id,
  );
});

test("Mention beta from alpha comment", async () => {
  if (!betaCommunity) throw Error("no community");
  const postOnAlphaRes = await createPost(
    alpha,
    betaCommunity.community.id,
  ).then(expectSuccess);
  // Create a new branch, trunk-level comment branch, from alpha instance
  const commentRes = await createComment(
    alpha,
    postOnAlphaRes.post_view.post.id,
  ).then(expectSuccess);
  // Create a reply comment to previous comment, this has a mention in body
  const mentionContent = "A test mention of @lemmy_beta@lemmy-beta:8551";
  const mentionRes = await createComment(
    alpha,
    postOnAlphaRes.post_view.post.id,
    commentRes.comment_view.comment.id,
    mentionContent,
  ).then(expectSuccess);
  expect(mentionRes.comment_view.comment.content).toBeDefined();
  expect(mentionRes.comment_view.community.local).toBe(false);
  expect(mentionRes.comment_view.creator.local).toBe(true);
  expect(mentionRes.comment_view.comment.score).toBe(1);

  // get beta's localized copy of the alpha post
  const betaPost = await waitForPost(beta, postOnAlphaRes.post_view.post);
  if (!betaPost) {
    throw new Error("unable to locate post on beta");
  }
  expect(betaPost.post.ap_id).toBe(postOnAlphaRes.post_view.post.ap_id);
  expect(betaPost.post.name).toBe(postOnAlphaRes.post_view.post.name);

  // Make sure that both new comments are seen on beta and have parent/child relationship
  const betaPostComments = await waitUntilSuccess(
    () => getComments(beta, betaPost.post.id),
    c => c.items[1]?.comment.score === 1,
  );
  expect(betaPostComments.items.length).toEqual(2);
  // the trunk-branch root comment will be older than the mention reply comment, so index 1
  const betaRootComment = betaPostComments.items[1];
  // the trunk-branch root comment should not have a parent
  expect(getCommentParentId(betaRootComment.comment)).toBeUndefined();
  expect(betaRootComment.comment.content).toBeDefined();
  // the mention reply comment should have parent that points to the branch root level comment
  expect(getCommentParentId(betaPostComments.items[0].comment)).toBe(
    betaPostComments.items[1].comment.id,
  );
  expect(betaRootComment.community.local).toBe(true);
  expect(betaRootComment.creator.local).toBe(false);
  expect(betaRootComment.comment.score).toBe(1);
  assertCommentFederation(betaRootComment, commentRes.comment_view);

  const mentionsRes = await waitUntilSuccess(
    () => listNotifications(beta, "mention"),
    m => !!m.items[0],
  );

  const firstMention = mentionsRes.items[0];
  const firstMentionData = firstMention.data as CommentView;
  expect(firstMentionData.comment.content).toBeDefined();
  expect(firstMentionData.community.local).toBe(true);
  expect(firstMentionData.creator.local).toBe(false);
  expect(firstMentionData.comment.score).toBe(1);
  // the reply comment with mention should be the most fresh, newest, index 0
  expect(firstMentionData.comment.id).toBe(
    betaPostComments.items[0].comment.id,
  );
});

test("Comment Search", async () => {
  const commentRes = await createComment(
    alpha,
    postOnAlphaRes.post_view.post.id,
  ).then(expectSuccess);
  const betaComment = await resolveComment(
    beta,
    commentRes.comment_view.comment,
  );
  assertCommentFederation(betaComment, commentRes.comment_view);
});

test("A and G subscribe to B (center) A posts, G mentions B, it gets announced to A", async () => {
  // Create a local post
  const alphaCommunity = await resolveCommunity(
    alpha,
    "!main@lemmy-alpha:8541",
  );
  if (!alphaCommunity) {
    throw new Error("Missing alpha community");
  }

  // follow community from beta so that it accepts the mention
  const betaCommunity = await resolveCommunity(
    beta,
    alphaCommunity.community.ap_id,
  );
  await followCommunity(beta, true, betaCommunity!.community.id);

  const alphaPost = await createPost(alpha, alphaCommunity.community.id).then(
    expectSuccess,
  );
  expect(alphaPost.post_view.community.local).toBe(true);

  // Make sure gamma sees it
  const gammaPost = await resolvePost(gamma, alphaPost.post_view.post);

  if (!gammaPost) {
    throw new Error("Missing gamma post");
  }

  const commentContent =
    "A jest test federated comment announce, lets mention @lemmy_beta@lemmy-beta:8551";
  const commentRes = await createComment(
    gamma,
    gammaPost.post.id,
    undefined,
    commentContent,
  ).then(expectSuccess);
  expect(commentRes.comment_view.comment.content).toBe(commentContent);
  expect(commentRes.comment_view.community.local).toBe(false);
  expect(commentRes.comment_view.creator.local).toBe(true);
  expect(commentRes.comment_view.comment.score).toBe(1);

  // Make sure alpha sees it
  const alphaPostComments2 = await waitUntilSuccess(
    () => getComments(alpha, alphaPost.post_view.post.id),
    e => e.items[0]?.comment.score === 1,
  );
  expect(alphaPostComments2.items[0].comment.content).toBe(commentContent);
  expect(alphaPostComments2.items[0].community.local).toBe(true);
  expect(alphaPostComments2.items[0].creator.local).toBe(false);
  expect(alphaPostComments2.items[0].comment.score).toBe(1);
  assertCommentFederation(alphaPostComments2.items[0], commentRes.comment_view);

  // Make sure beta has mentions
  const relevantMention = await waitUntil(
    () =>
      listNotifications(beta, "mention")
        .then(expectSuccess)
        .then(m =>
          m.items.find(m => {
            const data = m.data as CommentView;
            return (
              m.notification.kind == "mention" &&
              data.comment.ap_id === commentRes.comment_view.comment.ap_id
            );
          }),
        ),
    e => !!e,
  );
  if (!relevantMention) throw Error("could not find mention");
  const relevantMentionData = relevantMention.data as CommentView;
  expect(relevantMentionData.comment.content).toBe(commentContent);
  expect(relevantMentionData.community.local).toBe(false);
  expect(relevantMentionData.creator.local).toBe(false);
  // TODO this is failing because fetchInReplyTos aren't getting score
  // expect(mentionsRes.mentions[0].score).toBe(1);
});

test("Check that activity from another instance is sent to third instance", async () => {
  // Alpha and gamma users follow beta community
  const alphaFollow = await followBeta(alpha);
  expect(alphaFollow.community_view.community.local).toBe(false);
  expect(alphaFollow.community_view.community.name).toBe("main");

  const gammaFollow = await followBeta(gamma);
  expect(gammaFollow.community_view.community.local).toBe(false);
  expect(gammaFollow.community_view.community.name).toBe("main");
  await waitUntil(
    () => resolveBetaCommunity(alpha),
    c => c?.community_actions?.follow_state === "accepted",
  );
  await waitUntil(
    () => resolveBetaCommunity(gamma),
    c => c?.community_actions?.follow_state === "accepted",
  );

  // Create a post on beta
  const betaPost = await createPost(beta, 2).then(expectSuccess);
  expect(betaPost.post_view.community.local).toBe(true);

  // Make sure gamma and alpha see it
  const gammaPost = await waitForPost(gamma, betaPost.post_view.post);
  if (!gammaPost) {
    throw new Error("Missing gamma post");
  }
  expect(gammaPost.post).toBeDefined();

  const alphaPost = await waitForPost(alpha, betaPost.post_view.post);
  if (!alphaPost) {
    throw new Error("Missing alpha post");
  }
  expect(alphaPost.post).toBeDefined();

  // The bug: gamma comments, and alpha should see it.
  const commentContent = "Comment from gamma";
  const commentRes = await createComment(
    gamma,
    gammaPost.post.id,
    undefined,
    commentContent,
  ).then(expectSuccess);
  expect(commentRes.comment_view.comment.content).toBe(commentContent);
  expect(commentRes.comment_view.community.local).toBe(false);
  expect(commentRes.comment_view.creator.local).toBe(true);
  expect(commentRes.comment_view.comment.score).toBe(1);

  // Make sure alpha sees it
  const alphaPostComments2 = await waitUntilSuccess(
    () => getComments(alpha, alphaPost.post.id),
    e => e.items[0]?.comment.score === 1,
  );
  expect(alphaPostComments2.items[0].comment.content).toBe(commentContent);
  expect(alphaPostComments2.items[0].community.local).toBe(false);
  expect(alphaPostComments2.items[0].creator.local).toBe(false);
  expect(alphaPostComments2.items[0].comment.score).toBe(1);
  assertCommentFederation(alphaPostComments2.items[0], commentRes.comment_view);

  await Promise.allSettled([unfollowRemotes(alpha), unfollowRemotes(gamma)]);
});

test("Fetch in_reply_tos: A is unsubbed from B, B makes a post, and some embedded comments, A subs to B, B updates the lowest level comment, A fetches both the post and all the inreplyto comments for that post.", async () => {
  // Unfollow all remote communities
  const my_user = await unfollowRemotes(alpha).then(expectSuccess);
  expect(my_user.follows.filter(c => c.community.local == false).length).toBe(
    0,
  );

  // B creates a post, and two comments, should be invisible to A
  const postOnBetaRes = await createPost(beta, 2).then(expectSuccess);
  expect(postOnBetaRes.post_view.post.name).toBeDefined();

  const parentCommentContent = "An invisible top level comment from beta";
  const parentCommentRes = await createComment(
    beta,
    postOnBetaRes.post_view.post.id,
    undefined,
    parentCommentContent,
  ).then(expectSuccess);
  expect(parentCommentRes.comment_view.comment.content).toBe(
    parentCommentContent,
  );

  // B creates a comment, then a child one of that.
  const childCommentContent = "An invisible child comment from beta";
  const childCommentRes = await createComment(
    beta,
    postOnBetaRes.post_view.post.id,
    parentCommentRes.comment_view.comment.id,
    childCommentContent,
  ).then(expectSuccess);
  expect(childCommentRes.comment_view.comment.content).toBe(
    childCommentContent,
  );

  // Follow beta again
  const follow = await followBeta(alpha);
  expect(follow.community_view.community.local).toBe(false);
  expect(follow.community_view.community.name).toBe("main");

  // An update to the child comment on beta, should push the post, parent, and child to alpha now
  const updatedCommentContent = "An update child comment from beta";
  const updateRes = await editComment(
    beta,
    childCommentRes.comment_view.comment.id,
    updatedCommentContent,
  ).then(expectSuccess);
  expect(updateRes.comment_view.comment.content).toBe(updatedCommentContent);

  // Get the post from alpha
  const alphaPostB = await waitForPost(alpha, postOnBetaRes.post_view.post);

  if (!alphaPostB) {
    throw new Error("Missing alpha post B");
  }

  const alphaPost = await getPost(alpha, alphaPostB.post.id).then(
    expectSuccess,
  );
  const alphaPostComments = await waitUntilSuccess(
    () => getComments(alpha, alphaPostB.post.id),
    c =>
      c.items[1]?.comment.content ===
        parentCommentRes.comment_view.comment.content &&
      c.items[0]?.comment.content === updateRes.comment_view.comment.content,
  );
  expect(alphaPost.post_view.post.name).toBeDefined();
  assertCommentFederation(
    alphaPostComments.items[1],
    parentCommentRes.comment_view,
  );
  assertCommentFederation(alphaPostComments.items[0], updateRes.comment_view);
  expect(alphaPost.post_view.community.local).toBe(false);
  expect(alphaPost.post_view.creator.local).toBe(false);

  await unfollowRemotes(alpha);
});

test("Report a comment", async () => {
  const betaCommunity = await resolveBetaCommunity(beta);
  if (!betaCommunity) {
    throw new Error("Missing beta community");
  }
  const postOnBetaRes = (
    await createPost(beta, betaCommunity.community.id).then(expectSuccess)
  ).post_view.post;
  expect(postOnBetaRes).toBeDefined();
  const commentRes = (
    await createComment(beta, postOnBetaRes.id).then(expectSuccess)
  ).comment_view.comment;
  expect(commentRes).toBeDefined();

  const alphaComment = await resolveComment(alpha, commentRes);
  if (!alphaComment) {
    throw new Error("Missing alpha comment");
  }

  const reason = randomString(10);
  const alphaReport = (
    await reportComment(alpha, alphaComment.comment.id, reason).then(
      expectSuccess,
    )
  ).comment_report_view.comment_report;

  const betaReport = (
    (await waitUntil(
      () =>
        listReports(beta)
          .then(expectSuccess)
          .then(p =>
            p.items.find(r => {
              return checkCommentReportReason(r, reason);
            }),
          ),
      e => !!e,
    )) as CommentReportView
  ).comment_report;
  expect(betaReport).toBeDefined();
  expect(betaReport.resolved).toBe(false);
  expect(betaReport.original_comment_text).toBe(
    alphaReport.original_comment_text,
  );
  expect(betaReport.reason).toBe(alphaReport.reason);
});

test("Dont send a comment reply to a blocked community", async () => {
  await beta.markAllNotificationsAsRead();
  const newCommunity = await createCommunity(beta).then(expectSuccess);
  const newCommunityId = newCommunity.community_view.community.id;

  // Create a post on beta
  const betaPost = await createPost(beta, newCommunityId).then(expectSuccess);

  const alphaPost = await resolvePost(alpha, betaPost.post_view.post);
  if (!alphaPost) {
    throw new Error("unable to locate post on alpha");
  }

  // Check beta's inbox count
  let unreadCount = await getUnreadCounts(beta).then(expectSuccess);
  expect(unreadCount.notification_count).toBe(0);

  // Beta blocks the new beta community
  let blockRes = await blockCommunity(beta, newCommunityId, true).then(
    expectSuccess,
  );
  expect(blockRes.community_view.community_actions?.blocked_at).toBeDefined();

  // Alpha creates a comment
  const commentRes = await createComment(alpha, alphaPost.post.id).then(
    expectSuccess,
  );
  expect(commentRes.comment_view.comment.content).toBeDefined();
  const alphaComment = await resolveComment(
    beta,
    commentRes.comment_view.comment,
  );
  if (!alphaComment) {
    throw new Error("Missing alpha comment before block");
  }

  // Check beta's inbox count, make sure it stays the same
  unreadCount = await getUnreadCounts(beta).then(expectSuccess);
  expect(unreadCount.notification_count).toBe(0);

  const replies = await listNotifications(beta, "reply", true).then(
    expectSuccess,
  );
  expect(replies.items.length).toBe(0);

  // Unblock the community
  blockRes = await blockCommunity(beta, newCommunityId, false).then(
    expectSuccess,
  );
  expect(blockRes.community_view.community_actions?.blocked_at).toBeUndefined();
});

/// Fetching a deeply nested comment can lead to stack overflow as all parent comments are also
/// fetched recursively. Ensure that it works properly.
test("Fetch a deeply nested comment", async () => {
  const alphaCommunity = await resolveCommunity(
    alpha,
    "!main@lemmy-alpha:8541",
  );
  if (!alphaCommunity) {
    throw new Error("Missing alpha community");
  }
  const postOnAlphaRes = await createPost(
    alpha,
    alphaCommunity.community.id,
  ).then(expectSuccess);
  let lastComment: CommentResponse | undefined;
  for (let i = 1; i < 50; i++) {
    const commentRes = await createComment(
      alpha,
      postOnAlphaRes.post_view.post.id,
      lastComment?.comment_view.comment.id,
    ).then(expectSuccess);
    expect(commentRes.comment_view.comment).toBeDefined();
    lastComment = commentRes;
  }

  const betaComment = await resolveComment(
    beta,
    lastComment!.comment_view.comment,
  );

  expect(betaComment?.comment).toBeDefined();
  expect(betaComment?.post).toBeDefined();
});

test("Distinguish comment", async () => {
  const community = (await resolveBetaCommunity(beta))?.community;
  const post = await createPost(beta, community!.id).then(expectSuccess);
  const commentRes = await createComment(beta, post.post_view.post.id).then(
    expectSuccess,
  );
  const form: DistinguishComment = {
    comment_id: commentRes.comment_view.comment.id,
    distinguished: true,
  };
  await beta.distinguishComment(form);

  const alphaPost = await resolvePost(alpha, post.post_view.post);

  // Find the comment on alpha (home of community)
  const alphaComments = await waitUntilSuccess(
    () => getComments(alpha, alphaPost?.post.id),
    c => c.items[0].comment.distinguished,
  );

  assertCommentFederation(alphaComments.items[0], commentRes.comment_view);
});

test("Lock comment", async () => {
  const newBetaApi = await registerUser(beta, betaUrl);

  const alphaCommunity = await resolveCommunity(
    alpha,
    "!main@lemmy-alpha:8541",
  );
  if (!alphaCommunity) {
    throw new Error("Missing alpha community");
  }

  const post = await createPost(alpha, alphaCommunity.community.id).then(
    expectSuccess,
  );
  const betaPost = await resolvePost(beta, post.post_view.post);

  if (!betaPost) {
    throw new Error("unable to locate post on beta");
  }

  // Create a comment hierarchy like this:
  // 1
  // | \
  // 2  4
  // |
  // 3

  const comment1 = await createComment(alpha, post.post_view.post.id).then(
    expectSuccess,
  );
  const betaComment1 = await resolveComment(
    beta,
    comment1.comment_view.comment,
  );
  if (!betaComment1) {
    throw new Error("unable to locate comment on beta");
  }
  await followCommunity(newBetaApi, true, betaComment1.community.id);

  const comment2 = await createComment(
    alpha,
    post.post_view.post.id,
    comment1.comment_view.comment.id,
  ).then(expectSuccess);
  const betaComment2 = await resolveComment(
    beta,
    comment2.comment_view.comment,
  );
  if (!betaComment2) {
    throw new Error("unable to locate comment on beta");
  }
  const comment3 = await createComment(
    newBetaApi,
    betaPost.post.id,
    betaComment2.comment.id,
  ).then(expectSuccess);

  // Lock comment2 and wait for it to federate
  await lockComment(alpha, true, comment2.comment_view.comment);

  const comment_ap_id = comment3.comment_view.comment.ap_id;
  await waitUntilSuccess(
    () => getComments(newBetaApi, betaPost.post.id),
    c => {
      const find = c.items.find(c => c.comment.ap_id == comment_ap_id);
      return find?.comment.locked ?? false;
    },
  );

  // Make sure newBeta can't respond to comment3
  await jestLemmyError(
    () =>
      createComment(
        newBetaApi,
        betaPost.post.id,
        comment3.comment_view.comment.id,
      ).then(expectFailure),
    new LemmyError("locked", statusBadRequest),
  );

  // newBeta should still be able to respond to comment1
  expect(
    await createComment(newBetaApi, betaPost.post.id, betaComment1.comment.id),
  ).toBeDefined();
});

test("Remove children", async () => {
  const alphaCommunity = await resolveCommunity(
    alpha,
    "!main@lemmy-alpha:8541",
  );
  if (!alphaCommunity) {
    throw new Error("Missing alpha community");
  }

  const post = await createPost(alpha, alphaCommunity.community.id).then(
    expectSuccess,
  );
  const betaPost = await resolvePost(beta, post.post_view.post);

  if (!betaPost) {
    throw new Error("unable to locate post on beta");
  }
  await followCommunity(beta, true, betaPost.community.id);

  const comment1 = await createComment(beta, betaPost.post.id).then(
    expectSuccess,
  );
  const comment2 = await createComment(
    beta,
    betaPost.post.id,
    comment1.comment_view.comment.id,
  ).then(expectSuccess);
  await createComment(beta, betaPost.post.id, comment2.comment_view.comment.id);
  await createComment(beta, betaPost.post.id, comment1.comment_view.comment.id);

  // Wait until the comments have federated
  await waitUntilSuccess(
    () => getPost(alpha, post.post_view.post.id),
    p => p.post_view.post.comments == 4,
  );

  const commentOnAlpha = await resolveComment(
    alpha,
    comment1.comment_view.comment,
  );
  if (!commentOnAlpha) {
    throw new Error("unable to locate comment on alpha");
  }

  await removeComment(alpha, true, commentOnAlpha.comment.id, true);

  const post2 = await getPost(alpha, post.post_view.post.id).then(
    expectSuccess,
  );
  expect(post2.post_view.post.comments).toBe(0);

  // Wait until the remove has federated
  await waitUntilSuccess(
    () => getComment(beta, comment1.comment_view.comment.id),
    c => c.comment_view.comment.removed,
  );

  // Make sure removal federates properly
  const betaPost2 = await resolvePost(beta, post.post_view.post);
  if (!betaPost2) {
    throw new Error("unable to locate post on beta");
  }
  expect(betaPost2.post.comments).toBe(0);
});

function checkCommentReportReason(rcv: ReportCombinedView, reason: string) {
  switch (rcv.type_) {
    case "comment":
      return rcv.comment_report.reason === reason;
    default:
      return false;
  }
}
