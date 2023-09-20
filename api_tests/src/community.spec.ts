jest.setTimeout(120000);

import { CommunityView } from "lemmy-js-client/dist/types/CommunityView";
import {
  alpha,
  beta,
  gamma,
  setupLogins,
  resolveCommunity,
  createCommunity,
  deleteCommunity,
  removeCommunity,
  getCommunity,
  followCommunity,
  banPersonFromCommunity,
  resolvePerson,
  getSite,
  createPost,
  getPost,
  resolvePost,
  registerUser,
  API,
  getPosts,
  getComments,
  createComment,
  getCommunityByName,
  blockInstance,
  waitUntil,
  delay,
  waitForPost,
} from "./shared";

beforeAll(async () => {
  await setupLogins();
});

function assertCommunityFederation(
  communityOne?: CommunityView,
  communityTwo?: CommunityView,
) {
  expect(communityOne?.community.actor_id).toBe(
    communityTwo?.community.actor_id,
  );
  expect(communityOne?.community.name).toBe(communityTwo?.community.name);
  expect(communityOne?.community.title).toBe(communityTwo?.community.title);
  expect(communityOne?.community.description).toBe(
    communityTwo?.community.description,
  );
  expect(communityOne?.community.icon).toBe(communityTwo?.community.icon);
  expect(communityOne?.community.banner).toBe(communityTwo?.community.banner);
  expect(communityOne?.community.published).toBe(
    communityTwo?.community.published,
  );
  expect(communityOne?.community.nsfw).toBe(communityTwo?.community.nsfw);
  expect(communityOne?.community.removed).toBe(communityTwo?.community.removed);
  expect(communityOne?.community.deleted).toBe(communityTwo?.community.deleted);
}

test("Create community", async () => {
  let communityRes = await createCommunity(alpha);
  expect(communityRes.community_view.community.name).toBeDefined();

  // A dupe check
  let prevName = communityRes.community_view.community.name;
  await expect(createCommunity(alpha, prevName)).rejects.toBe(
    "community_already_exists",
  );

  // Cache the community on beta, make sure it has the other fields
  let searchShort = `!${prevName}@lemmy-alpha:8541`;
  let betaCommunity = (await resolveCommunity(beta, searchShort)).community;
  assertCommunityFederation(betaCommunity, communityRes.community_view);
});

test("Delete community", async () => {
  let communityRes = await createCommunity(beta);

  // Cache the community on Alpha
  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = (await resolveCommunity(alpha, searchShort)).community;
  if (!alphaCommunity) {
    throw "Missing alpha community";
  }
  assertCommunityFederation(alphaCommunity, communityRes.community_view);

  // Follow the community from alpha
  let follow = await followCommunity(alpha, true, alphaCommunity.community.id);

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(false);

  let deleteCommunityRes = await deleteCommunity(
    beta,
    true,
    communityRes.community_view.community.id,
  );
  expect(deleteCommunityRes.community_view.community.deleted).toBe(true);
  expect(deleteCommunityRes.community_view.community.title).toBe(
    communityRes.community_view.community.title,
  );

  // Make sure it got deleted on A
  let communityOnAlphaDeleted = await waitUntil(
    () => getCommunity(alpha, alphaCommunity!.community.id),
    g => g.community_view.community.deleted,
  );
  expect(communityOnAlphaDeleted.community_view.community.deleted).toBe(true);

  // Undelete
  let undeleteCommunityRes = await deleteCommunity(
    beta,
    false,
    communityRes.community_view.community.id,
  );
  expect(undeleteCommunityRes.community_view.community.deleted).toBe(false);

  // Make sure it got undeleted on A
  let communityOnAlphaUnDeleted = await waitUntil(
    () => getCommunity(alpha, alphaCommunity!.community.id),
    g => !g.community_view.community.deleted,
  );
  expect(communityOnAlphaUnDeleted.community_view.community.deleted).toBe(
    false,
  );
});

