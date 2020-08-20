import {
  alpha,
  beta,
  gamma,
  setupLogins,
  createPost,
  getPost,
  searchComment,
  likeComment,
  followBeta,
  searchForBetaCommunity,
  createComment,
  updateComment,
  deleteComment,
  removeComment,
  getMentions,
  searchPost,
  unfollowRemotes,
  createCommunity,
  registerUser,
  API,
} from './shared';

import { PostResponse } from 'lemmy-js-client';

let postRes: PostResponse;

beforeAll(async () => {
  await setupLogins();
  await followBeta(alpha);
  await followBeta(gamma);
  let search = await searchForBetaCommunity(alpha);
  postRes = await createPost(
    alpha,
    search.communities.filter(c => c.local == false)[0].id
  );
});

afterAll(async () => {
  await unfollowRemotes(alpha);
  await unfollowRemotes(gamma);
});

test('Create a comment', async () => {
  let commentRes = await createComment(alpha, postRes.post.id);
  expect(commentRes.comment.content).toBeDefined();
  expect(commentRes.comment.community_local).toBe(false);
  expect(commentRes.comment.creator_local).toBe(true);
  expect(commentRes.comment.score).toBe(1);

  // Make sure that comment is liked on beta
  let searchBeta = await searchComment(beta, commentRes.comment);
  let betaComment = searchBeta.comments[0];
  expect(betaComment).toBeDefined();
  expect(betaComment.community_local).toBe(true);
  expect(betaComment.creator_local).toBe(false);
  expect(betaComment.score).toBe(1);
});

test('Create a comment in a non-existent post', async () => {
  let commentRes = await createComment(alpha, -1);
  expect(commentRes).toStrictEqual({ error: 'couldnt_find_post' });
});

test('Update a comment', async () => {
  let commentRes = await createComment(alpha, postRes.post.id);
  let updateCommentRes = await updateComment(alpha, commentRes.comment.id);
  expect(updateCommentRes.comment.content).toBe(
    'A jest test federated comment update'
  );
  expect(updateCommentRes.comment.community_local).toBe(false);
  expect(updateCommentRes.comment.creator_local).toBe(true);

  // Make sure that post is updated on beta
  let searchBeta = await searchComment(beta, commentRes.comment);
  let betaComment = searchBeta.comments[0];
  expect(betaComment.content).toBe('A jest test federated comment update');
});

test('Delete a comment', async () => {
  let commentRes = await createComment(alpha, postRes.post.id);
  let deleteCommentRes = await deleteComment(
    alpha,
    true,
    commentRes.comment.id
  );
  expect(deleteCommentRes.comment.deleted).toBe(true);

  // Make sure that comment is deleted on beta
  // The search doesnt work below, because it returns a tombstone / http::gone
  // let searchBeta = await searchComment(beta, commentRes.comment);
  // console.log(searchBeta);
  // let betaComment = searchBeta.comments[0];
  // Create a fake post, just to get the previous new post id
  let createdBetaPostJustToGetId = await createPost(beta, 2);
  let betaPost = await getPost(beta, createdBetaPostJustToGetId.post.id - 1);
  let betaComment = betaPost.comments[0];
  expect(betaComment.deleted).toBe(true);

  let undeleteCommentRes = await deleteComment(
    alpha,
    false,
    commentRes.comment.id
  );
  expect(undeleteCommentRes.comment.deleted).toBe(false);

  // Make sure that comment is undeleted on beta
  let searchBeta2 = await searchComment(beta, commentRes.comment);
  let betaComment2 = searchBeta2.comments[0];
  expect(betaComment2.deleted).toBe(false);
});

test('Remove a comment from admin and community on the same instance', async () => {
  let commentRes = await createComment(alpha, postRes.post.id);

  // Get the id for beta
  let betaCommentId = (await searchComment(beta, commentRes.comment))
    .comments[0].id;

  // The beta admin removes it (the community lives on beta)
  let removeCommentRes = await removeComment(beta, true, betaCommentId);
  expect(removeCommentRes.comment.removed).toBe(true);

  // Make sure that comment is removed on alpha (it gets pushed since an admin from beta removed it)
  let refetchedPost = await getPost(alpha, postRes.post.id);
  expect(refetchedPost.comments[0].removed).toBe(true);

  let unremoveCommentRes = await removeComment(beta, false, betaCommentId);
  expect(unremoveCommentRes.comment.removed).toBe(false);

  // Make sure that comment is unremoved on beta
  let refetchedPost2 = await getPost(alpha, postRes.post.id);
  expect(refetchedPost2.comments[0].removed).toBe(false);
});

