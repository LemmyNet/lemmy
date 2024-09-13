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
  getMentions,
  resolvePost,
  unfollowRemotes,
  createCommunity,
  registerUser,
  reportComment,
  listCommentReports,
  randomString,
  unfollows,
  getComments,
  getCommentParentId,
  resolveCommunity,
  getPersonDetails,
  getReplies,
  getUnreadCount,
  waitUntil,
  waitForPost,
  alphaUrl,
  followCommunity,
  blockCommunity,
  delay,
  saveUserSettings,
} from "./shared";
import { CommentView, CommunityView, SaveUserSettings } from "lemmy-js-client";

let betaCommunity: CommunityView | undefined;
let postOnAlphaRes: PostResponse;

beforeAll(async () => {
  await setupLogins();
  await Promise.all([followBeta(alpha), followBeta(gamma)]);
  betaCommunity = (await resolveBetaCommunity(alpha)).community;
  if (betaCommunity) {
    postOnAlphaRes = await createPost(alpha, betaCommunity.community.id);
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
  expect(commentOne?.community.actor_id).toBe(commentTwo?.community.actor_id);
  expect(commentOne?.comment.published).toBe(commentTwo?.comment.published);
  expect(commentOne?.comment.updated).toBe(commentOne?.comment.updated);
  expect(commentOne?.comment.deleted).toBe(commentOne?.comment.deleted);
  expect(commentOne?.comment.removed).toBe(commentOne?.comment.removed);
}

test("Create a comment", async () => {
  let commentRes = await createComment(alpha, postOnAlphaRes.post_view.post.id);
  expect(commentRes.comment_view.comment.content).toBeDefined();
  expect(commentRes.comment_view.community.local).toBe(false);
  expect(commentRes.comment_view.creator.local).toBe(true);
  expect(commentRes.comment_view.counts.score).toBe(1);

  // Make sure that comment is liked on beta
  let betaComment = (
    await waitUntil(
      () => resolveComment(beta, commentRes.comment_view.comment),
      c => c.comment?.counts.score === 1,
    )
  ).comment;
  expect(betaComment).toBeDefined();
  expect(betaComment?.community.local).toBe(true);
  expect(betaComment?.creator.local).toBe(false);
  expect(betaComment?.counts.score).toBe(1);
  assertCommentFederation(betaComment, commentRes.comment_view);
});

test("Create a comment in a non-existent post", async () => {
  await expect(createComment(alpha, -1)).rejects.toStrictEqual(
    Error("couldnt_find_post"),
  );
});

test("Update a comment", async () => {
  let commentRes = await createComment(alpha, postOnAlphaRes.post_view.post.id);
  // Federate the comment first
  let betaComment = (
    await resolveComment(beta, commentRes.comment_view.comment)
  ).comment;
  assertCommentFederation(betaComment, commentRes.comment_view);

  let updateCommentRes = await editComment(
    alpha,
    commentRes.comment_view.comment.id,
  );
  expect(updateCommentRes.comment_view.comment.content).toBe(
    "A jest test federated comment update",
  );
  expect(updateCommentRes.comment_view.community.local).toBe(false);
  expect(updateCommentRes.comment_view.creator.local).toBe(true);

  // Make sure that post is updated on beta
  let betaCommentUpdated = (
    await waitUntil(
      () => resolveComment(beta, commentRes.comment_view.comment),
      c =>
        c.comment?.comment.content === "A jest test federated comment update",
    )
  ).comment;
  assertCommentFederation(betaCommentUpdated, updateCommentRes.comment_view);
});

test("Delete a comment", async () => {
  let post = await createPost(alpha, betaCommunity!.community.id);
  // creating a comment on alpha (remote from home of community)
  let commentRes = await createComment(alpha, post.post_view.post.id);

  // Find the comment on beta (home of community)
  let betaComment = (
    await resolveComment(beta, commentRes.comment_view.comment)
  ).comment;
  if (!betaComment) {
    throw "Missing beta comment before delete";
  }

  // Find the comment on remote instance gamma
  let gammaComment = (
    await waitUntil(
      () =>
        resolveComment(gamma, commentRes.comment_view.comment).catch(e => e),
      r => r.message !== "couldnt_find_object",
    )
  ).comment;
  if (!gammaComment) {
    throw "Missing gamma comment (remote-home-remote replication) before delete";
  }

  let deleteCommentRes = await deleteComment(
    alpha,
    true,
    commentRes.comment_view.comment.id,
  );
  expect(deleteCommentRes.comment_view.comment.deleted).toBe(true);
  expect(deleteCommentRes.comment_view.comment.content).toBe("");

  // Make sure that comment is undefined on beta
  await waitUntil(
    () => resolveComment(beta, commentRes.comment_view.comment).catch(e => e),
    e => e.message == "couldnt_find_object",
  );

  // Make sure that comment is undefined on gamma after delete
  await waitUntil(
    () => resolveComment(gamma, commentRes.comment_view.comment).catch(e => e),
    e => e.message === "couldnt_find_object",
  );

  // Test undeleting the comment
  let undeleteCommentRes = await deleteComment(
    alpha,
    false,
    commentRes.comment_view.comment.id,
  );
  expect(undeleteCommentRes.comment_view.comment.deleted).toBe(false);

  // Make sure that comment is undeleted on beta
  let betaComment2 = (
    await waitUntil(
      () => resolveComment(beta, commentRes.comment_view.comment).catch(e => e),
      e => e.message !== "couldnt_find_object",
    )
  ).comment;
  expect(betaComment2?.comment.deleted).toBe(false);
  assertCommentFederation(betaComment2, undeleteCommentRes.comment_view);
});

test.skip("Remove a comment from admin and community on the same instance", async () => {
  let commentRes = await createComment(alpha, postOnAlphaRes.post_view.post.id);

  // Get the id for beta
  let betaCommentId = (
    await resolveComment(beta, commentRes.comment_view.comment)
  ).comment?.comment.id;

  if (!betaCommentId) {
    throw "beta comment id is missing";
  }

  // The beta admin removes it (the community lives on beta)
  let removeCommentRes = await removeComment(beta, true, betaCommentId);
  expect(removeCommentRes.comment_view.comment.removed).toBe(true);

  // Make sure that comment is removed on alpha (it gets pushed since an admin from beta removed it)
  let refetchedPostComments = await getPersonDetails(
    alpha,
    commentRes.comment_view.comment.creator_id,
  );
  expect(refetchedPostComments.comments[0].comment.removed).toBe(true);

  // beta will unremove the comment
  let unremoveCommentRes = await removeComment(beta, false, betaCommentId);
  expect(unremoveCommentRes.comment_view.comment.removed).toBe(false);

  // Make sure that comment is unremoved on alpha
  let refetchedPostComments2 = await getComments(
    alpha,
    postOnAlphaRes.post_view.post.id,
  );
  expect(refetchedPostComments2.comments[0].comment.removed).toBe(false);
  assertCommentFederation(
    refetchedPostComments2.comments[0],
    unremoveCommentRes.comment_view,
  );
});

test("Remove a comment from admin and community on different instance", async () => {
  let newAlphaApi = await registerUser(alpha, alphaUrl);

  // New alpha user creates a community, post, and comment.
  let newCommunity = await createCommunity(newAlphaApi);
  let newPost = await createPost(
    newAlphaApi,
    newCommunity.community_view.community.id,
  );
  let commentRes = await createComment(newAlphaApi, newPost.post_view.post.id);
  expect(commentRes.comment_view.comment.content).toBeDefined();

  // Beta searches that to cache it, then removes it
  let betaComment = (
    await resolveComment(beta, commentRes.comment_view.comment)
  ).comment;

  if (!betaComment) {
    throw "beta comment missing";
  }

  let removeCommentRes = await removeComment(
    beta,
    true,
    betaComment.comment.id,
  );
  expect(removeCommentRes.comment_view.comment.removed).toBe(true);
  expect(removeCommentRes.comment_view.comment.content).toBe("");

  // Comment text is also hidden from list
  let listComments = await getComments(
    beta,
    removeCommentRes.comment_view.post.id,
  );
  expect(listComments.comments.length).toBe(1);
  expect(listComments.comments[0].comment.removed).toBe(true);
  expect(listComments.comments[0].comment.content).toBe("");

  // Make sure its not removed on alpha
  let refetchedPostComments = await getComments(
    alpha,
    newPost.post_view.post.id,
  );
  expect(refetchedPostComments.comments[0].comment.removed).toBe(false);
  assertCommentFederation(
    refetchedPostComments.comments[0],
    commentRes.comment_view,
  );
});

test("Unlike a comment", async () => {
  let commentRes = await createComment(alpha, postOnAlphaRes.post_view.post.id);

  // Lemmy automatically creates 1 like (vote) by author of comment.
  // Make sure that comment is liked (voted up) on gamma, downstream peer
  // This is testing replication from remote-home-remote (alpha-beta-gamma)

  let gammaComment1 = (
    await waitUntil(
      () => resolveComment(gamma, commentRes.comment_view.comment),
      c => c.comment?.counts.score === 1,
    )
  ).comment;
  expect(gammaComment1).toBeDefined();
  expect(gammaComment1?.community.local).toBe(false);
  expect(gammaComment1?.creator.local).toBe(false);
  expect(gammaComment1?.counts.score).toBe(1);

  let unlike = await likeComment(alpha, 0, commentRes.comment_view.comment);
  expect(unlike.comment_view.counts.score).toBe(0);

  // Make sure that comment is unliked on beta
  let betaComment = (
    await waitUntil(
      () => resolveComment(beta, commentRes.comment_view.comment),
      c => c.comment?.counts.score === 0,
    )
  ).comment;
  expect(betaComment).toBeDefined();
  expect(betaComment?.community.local).toBe(true);
  expect(betaComment?.creator.local).toBe(false);
  expect(betaComment?.counts.score).toBe(0);

  // Make sure that comment is unliked on gamma, downstream peer
  // This is testing replication from remote-home-remote (alpha-beta-gamma)
  let gammaComment = (
    await waitUntil(
      () => resolveComment(gamma, commentRes.comment_view.comment),
      c => c.comment?.counts.score === 0,
    )
  ).comment;
  expect(gammaComment).toBeDefined();
  expect(gammaComment?.community.local).toBe(false);
  expect(gammaComment?.creator.local).toBe(false);
  expect(gammaComment?.counts.score).toBe(0);
});

test("Federated comment like", async () => {
  let commentRes = await createComment(alpha, postOnAlphaRes.post_view.post.id);
  await waitUntil(
    () => resolveComment(beta, commentRes.comment_view.comment),
    c => c.comment?.counts.score === 1,
  );
  // Find the comment on beta
  let betaComment = (
    await resolveComment(beta, commentRes.comment_view.comment)
  ).comment;

  if (!betaComment) {
    throw "Missing beta comment";
  }

  let like = await likeComment(beta, 1, betaComment.comment);
  expect(like.comment_view.counts.score).toBe(2);

  // Get the post from alpha, check the likes
  let postComments = await waitUntil(
    () => getComments(alpha, postOnAlphaRes.post_view.post.id),
    c => c.comments[0].counts.score === 2,
  );
  expect(postComments.comments[0].counts.score).toBe(2);
});

test("Reply to a comment from another instance, get notification", async () => {
  await alpha.markAllAsRead();

  let betaCommunity = (
    await waitUntil(
      () => resolveBetaCommunity(alpha),
      c => !!c.community?.community.instance_id,
    )
  ).community;
  if (!betaCommunity) {
    throw "Missing beta community";
  }

  const postOnAlphaRes = await createPost(alpha, betaCommunity.community.id);

  // Create a root-level trunk-branch comment on alpha
  let commentRes = await createComment(alpha, postOnAlphaRes.post_view.post.id);
  // find that comment id on beta
  let betaComment = (
    await waitUntil(
      () => resolveComment(beta, commentRes.comment_view.comment),
      c => c.comment?.counts.score === 1,
    )
  ).comment;

  if (!betaComment) {
    throw "Missing beta comment";
  }

  // Reply from beta, extending the branch
  let replyRes = await createComment(
    beta,
    betaComment.post.id,
    betaComment.comment.id,
  );
  expect(replyRes.comment_view.comment.content).toBeDefined();
  expect(replyRes.comment_view.community.local).toBe(true);
  expect(replyRes.comment_view.creator.local).toBe(true);
  expect(getCommentParentId(replyRes.comment_view.comment)).toBe(
    betaComment.comment.id,
  );
  expect(replyRes.comment_view.counts.score).toBe(1);

  // Make sure that reply comment is seen on alpha
  let commentSearch = await waitUntil(
    () => resolveComment(alpha, replyRes.comment_view.comment),
    c => c.comment?.counts.score === 1,
  );
  let alphaComment = commentSearch.comment!;
  let postComments = await waitUntil(
    () => getComments(alpha, postOnAlphaRes.post_view.post.id),
    pc => pc.comments.length >= 2,
  );
  // Note: this test fails when run twice and this count will differ
  expect(postComments.comments.length).toBeGreaterThanOrEqual(2);
  expect(alphaComment.comment.content).toBeDefined();

  expect(getCommentParentId(alphaComment.comment)).toBe(
    postComments.comments[1].comment.id,
  );
  expect(alphaComment.community.local).toBe(false);
  expect(alphaComment.creator.local).toBe(false);
  expect(alphaComment.counts.score).toBe(1);
  assertCommentFederation(alphaComment, replyRes.comment_view);

  // Did alpha get notified of the reply from beta?
  let alphaUnreadCountRes = await waitUntil(
    () => getUnreadCount(alpha),
    e => e.replies >= 1,
  );
  expect(alphaUnreadCountRes.replies).toBeGreaterThanOrEqual(1);

  // check inbox of replies on alpha, fetching read/unread both
  let alphaRepliesRes = await waitUntil(
    () => getReplies(alpha),
    r => r.replies.length > 0,
  );
  const alphaReply = alphaRepliesRes.replies.find(
    r => r.comment.id === alphaComment.comment.id,
  );
  expect(alphaReply).toBeDefined();
  if (!alphaReply) throw Error();
  expect(alphaReply.comment.content).toBeDefined();
  expect(alphaReply.community.local).toBe(false);
  expect(alphaReply.creator.local).toBe(false);
  expect(alphaReply.counts.score).toBe(1);
  // ToDo: interesting alphaRepliesRes.replies[0].comment_reply.id is 1, meaning? how did that come about?
  expect(alphaReply.comment.id).toBe(alphaComment.comment.id);
  // this is a new notification, getReplies fetch was for read/unread both, confirm it is unread.
  expect(alphaReply.comment_reply.read).toBe(false);
  assertCommentFederation(alphaReply, replyRes.comment_view);
});

test("Bot reply notifications are filtered when bots are hidden", async () => {
  const newAlphaBot = await registerUser(alpha, alphaUrl);
  let form: SaveUserSettings = {
    bot_account: true,
  };
  await saveUserSettings(newAlphaBot, form);

  const alphaCommunity = (
    await resolveCommunity(alpha, "!main@lemmy-alpha:8541")
  ).community;

  if (!alphaCommunity) {
    throw "Missing alpha community";
  }

  await alpha.markAllAsRead();
  form = {
    show_bot_accounts: false,
  };
  await saveUserSettings(alpha, form);
  const postOnAlphaRes = await createPost(alpha, alphaCommunity.community.id);

  // Bot reply to alpha's post
  let commentRes = await createComment(
    newAlphaBot,
    postOnAlphaRes.post_view.post.id,
  );
  expect(commentRes).toBeDefined();

  let alphaUnreadCountRes = await getUnreadCount(alpha);
  expect(alphaUnreadCountRes.replies).toBe(0);

  let alphaUnreadRepliesRes = await getReplies(alpha, true);
  expect(alphaUnreadRepliesRes.replies.length).toBe(0);

  // This both restores the original state that may be expected by other tests
  // implicitly and is used by the next steps to ensure replies are still
  // returned when a user later decides to show bot accounts again.
  form = {
    show_bot_accounts: true,
  };
  await saveUserSettings(alpha, form);

  alphaUnreadCountRes = await getUnreadCount(alpha);
  expect(alphaUnreadCountRes.replies).toBe(1);

  alphaUnreadRepliesRes = await getReplies(alpha, true);
  expect(alphaUnreadRepliesRes.replies.length).toBe(1);
  expect(alphaUnreadRepliesRes.replies[0].comment.id).toBe(
    commentRes.comment_view.comment.id,
  );
});

test("Mention beta from alpha", async () => {
  if (!betaCommunity) throw Error("no community");
  const postOnAlphaRes = await createPost(alpha, betaCommunity.community.id);
  // Create a new branch, trunk-level comment branch, from alpha instance
  let commentRes = await createComment(alpha, postOnAlphaRes.post_view.post.id);
  // Create a reply comment to previous comment, this has a mention in body
  let mentionContent = "A test mention of @lemmy_beta@lemmy-beta:8551";
  let mentionRes = await createComment(
    alpha,
    postOnAlphaRes.post_view.post.id,
    commentRes.comment_view.comment.id,
    mentionContent,
  );
  expect(mentionRes.comment_view.comment.content).toBeDefined();
  expect(mentionRes.comment_view.community.local).toBe(false);
  expect(mentionRes.comment_view.creator.local).toBe(true);
  expect(mentionRes.comment_view.counts.score).toBe(1);

  // get beta's localized copy of the alpha post
  let betaPost = await waitForPost(beta, postOnAlphaRes.post_view.post);
  if (!betaPost) {
    throw "unable to locate post on beta";
  }
  expect(betaPost.post.ap_id).toBe(postOnAlphaRes.post_view.post.ap_id);
  expect(betaPost.post.name).toBe(postOnAlphaRes.post_view.post.name);

  // Make sure that both new comments are seen on beta and have parent/child relationship
  let betaPostComments = await waitUntil(
    () => getComments(beta, betaPost!.post.id),
    c => c.comments[1]?.counts.score === 1,
  );
  expect(betaPostComments.comments.length).toEqual(2);
  // the trunk-branch root comment will be older than the mention reply comment, so index 1
  let betaRootComment = betaPostComments.comments[1];
  // the trunk-branch root comment should not have a parent
  expect(getCommentParentId(betaRootComment.comment)).toBeUndefined();
  expect(betaRootComment.comment.content).toBeDefined();
  // the mention reply comment should have parent that points to the branch root level comment
  expect(getCommentParentId(betaPostComments.comments[0].comment)).toBe(
    betaPostComments.comments[1].comment.id,
  );
  expect(betaRootComment.community.local).toBe(true);
  expect(betaRootComment.creator.local).toBe(false);
  expect(betaRootComment.counts.score).toBe(1);
  assertCommentFederation(betaRootComment, commentRes.comment_view);

  let mentionsRes = await waitUntil(
    () => getMentions(beta),
    m => !!m.mentions[0],
  );
  expect(mentionsRes.mentions[0].comment.content).toBeDefined();
  expect(mentionsRes.mentions[0].community.local).toBe(true);
  expect(mentionsRes.mentions[0].creator.local).toBe(false);
  expect(mentionsRes.mentions[0].counts.score).toBe(1);
  // the reply comment with mention should be the most fresh, newest, index 0
  expect(mentionsRes.mentions[0].person_mention.comment_id).toBe(
    betaPostComments.comments[0].comment.id,
  );
});

test("Comment Search", async () => {
  let commentRes = await createComment(alpha, postOnAlphaRes.post_view.post.id);
  let betaComment = (
    await resolveComment(beta, commentRes.comment_view.comment)
  ).comment;
  assertCommentFederation(betaComment, commentRes.comment_view);
});

test("A and G subscribe to B (center) A posts, G mentions B, it gets announced to A", async () => {
  // Create a local post
  let alphaCommunity = (await resolveCommunity(alpha, "!main@lemmy-alpha:8541"))
    .community;

  if (!alphaCommunity) {
    throw "Missing alpha community";
  }

  // follow community from beta so that it accepts the mention
  let betaCommunity = await resolveCommunity(
    beta,
    alphaCommunity.community.actor_id,
  );
  await followCommunity(beta, true, betaCommunity.community!.community.id);

  let alphaPost = await createPost(alpha, alphaCommunity.community.id);
  expect(alphaPost.post_view.community.local).toBe(true);

  // Make sure gamma sees it
  let gammaPost = (await resolvePost(gamma, alphaPost.post_view.post))!.post;

  if (!gammaPost) {
    throw "Missing gamma post";
  }

  let commentContent =
    "A jest test federated comment announce, lets mention @lemmy_beta@lemmy-beta:8551";
  let commentRes = await createComment(
    gamma,
    gammaPost.post.id,
    undefined,
    commentContent,
  );
  expect(commentRes.comment_view.comment.content).toBe(commentContent);
  expect(commentRes.comment_view.community.local).toBe(false);
  expect(commentRes.comment_view.creator.local).toBe(true);
  expect(commentRes.comment_view.counts.score).toBe(1);

  // Make sure alpha sees it
  let alphaPostComments2 = await waitUntil(
    () => getComments(alpha, alphaPost.post_view.post.id),
    e => e.comments[0]?.counts.score === 1,
  );
  expect(alphaPostComments2.comments[0].comment.content).toBe(commentContent);
  expect(alphaPostComments2.comments[0].community.local).toBe(true);
  expect(alphaPostComments2.comments[0].creator.local).toBe(false);
  expect(alphaPostComments2.comments[0].counts.score).toBe(1);
  assertCommentFederation(
    alphaPostComments2.comments[0],
    commentRes.comment_view,
  );

  // Make sure beta has mentions
  let relevantMention = await waitUntil(
    () =>
      getMentions(beta).then(m =>
        m.mentions.find(
          m => m.comment.ap_id === commentRes.comment_view.comment.ap_id,
        ),
      ),
    e => !!e,
  );
  if (!relevantMention) throw Error("could not find mention");
  expect(relevantMention.comment.content).toBe(commentContent);
  expect(relevantMention.community.local).toBe(false);
  expect(relevantMention.creator.local).toBe(false);
  // TODO this is failing because fetchInReplyTos aren't getting score
  // expect(mentionsRes.mentions[0].score).toBe(1);
});

test("Check that activity from another instance is sent to third instance", async () => {
  // Alpha and gamma users follow beta community
  let alphaFollow = await followBeta(alpha);
  expect(alphaFollow.community_view.community.local).toBe(false);
  expect(alphaFollow.community_view.community.name).toBe("main");

  let gammaFollow = await followBeta(gamma);
  expect(gammaFollow.community_view.community.local).toBe(false);
  expect(gammaFollow.community_view.community.name).toBe("main");
  await waitUntil(
    () => resolveBetaCommunity(alpha),
    c => c.community?.subscribed === "Subscribed",
  );
  await waitUntil(
    () => resolveBetaCommunity(gamma),
    c => c.community?.subscribed === "Subscribed",
  );

  // Create a post on beta
  let betaPost = await createPost(beta, 2);
  expect(betaPost.post_view.community.local).toBe(true);

  // Make sure gamma and alpha see it
  let gammaPost = await waitForPost(gamma, betaPost.post_view.post);
  if (!gammaPost) {
    throw "Missing gamma post";
  }
  expect(gammaPost.post).toBeDefined();

  let alphaPost = await waitForPost(alpha, betaPost.post_view.post);
  if (!alphaPost) {
    throw "Missing alpha post";
  }
  expect(alphaPost.post).toBeDefined();

  // The bug: gamma comments, and alpha should see it.
  let commentContent = "Comment from gamma";
  let commentRes = await createComment(
    gamma,
    gammaPost.post.id,
    undefined,
    commentContent,
  );
  expect(commentRes.comment_view.comment.content).toBe(commentContent);
  expect(commentRes.comment_view.community.local).toBe(false);
  expect(commentRes.comment_view.creator.local).toBe(true);
  expect(commentRes.comment_view.counts.score).toBe(1);

  // Make sure alpha sees it
  let alphaPostComments2 = await waitUntil(
    () => getComments(alpha, alphaPost!.post.id),
    e => e.comments[0]?.counts.score === 1,
  );
  expect(alphaPostComments2.comments[0].comment.content).toBe(commentContent);
  expect(alphaPostComments2.comments[0].community.local).toBe(false);
  expect(alphaPostComments2.comments[0].creator.local).toBe(false);
  expect(alphaPostComments2.comments[0].counts.score).toBe(1);
  assertCommentFederation(
    alphaPostComments2.comments[0],
    commentRes.comment_view,
  );

  await Promise.all([unfollowRemotes(alpha), unfollowRemotes(gamma)]);
});

test("Fetch in_reply_tos: A is unsubbed from B, B makes a post, and some embedded comments, A subs to B, B updates the lowest level comment, A fetches both the post and all the inreplyto comments for that post.", async () => {
  // Unfollow all remote communities
  let site = await unfollowRemotes(alpha);
  expect(
    site.my_user?.follows.filter(c => c.community.local == false).length,
  ).toBe(0);

  // B creates a post, and two comments, should be invisible to A
  let postOnBetaRes = await createPost(beta, 2);
  expect(postOnBetaRes.post_view.post.name).toBeDefined();

  let parentCommentContent = "An invisible top level comment from beta";
  let parentCommentRes = await createComment(
    beta,
    postOnBetaRes.post_view.post.id,
    undefined,
    parentCommentContent,
  );
  expect(parentCommentRes.comment_view.comment.content).toBe(
    parentCommentContent,
  );

  // B creates a comment, then a child one of that.
  let childCommentContent = "An invisible child comment from beta";
  let childCommentRes = await createComment(
    beta,
    postOnBetaRes.post_view.post.id,
    parentCommentRes.comment_view.comment.id,
    childCommentContent,
  );
  expect(childCommentRes.comment_view.comment.content).toBe(
    childCommentContent,
  );

  // Follow beta again
  let follow = await followBeta(alpha);
  expect(follow.community_view.community.local).toBe(false);
  expect(follow.community_view.community.name).toBe("main");

  // An update to the child comment on beta, should push the post, parent, and child to alpha now
  let updatedCommentContent = "An update child comment from beta";
  let updateRes = await editComment(
    beta,
    childCommentRes.comment_view.comment.id,
    updatedCommentContent,
  );
  expect(updateRes.comment_view.comment.content).toBe(updatedCommentContent);

  // Get the post from alpha
  let alphaPostB = await waitForPost(alpha, postOnBetaRes.post_view.post);

  if (!alphaPostB) {
    throw "Missing alpha post B";
  }

  let alphaPost = await getPost(alpha, alphaPostB.post.id);
  let alphaPostComments = await waitUntil(
    () => getComments(alpha, alphaPostB!.post.id),
    c =>
      c.comments[1]?.comment.content ===
        parentCommentRes.comment_view.comment.content &&
      c.comments[0]?.comment.content === updateRes.comment_view.comment.content,
  );
  expect(alphaPost.post_view.post.name).toBeDefined();
  assertCommentFederation(
    alphaPostComments.comments[1],
    parentCommentRes.comment_view,
  );
  assertCommentFederation(
    alphaPostComments.comments[0],
    updateRes.comment_view,
  );
  expect(alphaPost.post_view.community.local).toBe(false);
  expect(alphaPost.post_view.creator.local).toBe(false);

  await unfollowRemotes(alpha);
});

test("Report a comment", async () => {
  let betaCommunity = (await resolveBetaCommunity(beta)).community;
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let postOnBetaRes = (await createPost(beta, betaCommunity.community.id))
    .post_view.post;
  expect(postOnBetaRes).toBeDefined();
  let commentRes = (await createComment(beta, postOnBetaRes.id)).comment_view
    .comment;
  expect(commentRes).toBeDefined();

  let alphaComment = (await resolveComment(alpha, commentRes)).comment?.comment;
  if (!alphaComment) {
    throw "Missing alpha comment";
  }

  const reason = randomString(10);
  let alphaReport = (await reportComment(alpha, alphaComment.id, reason))
    .comment_report_view.comment_report;

  let betaReport = (await waitUntil(
    () =>
      listCommentReports(beta).then(r =>
        r.comment_reports.find(rep => rep.comment_report.reason === reason),
      ),
    e => !!e,
  ))!.comment_report;
  expect(betaReport).toBeDefined();
  expect(betaReport.resolved).toBe(false);
  expect(betaReport.original_comment_text).toBe(
    alphaReport.original_comment_text,
  );
  expect(betaReport.reason).toBe(alphaReport.reason);
});

test("Dont send a comment reply to a blocked community", async () => {
  let newCommunity = await createCommunity(beta);
  let newCommunityId = newCommunity.community_view.community.id;

  // Create a post on beta
  let betaPost = await createPost(beta, newCommunityId);

  let alphaPost = (await resolvePost(alpha, betaPost.post_view.post))!.post;
  if (!alphaPost) {
    throw "unable to locate post on alpha";
  }

  // Check beta's inbox count
  let unreadCount = await getUnreadCount(beta);
  expect(unreadCount.replies).toBe(1);

  // Beta blocks the new beta community
  let blockRes = await blockCommunity(beta, newCommunityId, true);
  expect(blockRes.blocked).toBe(true);
  delay();

  // Alpha creates a comment
  let commentRes = await createComment(alpha, alphaPost.post.id);
  expect(commentRes.comment_view.comment.content).toBeDefined();
  let alphaComment = await resolveComment(
    beta,
    commentRes.comment_view.comment,
  );
  if (!alphaComment) {
    throw "Missing alpha comment before block";
  }

  // Check beta's inbox count, make sure it stays the same
  unreadCount = await getUnreadCount(beta);
  expect(unreadCount.replies).toBe(1);

  let replies = await getReplies(beta);
  expect(replies.replies.length).toBe(1);

  // Unblock the community
  blockRes = await blockCommunity(beta, newCommunityId, false);
  expect(blockRes.blocked).toBe(false);
});
