jest.setTimeout(120000);

import { CommunityView } from "lemmy-js-client/dist/types/CommunityView";
import {
  alpha,
  beta,
  gamma,
  delta,
  epsilon,
  setupLogins,
  createPost,
  editPost,
  featurePost,
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
  getSite,
  unfollows,
  resolveCommunity,
  waitUntil,
  waitForPost,
  alphaUrl,
} from "./shared";
import { PostView } from "lemmy-js-client/dist/types/PostView";
import { CreatePost } from "lemmy-js-client/dist/types/CreatePost";
import { LemmyHttp } from "lemmy-js-client";

let betaCommunity: CommunityView | undefined;

beforeAll(async () => {
  await setupLogins();
  betaCommunity = (await resolveBetaCommunity(alpha)).community;
  expect(betaCommunity).toBeDefined();
  await unfollows();
});

afterAll(async () => {
  await unfollows();
});

function assertPostFederation(postOne?: PostView, postTwo?: PostView) {
  expect(postOne?.post.ap_id).toBe(postTwo?.post.ap_id);
  expect(postOne?.post.name).toBe(postTwo?.post.name);
  expect(postOne?.post.body).toBe(postTwo?.post.body);
  // TODO url clears arent working
  // expect(postOne?.post.url).toBe(postTwo?.post.url);
  expect(postOne?.post.nsfw).toBe(postTwo?.post.nsfw);
  expect(postOne?.post.embed_title).toBe(postTwo?.post.embed_title);
  expect(postOne?.post.embed_description).toBe(postTwo?.post.embed_description);
  expect(postOne?.post.embed_video_url).toBe(postTwo?.post.embed_video_url);
  expect(postOne?.post.published).toBe(postTwo?.post.published);
  expect(postOne?.community.actor_id).toBe(postTwo?.community.actor_id);
  expect(postOne?.post.locked).toBe(postTwo?.post.locked);
  expect(postOne?.post.removed).toBe(postTwo?.post.removed);
  expect(postOne?.post.deleted).toBe(postTwo?.post.deleted);
}

test("Create a post", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }

  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();
  expect(postRes.post_view.community.local).toBe(false);
  expect(postRes.post_view.creator.local).toBe(true);
  expect(postRes.post_view.counts.score).toBe(1);

  // Make sure that post is liked on beta
  const betaPost = await waitForPost(
    beta,
    postRes.post_view.post,
    res => res?.counts.score === 1,
  );

  expect(betaPost).toBeDefined();
  expect(betaPost?.community.local).toBe(true);
  expect(betaPost?.creator.local).toBe(false);
  expect(betaPost?.counts.score).toBe(1);
  assertPostFederation(betaPost, postRes.post_view);

  // Delta only follows beta, so it should not see an alpha ap_id
  await expect(resolvePost(delta, postRes.post_view.post)).rejects.toBe(
    "couldnt_find_object",
  );

  // Epsilon has alpha blocked, it should not see the alpha post
  await expect(resolvePost(epsilon, postRes.post_view.post)).rejects.toBe(
    "couldnt_find_object",
  );
});

test("Create a post in a non-existent community", async () => {
  await expect(createPost(alpha, -2)).rejects.toBe("couldnt_find_community");
});

test("Unlike a post", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let postRes = await createPost(alpha, betaCommunity.community.id);
  let unlike = await likePost(alpha, 0, postRes.post_view.post);
  expect(unlike.post_view.counts.score).toBe(0);

  // Try to unlike it again, make sure it stays at 0
  let unlike2 = await likePost(alpha, 0, postRes.post_view.post);
  expect(unlike2.post_view.counts.score).toBe(0);

  // Make sure that post is unliked on beta
  const betaPost = await waitForPost(
    beta,
    postRes.post_view.post,
    post => post?.counts.score === 0,
  );

  expect(betaPost).toBeDefined();
  expect(betaPost?.community.local).toBe(true);
  expect(betaPost?.creator.local).toBe(false);
  expect(betaPost?.counts.score).toBe(0);
  assertPostFederation(betaPost, postRes.post_view);
});