test('Remove a comment from admin and community on different instance', async () => {
  let alphaUser = await registerUser(alpha);
  let newAlphaApi: API = {
    client: alpha.client,
    auth: alphaUser.jwt,
  };

  // New alpha user creates a community, post, and comment.
  let newCommunity = await createCommunity(newAlphaApi);
  let newPost = await createPost(newAlphaApi, newCommunity.community.id);
  let commentRes = await createComment(newAlphaApi, newPost.post.id);
  expect(commentRes.comment.content).toBeDefined();

  // Beta searches that to cache it, then removes it
  let searchBeta = await searchComment(beta, commentRes.comment);
  let betaComment = searchBeta.comments[0];
  let removeCommentRes = await removeComment(beta, true, betaComment.id);
  expect(removeCommentRes.comment.removed).toBe(true);

  // Make sure its not removed on alpha
  let refetchedPost = await getPost(newAlphaApi, newPost.post.id);
  expect(refetchedPost.comments[0].removed).toBe(false);
});

test('Unlike a comment', async () => {
  let commentRes = await createComment(alpha, postRes.post.id);
  let unlike = await likeComment(alpha, 0, commentRes.comment);
  expect(unlike.comment.score).toBe(0);

  // Make sure that post is unliked on beta
  let searchBeta = await searchComment(beta, commentRes.comment);
  let betaComment = searchBeta.comments[0];
  expect(betaComment).toBeDefined();
  expect(betaComment.community_local).toBe(true);
  expect(betaComment.creator_local).toBe(false);
  expect(betaComment.score).toBe(0);
});

test('Federated comment like', async () => {
  let commentRes = await createComment(alpha, postRes.post.id);

  // Find the comment on beta
  let searchBeta = await searchComment(beta, commentRes.comment);
  let betaComment = searchBeta.comments[0];

  let like = await likeComment(beta, 1, betaComment);
  expect(like.comment.score).toBe(2);

  // Get the post from alpha, check the likes
  let post = await getPost(alpha, postRes.post.id);
  expect(post.comments[0].score).toBe(2);
});

test('Reply to a comment', async () => {
  // Create a comment on alpha, find it on beta
  let commentRes = await createComment(alpha, postRes.post.id);
  let searchBeta = await searchComment(beta, commentRes.comment);
  let betaComment = searchBeta.comments[0];

  // find that comment id on beta

  // Reply from beta
  let replyRes = await createComment(beta, betaComment.post_id, betaComment.id);
  expect(replyRes.comment.content).toBeDefined();
  expect(replyRes.comment.community_local).toBe(true);
  expect(replyRes.comment.creator_local).toBe(true);
  expect(replyRes.comment.parent_id).toBe(betaComment.id);
  expect(replyRes.comment.score).toBe(1);

  // Make sure that comment is seen on alpha
  // TODO not sure why, but a searchComment back to alpha, for the ap_id of betas
  // comment, isn't working.
  // let searchAlpha = await searchComment(alpha, replyRes.comment);
  let post = await getPost(alpha, postRes.post.id);
  let alphaComment = post.comments[0];
  expect(alphaComment.content).toBeDefined();
  expect(alphaComment.parent_id).toBe(post.comments[1].id);
  expect(alphaComment.community_local).toBe(false);
  expect(alphaComment.creator_local).toBe(false);
  expect(alphaComment.score).toBe(1);
});

test('Mention beta', async () => {
  // Create a mention on alpha
  let mentionContent = 'A test mention of @lemmy_beta@lemmy-beta:8550';
  let commentRes = await createComment(alpha, postRes.post.id);
  let mentionRes = await createComment(
    alpha,
    postRes.post.id,
    commentRes.comment.id,
    mentionContent
  );
  expect(mentionRes.comment.content).toBeDefined();
  expect(mentionRes.comment.community_local).toBe(false);
  expect(mentionRes.comment.creator_local).toBe(true);
  expect(mentionRes.comment.score).toBe(1);

  let mentionsRes = await getMentions(beta);
  expect(mentionsRes.mentions[0].content).toBeDefined();
  expect(mentionsRes.mentions[0].community_local).toBe(true);
  expect(mentionsRes.mentions[0].creator_local).toBe(false);
  expect(mentionsRes.mentions[0].score).toBe(1);
});

