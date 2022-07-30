jest.setTimeout(120000);
import {None} from '@sniptt/monads';
import { PostView, CommunityView } from 'lemmy-js-client';
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
  reportPost,
  listPostReports,
  randomString,
  registerUser,
  API,
  getSite,
  unfollows
} from './shared';

let betaCommunity: CommunityView;

beforeAll(async () => {
  await setupLogins();
  betaCommunity = (await resolveBetaCommunity(alpha)).community.unwrap();
  expect(betaCommunity).toBeDefined();
  await unfollows();
});

afterAll(async () => {
  await unfollows();
});

function assertPostFederation(postOne: PostView, postTwo: PostView) {
  expect(postOne.post.ap_id).toBe(postTwo.post.ap_id);
  expect(postOne.post.name).toBe(postTwo.post.name);
  expect(postOne.post.body.unwrapOr("none")).toBe(postTwo.post.body.unwrapOr("none"));
  expect(postOne.post.url.unwrapOr("none")).toBe(postTwo.post.url.unwrapOr("none"));
  expect(postOne.post.nsfw).toBe(postTwo.post.nsfw);
  expect(postOne.post.embed_title.unwrapOr("none")).toBe(postTwo.post.embed_title.unwrapOr("none"));
  expect(postOne.post.embed_description.unwrapOr("none")).toBe(postTwo.post.embed_description.unwrapOr("none"));
  expect(postOne.post.embed_html.unwrapOr("none")).toBe(postTwo.post.embed_html.unwrapOr("none"));
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
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();

  expect(betaPost).toBeDefined();
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.counts.score).toBe(1);
  assertPostFederation(betaPost, postRes.post_view);

  // Delta only follows beta, so it should not see an alpha ap_id
  let deltaPost = (await resolvePost(delta, postRes.post_view.post)).post;
  expect(deltaPost.isNone()).toBe(true)

  // Epsilon has alpha blocked, it should not see the alpha post
  let epsilonPost = (await resolvePost(epsilon, postRes.post_view.post)).post;
  expect(epsilonPost.isNone()).toBe(true);
});

test('Create a post in a non-existent community', async () => {
  let postRes = await createPost(alpha, -2) as any;
  expect(postRes.error).toBe('couldnt_find_community');
});

test('Unlike a post', async () => {
  let postRes = await createPost(alpha, betaCommunity.community.id);
  let unlike = await likePost(alpha, 0, postRes.post_view.post);
  expect(unlike.post_view.counts.score).toBe(0);

  // Try to unlike it again, make sure it stays at 0
  let unlike2 = await likePost(alpha, 0, postRes.post_view.post);
  expect(unlike2.post_view.counts.score).toBe(0);

  // Make sure that post is unliked on beta
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
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
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.post.name).toBe(updatedName);
  assertPostFederation(betaPost, updatedPost.post_view);

  // Make sure lemmy beta cannot update the post
  let updatedPostBeta = await editPost(beta, betaPost.post) as any;
  expect(updatedPostBeta.error).toBe('no_post_edit_allowed');
});

test('Sticky a post', async () => {
  let postRes = await createPost(alpha, betaCommunity.community.id);

  let betaPost1 = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
  let stickiedPostRes = await stickyPost(beta, true, betaPost1.post);
  expect(stickiedPostRes.post_view.post.stickied).toBe(true);

  // Make sure that post is stickied on beta
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.post.stickied).toBe(true);

  // Unsticky a post
  let unstickiedPost = await stickyPost(beta, false, betaPost1.post);
  expect(unstickiedPost.post_view.post.stickied).toBe(false);

  // Make sure that post is unstickied on beta
  let betaPost2 = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
  expect(betaPost2.community.local).toBe(true);
  expect(betaPost2.creator.local).toBe(false);
  expect(betaPost2.post.stickied).toBe(false);

  // Make sure that gamma cannot sticky the post on beta
  let gammaPost = (await resolvePost(gamma, postRes.post_view.post)).post.unwrap();
  let gammaTrySticky = await stickyPost(gamma, true, gammaPost.post);
  let betaPost3 = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
  expect(gammaTrySticky.post_view.post.stickied).toBe(true);
  expect(betaPost3.post.stickied).toBe(false);
});

