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
  resolvePost,
  likePost,
  followBeta,
  resolveBetaCommunity,
  createComment,
  deletePost,
  removePost,
  getPost,
  unfollowRemotes,
  resolvePerson,
  banPersonFromSite,
  searchPostLocal,
  followCommunity,
  banPersonFromCommunity,
} from './shared';
import { PostView, CommunityView } from 'lemmy-js-client';

let betaCommunity: CommunityView;

beforeAll(async () => {
  await setupLogins();
  betaCommunity = (await resolveBetaCommunity(alpha)).community;
  expect(betaCommunity).toBeDefined();
  await unfollows();
});

afterAll(async () => {
  await unfollows();
});

async function unfollows() {
  await unfollowRemotes(alpha);
  await unfollowRemotes(gamma);
  await unfollowRemotes(delta);
  await unfollowRemotes(epsilon);
}

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
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();
  expect(postRes.post_view.community.local).toBe(false);
  expect(postRes.post_view.creator.local).toBe(true);
  expect(postRes.post_view.counts.score).toBe(1);

  // Make sure that post is liked on beta
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post;

  expect(betaPost).toBeDefined();
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.counts.score).toBe(1);
  assertPostFederation(betaPost, postRes.post_view);

  // Delta only follows beta, so it should not see an alpha ap_id
  let deltaPost = (await resolvePost(delta, postRes.post_view.post)).post;
  expect(deltaPost).toBeUndefined();

  // Epsilon has alpha blocked, it should not see the alpha post
  let epsilonPost = (await resolvePost(epsilon, postRes.post_view.post)).post;
  expect(epsilonPost).toBeUndefined();
});

test('Create a post in a non-existent community', async () => {
  let postRes = await createPost(alpha, -2);
  expect(postRes).toStrictEqual({ error: 'couldnt_create_post' });
});

test('Unlike a post', async () => {
  let postRes = await createPost(alpha, betaCommunity.community.id);
  let unlike = await likePost(alpha, 0, postRes.post_view.post);
  expect(unlike.post_view.counts.score).toBe(0);

  // Try to unlike it again, make sure it stays at 0
  let unlike2 = await likePost(alpha, 0, postRes.post_view.post);
  expect(unlike2.post_view.counts.score).toBe(0);

  // Make sure that post is unliked on beta
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(betaPost).toBeDefined();
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.counts.score).toBe(0);
  assertPostFederation(betaPost, postRes.post_view);
});

test('Update a post', async () => {
  let postRes = await createPost(alpha, betaCommunity.community.id);

  let updatedName = 'A jest test federated post, updated';
  let updatedPost = await editPost(alpha, postRes.post_view.post);
  expect(updatedPost.post_view.post.name).toBe(updatedName);
  expect(updatedPost.post_view.community.local).toBe(false);
  expect(updatedPost.post_view.creator.local).toBe(true);

  // Make sure that post is updated on beta
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.post.name).toBe(updatedName);
  assertPostFederation(betaPost, updatedPost.post_view);

  // Make sure lemmy beta cannot update the post
  let updatedPostBeta = await editPost(beta, betaPost.post);
  expect(updatedPostBeta).toStrictEqual({ error: 'no_post_edit_allowed' });
});

test('Sticky a post', async () => {
  let postRes = await createPost(alpha, betaCommunity.community.id);

  let stickiedPostRes = await stickyPost(alpha, true, postRes.post_view.post);
  expect(stickiedPostRes.post_view.post.stickied).toBe(true);

  // Make sure that post is stickied on beta
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.post.stickied).toBe(true);

  // Unsticky a post
  let unstickiedPost = await stickyPost(alpha, false, postRes.post_view.post);
  expect(unstickiedPost.post_view.post.stickied).toBe(false);

  // Make sure that post is unstickied on beta
  let betaPost2 = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(betaPost2.community.local).toBe(true);
  expect(betaPost2.creator.local).toBe(false);
  expect(betaPost2.post.stickied).toBe(false);

  // Make sure that gamma cannot sticky the post on beta
  let gammaPost = (await resolvePost(gamma, postRes.post_view.post)).post;
  let gammaTrySticky = await stickyPost(gamma, true, gammaPost.post);
  let betaPost3 = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(gammaTrySticky.post_view.post.stickied).toBe(true);
  expect(betaPost3.post.stickied).toBe(false);
});

test('Lock a post', async () => {
  await followCommunity(alpha, true, betaCommunity.community.id);
  let postRes = await createPost(alpha, betaCommunity.community.id);

  // Lock the post
  let betaPost1 = (await resolvePost(beta, postRes.post_view.post)).post;
  let lockedPostRes = await lockPost(beta, true, betaPost1.post);
  expect(lockedPostRes.post_view.post.locked).toBe(true);

  // Make sure that post is locked on alpha
  let searchAlpha = await searchPostLocal(alpha, postRes.post_view.post);
  let alphaPost1 = searchAlpha.posts[0];
  expect(alphaPost1.post.locked).toBe(true);

  // Try to make a new comment there, on alpha
  let comment: any = await createComment(alpha, alphaPost1.post.id);
  expect(comment['error']).toBe('locked');

  // Unlock a post
  let unlockedPost = await lockPost(beta, false, betaPost1.post);
  expect(unlockedPost.post_view.post.locked).toBe(false);

  // Make sure that post is unlocked on alpha
  let searchAlpha2 = await searchPostLocal(alpha, postRes.post_view.post);
  let alphaPost2 = searchAlpha2.posts[0];
  expect(alphaPost2.community.local).toBe(false);
  expect(alphaPost2.creator.local).toBe(true);
  expect(alphaPost2.post.locked).toBe(false);

  // Try to create a new comment, on alpha
  let commentAlpha = await createComment(alpha, alphaPost1.post.id);
  expect(commentAlpha).toBeDefined();
});