test("Remove community", async () => {
  let communityRes = await createCommunity(beta);

  // Cache the community on Alpha
  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = (await resolveCommunity(alpha, searchShort)).community;
  if (!alphaCommunity) {
    throw "Missing alpha community";
  }
  assertCommunityFederation(alphaCommunity, communityRes.community_view);

  // Follow the community from alpha
  let follow = await followCommunity(alpha, true, alphaCommunity.community.id);

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(false);

  let removeCommunityRes = await removeCommunity(
    beta,
    true,
    communityRes.community_view.community.id,
  );
  expect(removeCommunityRes.community_view.community.removed).toBe(true);
  expect(removeCommunityRes.community_view.community.title).toBe(
    communityRes.community_view.community.title,
  );

  // Make sure it got Removed on A
  let communityOnAlphaRemoved = await waitUntil(
    () => getCommunity(alpha, alphaCommunity!.community.id),
    g => g.community_view.community.removed,
  );
  expect(communityOnAlphaRemoved.community_view.community.removed).toBe(true);

  // unremove
  let unremoveCommunityRes = await removeCommunity(
    beta,
    false,
    communityRes.community_view.community.id,
  );
  expect(unremoveCommunityRes.community_view.community.removed).toBe(false);

  // Make sure it got undeleted on A
  let communityOnAlphaUnRemoved = await waitUntil(
    () => getCommunity(alpha, alphaCommunity!.community.id),
    g => !g.community_view.community.removed,
  );
  expect(communityOnAlphaUnRemoved.community_view.community.removed).toBe(
    false,
  );
});

test("Search for beta community", async () => {
  let communityRes = await createCommunity(beta);
  expect(communityRes.community_view.community.name).toBeDefined();

  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = (await resolveCommunity(alpha, searchShort)).community;
  assertCommunityFederation(alphaCommunity, communityRes.community_view);
});

test("Admin actions in remote community are not federated to origin", async () => {
  // create a community on alpha
  let communityRes = (await createCommunity(alpha)).community_view;
  expect(communityRes.community.name).toBeDefined();

  // gamma follows community and posts in it
  let gammaCommunity = (
    await resolveCommunity(gamma, communityRes.community.actor_id)
  ).community;
  if (!gammaCommunity) {
    throw "Missing gamma community";
  }
  await followCommunity(gamma, true, gammaCommunity.community.id);
  gammaCommunity = (
    await waitUntil(
      () => resolveCommunity(gamma, communityRes.community.actor_id),
      g => g.community?.subscribed === "Subscribed",
    )
  ).community;
  if (!gammaCommunity) {
    throw "Missing gamma community";
  }
  expect(gammaCommunity.subscribed).toBe("Subscribed");
  let gammaPost = (await createPost(gamma, gammaCommunity.community.id))
    .post_view;
  expect(gammaPost.post.id).toBeDefined();
  expect(gammaPost.creator_banned_from_community).toBe(false);

  // admin of beta decides to ban gamma from community
  let betaCommunity = (
    await resolveCommunity(beta, communityRes.community.actor_id)
  ).community;
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let bannedUserInfo1 = (await getSite(gamma)).my_user?.local_user_view.person;
  if (!bannedUserInfo1) {
    throw "Missing banned user 1";
  }
  let bannedUserInfo2 = (await resolvePerson(beta, bannedUserInfo1.actor_id))
    .person;
  if (!bannedUserInfo2) {
    throw "Missing banned user 2";
  }
  let banRes = await banPersonFromCommunity(
    beta,
    bannedUserInfo2.person.id,
    betaCommunity.community.id,
    true,
    true,
  );
  expect(banRes.banned).toBe(true);

  // ban doesnt federate to community's origin instance alpha
  let alphaPost = (await resolvePost(alpha, gammaPost.post, false)).post;
  expect(alphaPost?.creator_banned_from_community).toBe(false);

  // and neither to gamma
  let gammaPost2 = await getPost(gamma, gammaPost.post.id);
  expect(gammaPost2.post_view.creator_banned_from_community).toBe(false);
});