test('Lock a post', async () => {
  await followCommunity(alpha, true, betaCommunity.community.id);
  let postRes = await createPost(alpha, betaCommunity.community.id);

  // Lock the post
  let betaPost1 = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
  let lockedPostRes = await lockPost(beta, true, betaPost1.post);
  expect(lockedPostRes.post_view.post.locked).toBe(true);

  // Make sure that post is locked on alpha
  let searchAlpha = await searchPostLocal(alpha, postRes.post_view.post);
  let alphaPost1 = searchAlpha.posts[0];
  expect(alphaPost1.post.locked).toBe(true);

  // Try to make a new comment there, on alpha
  let comment: any = await createComment(alpha, alphaPost1.post.id, None);
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
  let commentAlpha = await createComment(alpha, alphaPost1.post.id, None);
  expect(commentAlpha).toBeDefined();
});

test('Delete a post', async () => {
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();

  let deletedPost = await deletePost(alpha, true, postRes.post_view.post);
  expect(deletedPost.post_view.post.deleted).toBe(true);
  expect(deletedPost.post_view.post.name).toBe(postRes.post_view.post.name);

  // Make sure lemmy beta sees post is deleted
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post;
  // This will be undefined because of the tombstone
  expect(betaPost.isNone()).toBe(true);

  // Undelete
  let undeletedPost = await deletePost(alpha, false, postRes.post_view.post);
  expect(undeletedPost.post_view.post.deleted).toBe(false);

  // Make sure lemmy beta sees post is undeleted
  let betaPost2 = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
  expect(betaPost2.post.deleted).toBe(false);
  assertPostFederation(betaPost2, undeletedPost.post_view);

  // Make sure lemmy beta cannot delete the post
  let deletedPostBeta = await deletePost(beta, true, betaPost2.post) as any;
  expect(deletedPostBeta.error).toStrictEqual('no_post_edit_allowed');
});

test('Remove a post from admin and community on different instance', async () => {
  let postRes = await createPost(gamma, betaCommunity.community.id);

  let alphaPost = (await resolvePost(alpha, postRes.post_view.post)).post.unwrap();
  let removedPost = await removePost(alpha, true, alphaPost.post);
  expect(removedPost.post_view.post.removed).toBe(true);
  expect(removedPost.post_view.post.name).toBe(postRes.post_view.post.name);

  // Make sure lemmy beta sees post is NOT removed
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
  expect(betaPost.post.removed).toBe(false);

  // Undelete
  let undeletedPost = await removePost(alpha, false, alphaPost.post);
  expect(undeletedPost.post_view.post.removed).toBe(false);

  // Make sure lemmy beta sees post is undeleted
  let betaPost2 = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
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
  // let alphaPost = await getPost(alpha, postRes.post_view.post.id);
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

  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post.unwrap();
  expect(betaPost.post.name).toBeDefined();
});

test('Enforce site ban for federated user', async () => {
  // create a test user
  let alphaUserJwt = await registerUser(alpha);
  expect(alphaUserJwt).toBeDefined();
  let alpha_user: API = {
      client: alpha.client,
      auth: alphaUserJwt.jwt,
  };
  let alphaUserActorId = (await getSite(alpha_user)).my_user.unwrap().local_user_view.person.actor_id;
  expect(alphaUserActorId).toBeDefined();
  let alphaPerson = (await resolvePerson(alpha_user, alphaUserActorId)).person.unwrap();
  expect(alphaPerson).toBeDefined();

  // alpha makes post in beta community, it federates to beta instance
  let postRes1 = await createPost(alpha_user, betaCommunity.community.id);
  let searchBeta1 = await searchPostLocal(beta, postRes1.post_view.post);
  expect(searchBeta1.posts[0]).toBeDefined();

  // ban alpha from its instance
  let banAlpha = await banPersonFromSite(alpha, alphaPerson.person.id, true, true);
  expect(banAlpha.banned).toBe(true);

  // alpha ban should be federated to beta
  let alphaUserOnBeta1 = await resolvePerson(beta, alphaUserActorId);
  expect(alphaUserOnBeta1.person.unwrap().person.banned).toBe(true);

  // existing alpha post should be removed on beta
  let searchBeta2 = await searchPostLocal(beta, postRes1.post_view.post);
  expect(searchBeta2.posts[0]).toBeUndefined();

  // Unban alpha
  let unBanAlpha = await banPersonFromSite(alpha, alphaPerson.person.id, false, false);
  expect(unBanAlpha.banned).toBe(false);

  // alpha makes new post in beta community, it federates
  let postRes2 = await createPost(alpha_user, betaCommunity.community.id);
  let searchBeta3 = await searchPostLocal(beta, postRes2.post_view.post);
  expect(searchBeta3.posts[0]).toBeDefined();

  let alphaUserOnBeta2 = await resolvePerson(beta, alphaUserActorId)
  expect(alphaUserOnBeta2.person.unwrap().person.banned).toBe(false);
});

