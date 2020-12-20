jest.setTimeout(120000);
import {
  alpha,
  beta,
  gamma,
  delta,
  epsilon,
  setupLogins,
  createPost,
  editPost,
  stickyPost,
  lockPost,
  searchPost,
  likePost,
  followBeta,
  searchForBetaCommunity,
  createComment,
  deletePost,
  removePost,
  getPost,
  unfollowRemotes,
  delay,
  longDelay,
  searchForUser,
  banUserFromSite,
  searchPostLocal,
  banUserFromCommunity,
} from './shared';
import { PostView } from 'lemmy-js-client';

beforeAll(async () => {
  await setupLogins();
  await followBeta(alpha);
  await followBeta(gamma);
  await followBeta(delta);
  await followBeta(epsilon);
  await longDelay();
});

afterAll(async () => {
  await unfollowRemotes(alpha);
  await unfollowRemotes(gamma);
  await unfollowRemotes(delta);
  await unfollowRemotes(epsilon);
});

function assertPostFederation(postOne: PostView, postTwo: PostView) {
  expect(postOne.post.ap_id).toBe(postTwo.post.ap_id);
  expect(postOne.post.name).toBe(postTwo.post.name);
  expect(postOne.post.body).toBe(postTwo.post.body);
  expect(postOne.post.url).toBe(postTwo.post.url);
  expect(postOne.post.nsfw).toBe(postTwo.post.nsfw);
  expect(postOne.post.embed_title).toBe(postTwo.post.embed_title);
  expect(postOne.post.embed_description).toBe(postTwo.post.embed_description);
  expect(postOne.post.embed_html).toBe(postTwo.post.embed_html);
  expect(postOne.post.published).toBe(postTwo.post.published);
  expect(postOne.community.actor_id).toBe(postTwo.community.actor_id);
  expect(postOne.post.locked).toBe(postTwo.post.locked);
  expect(postOne.post.removed).toBe(postTwo.post.removed);
  expect(postOne.post.deleted).toBe(postTwo.post.deleted);
}

test('Create a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  await delay();
  let postRes = await createPost(alpha, search.communities[0].community.id);
  expect(postRes.post_view.post).toBeDefined();
  expect(postRes.post_view.community.local).toBe(false);
  expect(postRes.post_view.creator.local).toBe(true);
  expect(postRes.post_view.counts.score).toBe(1);
  await longDelay();

  // Make sure that post is liked on beta
  let searchBeta = await searchPost(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];

  expect(betaPost).toBeDefined();
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.counts.score).toBe(1);
  assertPostFederation(betaPost, postRes.post_view);

  // Delta only follows beta, so it should not see an alpha ap_id
  let searchDelta = await searchPost(delta, postRes.post_view.post);
  expect(searchDelta.posts[0]).toBeUndefined();

  // Epsilon has alpha blocked, it should not see the alpha post
  let searchEpsilon = await searchPost(epsilon, postRes.post_view.post);
  expect(searchEpsilon.posts[0]).toBeUndefined();
});

test('Create a post in a non-existent community', async () => {
  let postRes = await createPost(alpha, -2);
  expect(postRes).toStrictEqual({ error: 'couldnt_create_post' });
});

test('Unlike a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].community.id);
  await delay();
  let unlike = await likePost(alpha, 0, postRes.post_view.post);
  expect(unlike.post_view.counts.score).toBe(0);
  await delay();

  // Try to unlike it again, make sure it stays at 0
  let unlike2 = await likePost(alpha, 0, postRes.post_view.post);
  expect(unlike2.post_view.counts.score).toBe(0);
  await longDelay();

  // Make sure that post is unliked on beta
  let searchBeta = await searchPost(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];

  expect(betaPost).toBeDefined();
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.counts.score).toBe(0);
  assertPostFederation(betaPost, postRes.post_view);
});

test('Update a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].community.id);
  await delay();

  let updatedName = 'A jest test federated post, updated';
  let updatedPost = await editPost(alpha, postRes.post_view.post);
  expect(updatedPost.post_view.post.name).toBe(updatedName);
  expect(updatedPost.post_view.community.local).toBe(false);
  expect(updatedPost.post_view.creator.local).toBe(true);
  await delay();

  // Make sure that post is updated on beta
  let searchBeta = await searchPost(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.post.name).toBe(updatedName);
  assertPostFederation(betaPost, updatedPost.post_view);
  await delay();

  // Make sure lemmy beta cannot update the post
  let updatedPostBeta = await editPost(beta, betaPost.post);
  expect(updatedPostBeta).toStrictEqual({ error: 'no_post_edit_allowed' });
});