test("Update a post", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let postRes = await createPost(alpha, betaCommunity.community.id);
  await waitForPost(beta, postRes.post_view.post);

  let updatedName = "A jest test federated post, updated";
  let updatedPost = await editPost(alpha, postRes.post_view.post);
  expect(updatedPost.post_view.post.name).toBe(updatedName);
  expect(updatedPost.post_view.community.local).toBe(false);
  expect(updatedPost.post_view.creator.local).toBe(true);

  // Make sure that post is updated on beta
  let betaPost = await waitForPost(beta, updatedPost.post_view.post);
  expect(betaPost.community.local).toBe(true);
  expect(betaPost.creator.local).toBe(false);
  expect(betaPost.post.name).toBe(updatedName);
  assertPostFederation(betaPost, updatedPost.post_view);

  // Make sure lemmy beta cannot update the post
  await expect(editPost(beta, betaPost.post)).rejects.toBe(
    "no_post_edit_allowed",
  );
});

test("Sticky a post", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let postRes = await createPost(alpha, betaCommunity.community.id);

  let betaPost1 = await waitForPost(beta, postRes.post_view.post);
  if (!betaPost1) {
    throw "Missing beta post1";
  }
  let stickiedPostRes = await featurePost(beta, true, betaPost1.post);
  expect(stickiedPostRes.post_view.post.featured_community).toBe(true);

  // Make sure that post is stickied on beta
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(betaPost?.community.local).toBe(true);
  expect(betaPost?.creator.local).toBe(false);
  expect(betaPost?.post.featured_community).toBe(true);

  // Unsticky a post
  let unstickiedPost = await featurePost(beta, false, betaPost1.post);
  expect(unstickiedPost.post_view.post.featured_community).toBe(false);

  // Make sure that post is unstickied on beta
  let betaPost2 = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(betaPost2?.community.local).toBe(true);
  expect(betaPost2?.creator.local).toBe(false);
  expect(betaPost2?.post.featured_community).toBe(false);

  // Make sure that gamma cannot sticky the post on beta
  let gammaPost = (await resolvePost(gamma, postRes.post_view.post)).post;
  if (!gammaPost) {
    throw "Missing gamma post";
  }
  let gammaTrySticky = await featurePost(gamma, true, gammaPost.post);
  let betaPost3 = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(gammaTrySticky.post_view.post.featured_community).toBe(true);
  expect(betaPost3?.post.featured_community).toBe(false);
});

test("Lock a post", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  await followCommunity(alpha, true, betaCommunity.community.id);
  await waitUntil(
    () => resolveBetaCommunity(alpha),
    c => c.community?.subscribed === "Subscribed",
  );

  let postRes = await createPost(alpha, betaCommunity.community.id);
  let betaPost1 = await waitForPost(beta, postRes.post_view.post);
  // Lock the post
  let lockedPostRes = await lockPost(beta, true, betaPost1.post);
  expect(lockedPostRes.post_view.post.locked).toBe(true);

  // Make sure that post is locked on alpha
  let alphaPost1 = await waitForPost(
    alpha,
    postRes.post_view.post,
    post => !!post && post.post.locked,
  );

  // Try to make a new comment there, on alpha
  await expect(createComment(alpha, alphaPost1.post.id)).rejects.toBe("locked");

  // Unlock a post
  let unlockedPost = await lockPost(beta, false, betaPost1.post);
  expect(unlockedPost.post_view.post.locked).toBe(false);

  // Make sure that post is unlocked on alpha
  let alphaPost2 = await waitForPost(
    alpha,
    postRes.post_view.post,
    post => !!post && !post.post.locked,
  );
  expect(alphaPost2.community.local).toBe(false);
  expect(alphaPost2.creator.local).toBe(true);
  expect(alphaPost2.post.locked).toBe(false);

  // Try to create a new comment, on alpha
  let commentAlpha = await createComment(alpha, alphaPost1.post.id);
  expect(commentAlpha).toBeDefined();
});

test("Delete a post", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }

  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();
  await waitForPost(beta, postRes.post_view.post);

  let deletedPost = await deletePost(alpha, true, postRes.post_view.post);
  expect(deletedPost.post_view.post.deleted).toBe(true);
  expect(deletedPost.post_view.post.name).toBe(postRes.post_view.post.name);

  // Make sure lemmy beta sees post is deleted
  // This will be undefined because of the tombstone
  await waitForPost(beta, postRes.post_view.post, p => !p || p.post.deleted);

  // Undelete
  let undeletedPost = await deletePost(alpha, false, postRes.post_view.post);

  // Make sure lemmy beta sees post is undeleted
  let betaPost2 = await waitForPost(
    beta,
    postRes.post_view.post,
    p => !!p && !p.post.deleted,
  );

  if (!betaPost2) {
    throw "Missing beta post 2";
  }
  expect(betaPost2.post.deleted).toBe(false);
  assertPostFederation(betaPost2, undeletedPost.post_view);

  // Make sure lemmy beta cannot delete the post
  await expect(deletePost(beta, true, betaPost2.post)).rejects.toBe(
    "no_post_edit_allowed",
  );
});