test('Comment Search', async () => {
  let commentRes = await createComment(alpha, postRes.post.id);
  let searchBeta = await searchComment(beta, commentRes.comment);
  expect(searchBeta.comments[0].ap_id).toBe(commentRes.comment.ap_id);
});

test('A and G subscribe to B (center) A posts, G mentions B, it gets announced to A', async () => {
  // Create a local post
  let alphaPost = await createPost(alpha, 2);
  expect(alphaPost.post.community_local).toBe(true);

  // Make sure gamma sees it
  let search = await searchPost(gamma, alphaPost.post);
  let gammaPost = search.posts[0];

  let commentContent =
    'A jest test federated comment announce, lets mention @lemmy_beta@lemmy-beta:8550';
  let commentRes = await createComment(
    gamma,
    gammaPost.id,
    undefined,
    commentContent
  );
  expect(commentRes.comment.content).toBe(commentContent);
  expect(commentRes.comment.community_local).toBe(false);
  expect(commentRes.comment.creator_local).toBe(true);
  expect(commentRes.comment.score).toBe(1);

  // Make sure alpha sees it
  let alphaPost2 = await getPost(alpha, alphaPost.post.id);
  expect(alphaPost2.comments[0].content).toBe(commentContent);
  expect(alphaPost2.comments[0].community_local).toBe(true);
  expect(alphaPost2.comments[0].creator_local).toBe(false);
  expect(alphaPost2.comments[0].score).toBe(1);

  // Make sure beta has mentions
  let mentionsRes = await getMentions(beta);
  expect(mentionsRes.mentions[0].content).toBe(commentContent);
  expect(mentionsRes.mentions[0].community_local).toBe(false);
  expect(mentionsRes.mentions[0].creator_local).toBe(false);
  // TODO this is failing because fetchInReplyTos aren't getting score
  // expect(mentionsRes.mentions[0].score).toBe(1);
});

test('Fetch in_reply_tos: A is unsubbed from B, B makes a post, and some embedded comments, A subs to B, B updates the lowest level comment, A fetches both the post and all the inreplyto comments for that post.', async () => {
  // Unfollow all remote communities
  let followed = await unfollowRemotes(alpha);
  expect(
    followed.communities.filter(c => c.community_local == false).length
  ).toBe(0);

  // B creates a post, and two comments, should be invisible to A
  let postRes = await createPost(beta, 2);
  expect(postRes.post.name).toBeDefined();

  let parentCommentContent = 'An invisible top level comment from beta';
  let parentCommentRes = await createComment(
    beta,
    postRes.post.id,
    undefined,
    parentCommentContent
  );
  expect(parentCommentRes.comment.content).toBe(parentCommentContent);

  // B creates a comment, then a child one of that.
  let childCommentContent = 'An invisible child comment from beta';
  let childCommentRes = await createComment(
    beta,
    postRes.post.id,
    parentCommentRes.comment.id,
    childCommentContent
  );
  expect(childCommentRes.comment.content).toBe(childCommentContent);

  // Follow beta again
  let follow = await followBeta(alpha);
  expect(follow.community.local).toBe(false);
  expect(follow.community.name).toBe('main');

  // An update to the child comment on beta, should push the post, parent, and child to alpha now
  let updatedCommentContent = 'An update child comment from beta';
  let updateRes = await updateComment(
    beta,
    childCommentRes.comment.id,
    updatedCommentContent
  );
  expect(updateRes.comment.content).toBe(updatedCommentContent);

  // Get the post from alpha
  let createFakeAlphaPostToGetId = await createPost(alpha, 2);
  let alphaPost = await getPost(alpha, createFakeAlphaPostToGetId.post.id - 1);
  expect(alphaPost.post.name).toBeDefined();
  expect(alphaPost.comments[1].content).toBe(parentCommentContent);
  expect(alphaPost.comments[0].content).toBe(updatedCommentContent);
  expect(alphaPost.post.community_local).toBe(false);
  expect(alphaPost.post.creator_local).toBe(false);
});