test("moderator view", async () => {
  // register a new user with their own community on alpha and post to it
  let otherUser: API = {
    auth: (await registerUser(alpha)).jwt ?? "",
    client: alpha.client,
  };
  expect(otherUser.auth).not.toBe("");

  let otherCommunity = (await createCommunity(otherUser)).community_view;
  expect(otherCommunity.community.name).toBeDefined();
  let otherPost = (await createPost(otherUser, otherCommunity.community.id))
    .post_view;
  expect(otherPost.post.id).toBeDefined();

  let otherComment = (await createComment(otherUser, otherPost.post.id))
    .comment_view;
  expect(otherComment.comment.id).toBeDefined();

  // create a community and post on alpha
  let alphaCommunity = (await createCommunity(alpha)).community_view;
  expect(alphaCommunity.community.name).toBeDefined();
  let alphaPost = (await createPost(alpha, alphaCommunity.community.id))
    .post_view;
  expect(alphaPost.post.id).toBeDefined();

  let alphaComment = (await createComment(otherUser, alphaPost.post.id))
    .comment_view;
  expect(alphaComment.comment.id).toBeDefined();

  // other user also posts on alpha's community
  let otherAlphaPost = (
    await createPost(otherUser, alphaCommunity.community.id)
  ).post_view;
  expect(otherAlphaPost.post.id).toBeDefined();

  let otherAlphaComment = (
    await createComment(otherUser, otherAlphaPost.post.id)
  ).comment_view;
  expect(otherAlphaComment.comment.id).toBeDefined();

  // alpha lists posts and comments on home page, should contain all posts that were made
  let posts = (await getPosts(alpha, "All")).posts;
  expect(posts).toBeDefined();
  let postIds = posts.map(post => post.post.id);

  let comments = (await getComments(alpha, undefined, "All")).comments;
  expect(comments).toBeDefined();
  let commentIds = comments.map(comment => comment.comment.id);

  expect(postIds).toContain(otherPost.post.id);
  expect(commentIds).toContain(otherComment.comment.id);

  expect(postIds).toContain(alphaPost.post.id);
  expect(commentIds).toContain(alphaComment.comment.id);

  expect(postIds).toContain(otherAlphaPost.post.id);
  expect(commentIds).toContain(otherAlphaComment.comment.id);

  // in moderator view, alpha should not see otherPost, wich was posted on a community alpha doesn't moderate
  posts = (await getPosts(alpha, "ModeratorView")).posts;
  expect(posts).toBeDefined();
  postIds = posts.map(post => post.post.id);

  comments = (await getComments(alpha, undefined, "ModeratorView")).comments;
  expect(comments).toBeDefined();
  commentIds = comments.map(comment => comment.comment.id);

  expect(postIds).not.toContain(otherPost.post.id);
  expect(commentIds).not.toContain(otherComment.comment.id);

  expect(postIds).toContain(alphaPost.post.id);
  expect(commentIds).toContain(alphaComment.comment.id);

  expect(postIds).toContain(otherAlphaPost.post.id);
  expect(commentIds).toContain(otherAlphaComment.comment.id);
});

test("Get community for different casing on domain", async () => {
  let communityRes = await createCommunity(alpha);
  expect(communityRes.community_view.community.name).toBeDefined();

  // A dupe check
  let prevName = communityRes.community_view.community.name;
  await expect(createCommunity(alpha, prevName)).rejects.toBe(
    "community_already_exists",
  );

  // Cache the community on beta, make sure it has the other fields
  let communityName = `${communityRes.community_view.community.name}@LEMMY-ALPHA:8541`;
  let betaCommunity = (await getCommunityByName(beta, communityName))
    .community_view;
  assertCommunityFederation(betaCommunity, communityRes.community_view);
});

test("User blocks instance, communities are hidden", async () => {
  // create community and post on beta
  let communityRes = await createCommunity(beta);
  expect(communityRes.community_view.community.name).toBeDefined();
  let postRes = await createPost(
    beta,
    communityRes.community_view.community.id,
  );
  expect(postRes.post_view.post.id).toBeDefined();

  // fetch post to alpha
  let alphaPost = (await resolvePost(alpha, postRes.post_view.post, false))
    .post!;
  expect(alphaPost.post).toBeDefined();

  // post should be included in listing
  let listing = await getPosts(alpha, "All");
  let listing_ids = listing.posts.map(p => p.post.ap_id);
  expect(listing_ids).toContain(postRes.post_view.post.ap_id);

  // block the beta instance
  await blockInstance(alpha, alphaPost.community.instance_id, true);

  // after blocking, post should not be in listing
  let listing2 = await getPosts(alpha, "All");
  let listing_ids2 = listing2.posts.map(p => p.post.ap_id);
  expect(listing_ids2.indexOf(postRes.post_view.post.ap_id)).toBe(-1);

  // unblock instance again
  await blockInstance(alpha, alphaPost.community.instance_id, false);

  // post should be included in listing
  let listing3 = await getPosts(alpha, "All");
  let listing_ids3 = listing3.posts.map(p => p.post.ap_id);
  expect(listing_ids3).toContain(postRes.post_view.post.ap_id);
});
