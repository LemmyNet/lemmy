import {
  alpha,
  beta,
  gamma,
  setupLogins,
  createPost,
  updatePost,
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
} from './shared';

beforeAll(async () => {
  await setupLogins();
  await followBeta(alpha);
  await followBeta(gamma);
});

afterAll(async () => {
  await unfollowRemotes(alpha);
  await unfollowRemotes(gamma);
});

test('Create a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].id);
  expect(postRes.post).toBeDefined();
  expect(postRes.post.community_local).toBe(false);
  expect(postRes.post.creator_local).toBe(true);
  expect(postRes.post.score).toBe(1);

  // Make sure that post is liked on beta
  let searchBeta = await searchPost(beta, postRes.post);
  let betaPost = searchBeta.posts[0];

  expect(betaPost).toBeDefined();
  expect(betaPost.community_local).toBe(true);
  expect(betaPost.creator_local).toBe(false);
  expect(betaPost.score).toBe(1);
});

test('Unlike a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].id);
  let unlike = await likePost(alpha, 0, postRes.post);
  expect(unlike.post.score).toBe(0);

  // Make sure that post is unliked on beta
  let searchBeta = await searchPost(beta, postRes.post);
  let betaPost = searchBeta.posts[0];

  expect(betaPost).toBeDefined();
  expect(betaPost.community_local).toBe(true);
  expect(betaPost.creator_local).toBe(false);
  expect(betaPost.score).toBe(0);
});

test('Update a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].id);

  let updatedPost = await updatePost(alpha, postRes.post);
  expect(updatedPost.post.name).toBe('A jest test federated post, updated');
  expect(updatedPost.post.community_local).toBe(false);
  expect(updatedPost.post.creator_local).toBe(true);
});

test('Sticky a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].id);

  let stickiedPostRes = await stickyPost(alpha, true, postRes.post);
  expect(stickiedPostRes.post.stickied).toBe(true);

  // Make sure that post is stickied on beta
  let searchBeta = await searchPost(beta, postRes.post);
  let betaPost = searchBeta.posts[0];
  expect(betaPost.community_local).toBe(true);
  expect(betaPost.creator_local).toBe(false);
  expect(betaPost.stickied).toBe(true);

  // Unsticky a post
  let unstickiedPost = await stickyPost(alpha, false, postRes.post);
  expect(unstickiedPost.post.stickied).toBe(false);

  // Make sure that post is unstickied on beta
  let searchBeta2 = await searchPost(beta, postRes.post);
  let betaPost2 = searchBeta2.posts[0];
  expect(betaPost2.community_local).toBe(true);
  expect(betaPost2.creator_local).toBe(false);
  expect(betaPost2.stickied).toBe(false);
});

test('Lock a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].id);

  let lockedPostRes = await lockPost(alpha, true, postRes.post);
  expect(lockedPostRes.post.locked).toBe(true);

  // Make sure that post is locked on beta
  let searchBeta = await searchPost(beta, postRes.post);
  let betaPost = searchBeta.posts[0];
  expect(betaPost.community_local).toBe(true);
  expect(betaPost.creator_local).toBe(false);
  expect(betaPost.locked).toBe(true);

  // Try to make a new comment there, on alpha
  let comment = await createComment(alpha, postRes.post.id);
  expect(comment['error']).toBe('locked');

  // Try to create a new comment, on beta
  let commentBeta = await createComment(beta, betaPost.id);
  expect(commentBeta['error']).toBe('locked');

  // Unlock a post
  let unlockedPost = await lockPost(alpha, false, postRes.post);
  expect(unlockedPost.post.locked).toBe(false);

  // Make sure that post is unlocked on beta
  let searchBeta2 = await searchPost(beta, postRes.post);
  let betaPost2 = searchBeta2.posts[0];
  expect(betaPost2.community_local).toBe(true);
  expect(betaPost2.creator_local).toBe(false);
  expect(betaPost2.locked).toBe(false);
});

test('Delete a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].id);

  let deletedPost = await deletePost(alpha, true, postRes.post);
  expect(deletedPost.post.deleted).toBe(true);

  // Make sure lemmy beta sees post is deleted
  let createFakeBetaPostToGetId = (await createPost(beta, 2)).post.id - 1;
  let betaPost = await getPost(beta, createFakeBetaPostToGetId);
  expect(betaPost.post.deleted).toBe(true);

  // Undelete
  let undeletedPost = await deletePost(alpha, false, postRes.post);
  expect(undeletedPost.post.deleted).toBe(false);

  // Make sure lemmy beta sees post is undeleted
  let betaPost2 = await getPost(beta, createFakeBetaPostToGetId);
  expect(betaPost2.post.deleted).toBe(false);
});

test('Remove a post from admin and community on different instance', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].id);

  let removedPost = await removePost(alpha, true, postRes.post);
  expect(removedPost.post.removed).toBe(true);

  // Make sure lemmy beta sees post is NOT removed
  let createFakeBetaPostToGetId = (await createPost(beta, 2)).post.id - 1;
  let betaPost = await getPost(beta, createFakeBetaPostToGetId);
  expect(betaPost.post.removed).toBe(false);

  // Undelete
  let undeletedPost = await removePost(alpha, false, postRes.post);
  expect(undeletedPost.post.removed).toBe(false);

  // Make sure lemmy beta sees post is undeleted
  let betaPost2 = await getPost(beta, createFakeBetaPostToGetId);
  expect(betaPost2.post.removed).toBe(false);
});

test('Remove a post from admin and community on same instance', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].id);

  // Get the id for beta
  let createFakeBetaPostToGetId = (await createPost(beta, 2)).post.id - 1;
  let betaPost = await getPost(beta, createFakeBetaPostToGetId);

  // The beta admin removes it (the community lives on beta)
  let removePostRes = await removePost(beta, true, betaPost.post);
  expect(removePostRes.post.removed).toBe(true);

  // Make sure lemmy alpha sees post is removed
  let alphaPost = await getPost(alpha, postRes.post.id);
  expect(alphaPost.post.removed).toBe(true);

  // Undelete
  let undeletedPost = await removePost(beta, false, betaPost.post);
  expect(undeletedPost.post.removed).toBe(false);

  // Make sure lemmy alpha sees post is undeleted
  let alphaPost2 = await getPost(alpha, postRes.post.id);
  expect(alphaPost2.post.removed).toBe(false);
});

test('Search for a post', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].id);
  let searchBeta = await searchPost(beta, postRes.post);

  expect(searchBeta.posts[0].name).toBeDefined();
});

test('A and G subscribe to B (center) A posts, it gets announced to G', async () => {
  let search = await searchForBetaCommunity(alpha);
  let postRes = await createPost(alpha, search.communities[0].id);

  let search2 = await searchPost(gamma, postRes.post);
  expect(search2.posts[0].name).toBeDefined();
});