test("Remove a post from admin and community on different instance", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }

  let gammaCommunity = (
    await resolveCommunity(gamma, betaCommunity.community.actor_id)
  ).community?.community;
  if (!gammaCommunity) {
    throw "Missing gamma community";
  }
  let postRes = await createPost(gamma, gammaCommunity.id);

  let alphaPost = (await resolvePost(alpha, postRes.post_view.post)).post;
  if (!alphaPost) {
    throw "Missing alpha post";
  }
  let removedPost = await removePost(alpha, true, alphaPost.post);
  expect(removedPost.post_view.post.removed).toBe(true);
  expect(removedPost.post_view.post.name).toBe(postRes.post_view.post.name);

  // Make sure lemmy beta sees post is NOT removed
  let betaPost = (await resolvePost(beta, postRes.post_view.post)).post;
  if (!betaPost) {
    throw "Missing beta post";
  }
  expect(betaPost.post.removed).toBe(false);

  // Undelete
  let undeletedPost = await removePost(alpha, false, alphaPost.post);
  expect(undeletedPost.post_view.post.removed).toBe(false);

  // Make sure lemmy beta sees post is undeleted
  let betaPost2 = (await resolvePost(beta, postRes.post_view.post)).post;
  expect(betaPost2?.post.removed).toBe(false);
  assertPostFederation(betaPost2, undeletedPost.post_view);
});

test("Remove a post from admin and community on same instance", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  await followBeta(alpha);
  let gammaCommunity = await resolveCommunity(
    gamma,
    betaCommunity.community.actor_id,
  );
  let postRes = await createPost(gamma, gammaCommunity.community!.community.id);
  expect(postRes.post_view.post).toBeDefined();
  // Get the id for beta
  let betaPost = await waitForPost(beta, postRes.post_view.post);
  expect(betaPost).toBeDefined();

  let alphaPost0 = await waitForPost(alpha, postRes.post_view.post);
  expect(alphaPost0).toBeDefined();

  // The beta admin removes it (the community lives on beta)
  let removePostRes = await removePost(beta, true, betaPost.post);
  expect(removePostRes.post_view.post.removed).toBe(true);

  // Make sure lemmy alpha sees post is removed
  let alphaPost = await waitUntil(
    () => getPost(alpha, alphaPost0.post.id),
    p => p?.post_view.post.removed ?? false,
  );
  expect(alphaPost?.post_view.post.removed).toBe(true);
  assertPostFederation(alphaPost.post_view, removePostRes.post_view);

  // Undelete
  let undeletedPost = await removePost(beta, false, betaPost.post);
  expect(undeletedPost.post_view.post.removed).toBe(false);

  // Make sure lemmy alpha sees post is undeleted
  let alphaPost2 = await waitForPost(
    alpha,
    postRes.post_view.post,
    p => !!p && !p.post.removed,
  );
  expect(alphaPost2.post.removed).toBe(false);
  assertPostFederation(alphaPost2, undeletedPost.post_view);
  await unfollowRemotes(alpha);
});

test("Search for a post", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  await unfollowRemotes(alpha);
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();

  let betaPost = await waitForPost(beta, postRes.post_view.post);
  expect(betaPost?.post.name).toBeDefined();
});