test('Enforce community ban for federated user', async () => {
  let alphaShortname = `@lemmy_alpha@lemmy-alpha:8541`;
  let alphaPerson = (await resolvePerson(beta, alphaShortname)).person.unwrap();
  expect(alphaPerson).toBeDefined();

  // make a post in beta, it goes through
  let postRes1 = await createPost(alpha, betaCommunity.community.id);
  let searchBeta1 = await searchPostLocal(beta, postRes1.post_view.post);
  expect(searchBeta1.posts[0]).toBeDefined();

  // ban alpha from beta community
  let banAlpha = await banPersonFromCommunity(beta, alphaPerson.person.id, 2, true, true);
  expect(banAlpha.banned).toBe(true);

  // ensure that the post by alpha got removed
  let searchAlpha1 = await searchPostLocal(alpha, postRes1.post_view.post);
  expect(searchAlpha1.posts[0]).toBeUndefined();

  // Alpha tries to make post on beta, but it fails because of ban
  let postRes2 = await createPost(alpha, betaCommunity.community.id);
  expect(postRes2.post_view).toBeUndefined();

  // Unban alpha
  let unBanAlpha = await banPersonFromCommunity(
    beta,
    alphaPerson.person.id,
    2,
    false,
    false
  );
  expect(unBanAlpha.banned).toBe(false);
  let postRes3 = await createPost(alpha, betaCommunity.community.id);
  expect(postRes3.post_view.post).toBeDefined();
  expect(postRes3.post_view.community.local).toBe(false);
  expect(postRes3.post_view.creator.local).toBe(true);
  expect(postRes3.post_view.counts.score).toBe(1);

  // Make sure that post makes it to beta community
  let searchBeta2 = await searchPostLocal(beta, postRes3.post_view.post);
  expect(searchBeta2.posts[0]).toBeDefined();
});


test('A and G subscribe to B (center) A posts, it gets announced to G', async () => {
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();

  let betaPost = (await resolvePost(gamma, postRes.post_view.post)).post.unwrap();
  expect(betaPost.post.name).toBeDefined();
});

test('Report a post', async () => {
  let betaCommunity = (await resolveBetaCommunity(beta)).community.unwrap();
  let postRes = await createPost(beta, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();

  let alphaPost = (await resolvePost(alpha, postRes.post_view.post)).post.unwrap();
  let alphaReport = (await reportPost(alpha, alphaPost.post.id, randomString(10)))
        .post_report_view.post_report;

  let betaReport = (await listPostReports(beta)).post_reports[0].post_report;
  expect(betaReport).toBeDefined();
  expect(betaReport.resolved).toBe(false);
  expect(betaReport.original_post_name).toBe(alphaReport.original_post_name);
  expect(betaReport.original_post_url.unwrapOr("none")).toBe(alphaReport.original_post_url.unwrapOr("none"));
  expect(betaReport.original_post_body.unwrapOr("none")).toBe(alphaReport.original_post_body.unwrapOr("none"));
  expect(betaReport.reason).toBe(alphaReport.reason);
});