test('Sticky a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].community.id);
  await delay();

  let stickiedPostRes = await stickyPost(alpha, true, postRes.post_view.post);
  expect(stickiedPostRes.post_view.post.stickied).toBe(true);
  await delay();

  // Make sure that post is stickied on beta
  let searchBeta = await searchPost(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.post.stickied).toBe(true);

  // Unsticky a post
  let unstickiedPost = await stickyPost(alpha, false, postRes.post_view.post);
  expect(unstickiedPost.post_view.post.stickied).toBe(false);
  await delay();

  // Make sure that post is unstickied on beta
  let searchBeta2 = await searchPost(beta, postRes.post_view.post);
  let betaPost2 = searchBeta2.posts[0];
  expect(betaPost2.community.local).toBe(true);
  expect(betaPost2.creator.local).toBe(false);
  expect(betaPost2.post.stickied).toBe(false);

  // Make sure that gamma cannot sticky the post on beta
  let searchGamma = await searchPost(gamma, postRes.post_view.post);
  let gammaPost = searchGamma.posts[0];
  let gammaTrySticky = await stickyPost(gamma, true, gammaPost.post);
  await delay();
  let searchBeta3 = await searchPost(beta, postRes.post_view.post);
  let betaPost3 = searchBeta3.posts[0];
  expect(gammaTrySticky.post_view.post.stickied).toBe(true);
  expect(betaPost3.post.stickied).toBe(false);
});

test('Lock a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  await delay();
  let postRes = await createPost(alpha, search.communities[0].community.id);
  await delay();

  // Lock the post
  let lockedPostRes = await lockPost(alpha, true, postRes.post_view.post);
  expect(lockedPostRes.post_view.post.locked).toBe(true);
  await longDelay();

  // Make sure that post is locked on beta
  let searchBeta = await searchPostLocal(beta, postRes.post_view.post);
  let betaPost1 = searchBeta.posts[0];
  expect(betaPost1.post.locked).toBe(true);
  await delay();

  // Try to make a new comment there, on alpha
  let comment: any = await createComment(alpha, postRes.post_view.post.id);
  expect(comment['error']).toBe('locked');
  await delay();

  // Unlock a post
  let unlockedPost = await lockPost(alpha, false, postRes.post_view.post);
  expect(unlockedPost.post_view.post.locked).toBe(false);
  await delay();

  // Make sure that post is unlocked on beta
  let searchBeta2 = await searchPost(beta, postRes.post_view.post);
  let betaPost2 = searchBeta2.posts[0];
  expect(betaPost2.community.local).toBe(true);
  expect(betaPost2.creator.local).toBe(false);
  expect(betaPost2.post.locked).toBe(false);

  // Try to create a new comment, on beta
  let commentBeta = await createComment(beta, betaPost2.post.id);
  expect(commentBeta).toBeDefined();
});

test('Delete a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].community.id);
  await delay();

  let deletedPost = await deletePost(alpha, true, postRes.post_view.post);
  expect(deletedPost.post_view.post.deleted).toBe(true);
  await delay();

  // Make sure lemmy beta sees post is deleted
  let searchBeta = await searchPost(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];
  // This will be undefined because of the tombstone
  expect(betaPost).toBeUndefined();
  await delay();

  // Undelete
  let undeletedPost = await deletePost(alpha, false, postRes.post_view.post);
  expect(undeletedPost.post_view.post.deleted).toBe(false);
  await delay();

  // Make sure lemmy beta sees post is undeleted
  let searchBeta2 = await searchPost(beta, postRes.post_view.post);
  let betaPost2 = searchBeta2.posts[0];
  expect(betaPost2.post.deleted).toBe(false);
  assertPostFederation(betaPost2, undeletedPost.post_view);

  // Make sure lemmy beta cannot delete the post
  let deletedPostBeta = await deletePost(beta, true, betaPost2.post);
  expect(deletedPostBeta).toStrictEqual({ error: 'no_post_edit_allowed' });
});

test('Remove a post from admin and community on different instance', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].community.id);
  await delay();

  let removedPost = await removePost(alpha, true, postRes.post_view.post);
  expect(removedPost.post_view.post.removed).toBe(true);
  await delay();

  // Make sure lemmy beta sees post is NOT removed
  let searchBeta = await searchPost(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];
  expect(betaPost.post.removed).toBe(false);
  await delay();

  // Undelete
  let undeletedPost = await removePost(alpha, false, postRes.post_view.post);
  expect(undeletedPost.post_view.post.removed).toBe(false);
  await delay();

  // Make sure lemmy beta sees post is undeleted
  let searchBeta2 = await searchPost(beta, postRes.post_view.post);
  let betaPost2 = searchBeta2.posts[0];
  expect(betaPost2.post.removed).toBe(false);
  assertPostFederation(betaPost2, undeletedPost.post_view);
});