test('Delete a post', async () => {
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();

  let deletedPost = await deletePost(alpha, true, postRes.post_view.post);
  expect(deletedPost.post_view.post.deleted).toBe(true);
  expect(deletedPost.post_view.post.name).toBe("");

  // Make sure lemmy beta sees post is deleted
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post;
  // This will be undefined because of the tombstone
  expect(betaPost).toBeUndefined();

  // Undelete
  let undeletedPost = await deletePost(alpha, false, postRes.post_view.post);
  expect(undeletedPost.post_view.post.deleted).toBe(false);

  // Make sure lemmy beta sees post is undeleted
  let betaPost2 = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(betaPost2.post.deleted).toBe(false);
  assertPostFederation(betaPost2, undeletedPost.post_view);

  // Make sure lemmy beta cannot delete the post
  let deletedPostBeta = await deletePost(beta, true, betaPost2.post);
  expect(deletedPostBeta).toStrictEqual({ error: 'no_post_edit_allowed' });
});

test('Remove a post from admin and community on different instance', async () => {
  let postRes = await createPost(alpha, betaCommunity.community.id);

  let removedPost = await removePost(alpha, true, postRes.post_view.post);
  expect(removedPost.post_view.post.removed).toBe(true);
  expect(removedPost.post_view.post.name).toBe("");

  // Make sure lemmy beta sees post is NOT removed
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(betaPost.post.removed).toBe(false);

  // Undelete
  let undeletedPost = await removePost(alpha, false, postRes.post_view.post);
  expect(undeletedPost.post_view.post.removed).toBe(false);

  // Make sure lemmy beta sees post is undeleted
  let betaPost2 = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(betaPost2.post.removed).toBe(false);
  assertPostFederation(betaPost2, undeletedPost.post_view);
});

test('Remove a post from admin and community on same instance', async () => {
  await followBeta(alpha);
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();

  // Get the id for beta
  let searchBeta = await searchPostLocal(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];
  expect(betaPost).toBeDefined();

  // The beta admin removes it (the community lives on beta)
  let removePostRes = await removePost(beta, true, betaPost.post);
  expect(removePostRes.post_view.post.removed).toBe(true);

  // Make sure lemmy alpha sees post is removed
  let alphaPost = await getPost(alpha, postRes.post_view.post.id);
  // expect(alphaPost.post_view.post.removed).toBe(true); // TODO this shouldn't be commented
  // assertPostFederation(alphaPost.post_view, removePostRes.post_view);

  // Undelete
  let undeletedPost = await removePost(beta, false, betaPost.post);
  expect(undeletedPost.post_view.post.removed).toBe(false);

  // Make sure lemmy alpha sees post is undeleted
  let alphaPost2 = await getPost(alpha, postRes.post_view.post.id);
  expect(alphaPost2.post_view.post.removed).toBe(false);
  assertPostFederation(alphaPost2.post_view, undeletedPost.post_view);
  await unfollowRemotes(alpha);
});

test('Search for a post', async () => {
  await unfollowRemotes(alpha);
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();

  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post;

  expect(betaPost.post.name).toBeDefined();
});

test('A and G subscribe to B (center) A posts, it gets announced to G', async () => {
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();

  let betaPost = (await resolvePost(gamma, postRes.post_view.post)).post;
  expect(betaPost.post.name).toBeDefined();
});

test('Enforce site ban for federated user', async () => {
  let alphaShortname = `@lemmy_alpha@lemmy-alpha:8541`;
  let alphaPerson = (await resolvePerson(beta, alphaShortname)).person;
  expect(alphaPerson).toBeDefined();

  // ban alpha from beta site
  let banAlpha = await banPersonFromSite(beta, alphaPerson.person.id, true);
  expect(banAlpha.banned).toBe(true);

  // Alpha makes post on beta
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();
  expect(postRes.post_view.community.local).toBe(false);
  expect(postRes.post_view.creator.local).toBe(true);
  expect(postRes.post_view.counts.score).toBe(1);

  // Make sure that post doesn't make it to beta
  let searchBeta = await searchPostLocal(beta, postRes.post_view.post);
  let betaPost = searchBeta.posts[0];
  expect(betaPost).toBeUndefined();

  // Unban alpha
  let unBanAlpha = await banPersonFromSite(beta, alphaPerson.person.id, false);
  expect(unBanAlpha.banned).toBe(false);
});

test('Enforce community ban for federated user', async () => {
  let alphaShortname = `@lemmy_alpha@lemmy-alpha:8541`;
  let alphaPerson = (await resolvePerson(beta, alphaShortname)).person;
  expect(alphaPerson).toBeDefined();

  // ban alpha from beta site
  await banPersonFromCommunity(beta, alphaPerson.person.id, 2, false);
  let banAlpha = await banPersonFromCommunity(beta, alphaPerson.person.id, 2, true);
  expect(banAlpha.banned).toBe(true);

  // Alpha tries to make post on beta, but it fails because of ban
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view).toBeUndefined();

  // Unban alpha
  let unBanAlpha = await banPersonFromCommunity(
    beta,
    alphaPerson.person.id,
    2,
    false
  );
  expect(unBanAlpha.banned).toBe(false);
  let postRes2 = await createPost(alpha, betaCommunity.community.id);
  expect(postRes2.post_view.post).toBeDefined();
  expect(postRes2.post_view.community.local).toBe(false);
  expect(postRes2.post_view.creator.local).toBe(true);
  expect(postRes2.post_view.counts.score).toBe(1);

  // Make sure that post makes it to beta community
  let searchBeta = await searchPostLocal(beta, postRes2.post_view.post);
  let betaPost = searchBeta.posts[0];
  expect(betaPost).toBeDefined();
});