test("Enforce site ban for federated user", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  // create a test user
  let alphaUserJwt = await registerUser(alpha);
  expect(alphaUserJwt).toBeDefined();
  let alpha_user = new LemmyHttp(alphaUrl, {
    headers: { Authorization: `Bearer ${alphaUserJwt.jwt ?? ""}` },
  });
  let alphaUserActorId = (await getSite(alpha_user)).my_user?.local_user_view
    .person.actor_id;
  if (!alphaUserActorId) {
    throw "Missing alpha user actor id";
  }
  expect(alphaUserActorId).toBeDefined();
  let alphaPerson = (await resolvePerson(alpha_user, alphaUserActorId!)).person;
  if (!alphaPerson) {
    throw "Missing alpha person";
  }
  expect(alphaPerson).toBeDefined();

  // alpha makes post in beta community, it federates to beta instance
  let postRes1 = await createPost(alpha_user, betaCommunity.community.id);
  let searchBeta1 = await waitForPost(beta, postRes1.post_view.post);

  // ban alpha from its instance
  let banAlpha = await banPersonFromSite(
    alpha,
    alphaPerson.person.id,
    true,
    true,
  );
  expect(banAlpha.banned).toBe(true);

  // alpha ban should be federated to beta
  let alphaUserOnBeta1 = await waitUntil(
    () => resolvePerson(beta, alphaUserActorId!),
    res => res.person?.person.banned ?? false,
  );
  expect(alphaUserOnBeta1.person?.person.banned).toBe(true);

  // existing alpha post should be removed on beta
  let searchBeta2 = await waitUntil(
    () => getPost(beta, searchBeta1.post.id),
    s => s.post_view.post.removed,
  );

  // Unban alpha
  let unBanAlpha = await banPersonFromSite(
    alpha,
    alphaPerson.person.id,
    false,
    false,
  );
  expect(unBanAlpha.banned).toBe(false);

  // alpha makes new post in beta community, it federates
  let postRes2 = await createPost(alpha_user, betaCommunity.community.id);
  let searchBeta3 = await waitForPost(beta, postRes2.post_view.post);

  let alphaUserOnBeta2 = await resolvePerson(beta, alphaUserActorId!);
  expect(alphaUserOnBeta2.person?.person.banned).toBe(false);
});

test.skip("Enforce community ban for federated user", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let alphaShortname = `@lemmy_alpha@lemmy-alpha:8541`;
  let alphaPerson = (await resolvePerson(beta, alphaShortname)).person;
  if (!alphaPerson) {
    throw "Missing alpha person";
  }
  expect(alphaPerson).toBeDefined();

  // make a post in beta, it goes through
  let postRes1 = await createPost(alpha, betaCommunity.community.id);
  let searchBeta1 = await searchPostLocal(beta, postRes1.post_view.post);
  expect(searchBeta1.posts[0]).toBeDefined();

  // ban alpha from beta community
  let banAlpha = await banPersonFromCommunity(
    beta,
    alphaPerson.person.id,
    2,
    true,
    true,
  );
  expect(banAlpha.banned).toBe(true);

  // ensure that the post by alpha got removed
  await expect(getPost(alpha, searchBeta1.posts[0].post.id)).rejects.toBe(
    "unknown",
  );

  // Alpha tries to make post on beta, but it fails because of ban
  await expect(createPost(alpha, betaCommunity.community.id)).rejects.toBe(
    "banned_from_community",
  );

  // Unban alpha
  let unBanAlpha = await banPersonFromCommunity(
    beta,
    alphaPerson.person.id,
    2,
    false,
    false,
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

test("A and G subscribe to B (center) A posts, it gets announced to G", async () => {
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let postRes = await createPost(alpha, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();

  let betaPost = (await resolvePost(gamma, postRes.post_view.post)).post;
  expect(betaPost?.post.name).toBeDefined();
});

test("Report a post", async () => {
  // Note, this is a different one from the setup
  let betaCommunity = (await resolveBetaCommunity(beta)).community;
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let postRes = await createPost(beta, betaCommunity.community.id);
  expect(postRes.post_view.post).toBeDefined();

  let alphaPost = (await resolvePost(alpha, postRes.post_view.post)).post;
  if (!alphaPost) {
    throw "Missing alpha post";
  }
  let alphaReport = (
    await reportPost(alpha, alphaPost.post.id, randomString(10))
  ).post_report_view.post_report;

  let betaReport = (await waitUntil(
    () =>
      listPostReports(beta).then(p =>
        p.post_reports.find(
          r =>
            r.post_report.original_post_name === alphaReport.original_post_name,
        ),
      ),
    res => !!res,
  ))!.post_report;
  expect(betaReport).toBeDefined();
  expect(betaReport.resolved).toBe(false);
  expect(betaReport.original_post_name).toBe(alphaReport.original_post_name);
  expect(betaReport.original_post_url).toBe(alphaReport.original_post_url);
  expect(betaReport.original_post_body).toBe(alphaReport.original_post_body);
  expect(betaReport.reason).toBe(alphaReport.reason);
});