test('Remove a post from admin and community on same instance', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].community.id);
  await longDelay();

  // Get the id for beta
  let searchBeta = await searchPost(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];
  await longDelay();

  // The beta admin removes it (the community lives on beta)
  let removePostRes = await removePost(beta, true, betaPost.post);
  expect(removePostRes.post_view.post.removed).toBe(true);
  await longDelay();

  // Make sure lemmy alpha sees post is removed
  let alphaPost = await getPost(alpha, postRes.post_view.post.id);
  expect(alphaPost.post_view.post.removed).toBe(true);
  assertPostFederation(alphaPost.post_view, removePostRes.post_view);
  await longDelay();

  // Undelete
  let undeletedPost = await removePost(beta, false, betaPost.post);
  expect(undeletedPost.post_view.post.removed).toBe(false);
  await longDelay();

  // Make sure lemmy alpha sees post is undeleted
  let alphaPost2 = await getPost(alpha, postRes.post_view.post.id);
  await delay();
  expect(alphaPost2.post_view.post.removed).toBe(false);
  assertPostFederation(alphaPost2.post_view, undeletedPost.post_view);
});

test('Search for a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  await delay();
  let postRes = await createPost(alpha, search.communities[0].community.id);
  await delay();
  let searchBeta = await searchPost(beta, postRes.post_view.post);

  expect(searchBeta.posts[0].post.name).toBeDefined();
});

test('A and G subscribe to B (center) A posts, it gets announced to G', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].community.id);
  await delay();

  let search2 = await searchPost(gamma, postRes.post_view.post);
  expect(search2.posts[0].post.name).toBeDefined();
});

test('Enforce site ban for federated user', async () => {
  let alphaShortname = `@lemmy_alpha@lemmy-alpha:8541`;
  let userSearch = await searchForUser(beta, alphaShortname);
  let alphaUser = userSearch.users[0];
  expect(alphaUser).toBeDefined();
  await delay();

  // ban alpha from beta site
  let banAlpha = await banUserFromSite(beta, alphaUser.user.id, true);
  expect(banAlpha.banned).toBe(true);
  await longDelay();

  // Alpha makes post on beta
  let search = await searchForBetaCommunity(alpha);
  await delay();
  let postRes = await createPost(alpha, search.communities[0].community.id);
  expect(postRes.post_view.post).toBeDefined();
  expect(postRes.post_view.community.local).toBe(false);
  expect(postRes.post_view.creator.local).toBe(true);
  expect(postRes.post_view.counts.score).toBe(1);
  await longDelay();

  // Make sure that post doesn't make it to beta
  let searchBeta = await searchPostLocal(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];
  expect(betaPost).toBeUndefined();
  await delay();

  // Unban alpha
  let unBanAlpha = await banUserFromSite(beta, alphaUser.user.id, false);
  expect(unBanAlpha.banned).toBe(false);
});

test('Enforce community ban for federated user', async () => {
  let alphaShortname = `@lemmy_alpha@lemmy-alpha:8541`;
  let userSearch = await searchForUser(beta, alphaShortname);
  let alphaUser = userSearch.users[0];
  expect(alphaUser).toBeDefined();
  await delay();

  // ban alpha from beta site
  await banUserFromCommunity(beta, alphaUser.user.id, 2, false);
  let banAlpha = await banUserFromCommunity(beta, alphaUser.user.id, 2, true);
  expect(banAlpha.banned).toBe(true);
  await longDelay();

  // Alpha makes post on beta
  let search = await searchForBetaCommunity(alpha);
  await delay();
  let postRes = await createPost(alpha, search.communities[0].community.id);
  expect(postRes.post_view.post).toBeDefined();
  expect(postRes.post_view.community.local).toBe(false);
  expect(postRes.post_view.creator.local).toBe(true);
  expect(postRes.post_view.counts.score).toBe(1);
  await longDelay();

  // Make sure that post doesn't make it to beta community
  let searchBeta = await searchPostLocal(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];
  expect(betaPost).toBeUndefined();

  // Unban alpha
  let unBanAlpha = await banUserFromCommunity(
    beta,
    alphaUser.user.id,
    2,
    false
  );
  expect(unBanAlpha.banned).toBe(false);
});
