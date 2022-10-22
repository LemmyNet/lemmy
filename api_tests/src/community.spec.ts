jest.setTimeout(120000);
import { CommunityView } from "lemmy-js-client";

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
} from "./shared";

beforeAll(async () => {
  await setupLogins();
});

function assertCommunityFederation(
  communityOne: CommunityView,
  communityTwo: CommunityView
) {
  expect(communityOne.community.actor_id).toBe(communityTwo.community.actor_id);
  expect(communityOne.community.name).toBe(communityTwo.community.name);
  expect(communityOne.community.title).toBe(communityTwo.community.title);
  expect(communityOne.community.description.unwrapOr("none")).toBe(
    communityTwo.community.description.unwrapOr("none")
  );
  expect(communityOne.community.icon.unwrapOr("none")).toBe(
    communityTwo.community.icon.unwrapOr("none")
  );
  expect(communityOne.community.banner.unwrapOr("none")).toBe(
    communityTwo.community.banner.unwrapOr("none")
  );
  expect(communityOne.community.published).toBe(
    communityTwo.community.published
  );
  expect(communityOne.community.nsfw).toBe(communityTwo.community.nsfw);
  expect(communityOne.community.removed).toBe(communityTwo.community.removed);
  expect(communityOne.community.deleted).toBe(communityTwo.community.deleted);
}

test("Create community", async () => {
  let communityRes = await createCommunity(alpha);
  expect(communityRes.community_view.community.name).toBeDefined();

  // A dupe check
  let prevName = communityRes.community_view.community.name;
  let communityRes2: any = await createCommunity(alpha, prevName);
  expect(communityRes2["error"]).toBe("community_already_exists");

  // Cache the community on beta, make sure it has the other fields
  let searchShort = `!${prevName}@lemmy-alpha:8541`;
  let betaCommunity = (
    await resolveCommunity(beta, searchShort)
  ).community.unwrap();
  assertCommunityFederation(betaCommunity, communityRes.community_view);
});

test("Delete community", async () => {
  let communityRes = await createCommunity(beta);

  // Cache the community on Alpha
  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = (
    await resolveCommunity(alpha, searchShort)
  ).community.unwrap();
  assertCommunityFederation(alphaCommunity, communityRes.community_view);

  // Follow the community from alpha
  let follow = await followCommunity(alpha, true, alphaCommunity.community.id);

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(false);

  let deleteCommunityRes = await deleteCommunity(
    beta,
    true,
    communityRes.community_view.community.id
  );
  expect(deleteCommunityRes.community_view.community.deleted).toBe(true);
  expect(deleteCommunityRes.community_view.community.title).toBe(
    communityRes.community_view.community.title
  );

  // Make sure it got deleted on A
  let communityOnAlphaDeleted = await getCommunity(
    alpha,
    alphaCommunity.community.id
  );
  expect(communityOnAlphaDeleted.community_view.community.deleted).toBe(true);

  // Undelete
  let undeleteCommunityRes = await deleteCommunity(
    beta,
    false,
    communityRes.community_view.community.id
  );
  expect(undeleteCommunityRes.community_view.community.deleted).toBe(false);

  // Make sure it got undeleted on A
  let communityOnAlphaUnDeleted = await getCommunity(
    alpha,
    alphaCommunity.community.id
  );
  expect(communityOnAlphaUnDeleted.community_view.community.deleted).toBe(
    false
  );
});

test("Remove community", async () => {
  let communityRes = await createCommunity(beta);

  // Cache the community on Alpha
  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = (
    await resolveCommunity(alpha, searchShort)
  ).community.unwrap();
  assertCommunityFederation(alphaCommunity, communityRes.community_view);

  // Follow the community from alpha
  let follow = await followCommunity(alpha, true, alphaCommunity.community.id);

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(false);

  let removeCommunityRes = await removeCommunity(
    beta,
    true,
    communityRes.community_view.community.id
  );
  expect(removeCommunityRes.community_view.community.removed).toBe(true);
  expect(removeCommunityRes.community_view.community.title).toBe(
    communityRes.community_view.community.title
  );

  // Make sure it got Removed on A
  let communityOnAlphaRemoved = await getCommunity(
    alpha,
    alphaCommunity.community.id
  );
  expect(communityOnAlphaRemoved.community_view.community.removed).toBe(true);

  // unremove
  let unremoveCommunityRes = await removeCommunity(
    beta,
    false,
    communityRes.community_view.community.id
  );
  expect(unremoveCommunityRes.community_view.community.removed).toBe(false);

  // Make sure it got undeleted on A
  let communityOnAlphaUnRemoved = await getCommunity(
    alpha,
    alphaCommunity.community.id
  );
  expect(communityOnAlphaUnRemoved.community_view.community.removed).toBe(
    false
  );
});

test("Search for beta community", async () => {
  let communityRes = await createCommunity(beta);
  expect(communityRes.community_view.community.name).toBeDefined();

  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = (
    await resolveCommunity(alpha, searchShort)
  ).community.unwrap();
  assertCommunityFederation(alphaCommunity, communityRes.community_view);
});

test("Admin actions in remote community are not federated to origin", async () => {
  // create a community on alpha
  let communityRes = (await createCommunity(alpha)).community_view;
  expect(communityRes.community.name).toBeDefined();

  // gamma follows community and posts in it
  let gammaCommunity = (
    await resolveCommunity(gamma, communityRes.community.actor_id)
  ).community.unwrap();
  let gammaFollow = await followCommunity(
    gamma,
    true,
    gammaCommunity.community.id
  );
  expect(gammaFollow.community_view.subscribed).toBe("Subscribed");
  let gammaPost = (await createPost(gamma, gammaCommunity.community.id))
    .post_view;
  expect(gammaPost.post.id).toBeDefined();
  expect(gammaPost.creator_banned_from_community).toBe(false);

  // admin of beta decides to ban gamma from community
  let betaCommunity = (
    await resolveCommunity(beta, communityRes.community.actor_id)
  ).community.unwrap();
  let bannedUserInfo1 = (await getSite(gamma)).my_user.unwrap().local_user_view
    .person;
  let bannedUserInfo2 = (
    await resolvePerson(beta, bannedUserInfo1.actor_id)
  ).person.unwrap();
  let banRes = await banPersonFromCommunity(
    beta,
    bannedUserInfo2.person.id,
    betaCommunity.community.id,
    true,
    true
  );
  expect(banRes.banned).toBe(true);

  // ban doesnt federate to community's origin instance alpha
  let alphaPost = (await resolvePost(alpha, gammaPost.post)).post.unwrap();
  expect(alphaPost.creator_banned_from_community).toBe(false);

  // and neither to gamma
  let gammaPost2 = await getPost(gamma, gammaPost.post.id);
  expect(gammaPost2.post_view.creator_banned_from_community).toBe(false);
});
