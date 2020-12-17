jest.setTimeout(120000);
import {
  alpha,
  beta,
  setupLogins,
  searchForCommunity,
  createCommunity,
  deleteCommunity,
  removeCommunity,
  getCommunity,
  followCommunity,
} from './shared';
import {
  Community,
} from 'lemmy-js-client';

beforeAll(async () => {
  await setupLogins();
});

function assertCommunityFederation(
  communityOne: Community,
  communityTwo: Community) {
  expect(communityOne.actor_id).toBe(communityTwo.actor_id);
  expect(communityOne.name).toBe(communityTwo.name);
  expect(communityOne.title).toBe(communityTwo.title);
  expect(communityOne.description).toBe(communityTwo.description);
  expect(communityOne.icon).toBe(communityTwo.icon);
  expect(communityOne.banner).toBe(communityTwo.banner);
  expect(communityOne.published).toBe(communityTwo.published);
  expect(communityOne.creator_actor_id).toBe(communityTwo.creator_actor_id);
  expect(communityOne.nsfw).toBe(communityTwo.nsfw);
  expect(communityOne.category_id).toBe(communityTwo.category_id);
  expect(communityOne.removed).toBe(communityTwo.removed);
  expect(communityOne.deleted).toBe(communityTwo.deleted);
}

test('Create community', async () => {
  let communityRes = await createCommunity(alpha);
  expect(communityRes.community.name).toBeDefined();

  // A dupe check
  let prevName = communityRes.community.name;
  let communityRes2 = await createCommunity(alpha, prevName);
  expect(communityRes2['error']).toBe('community_already_exists');

  // Cache the community on beta, make sure it has the other fields
  let searchShort = `!${prevName}@lemmy-alpha:8541`;
  let search = await searchForCommunity(beta, searchShort);
  let communityOnBeta = search.communities[0];
  assertCommunityFederation(communityOnBeta, communityRes.community);
});

test('Delete community', async () => {
  let communityRes = await createCommunity(beta);

  // Cache the community on Alpha
  let searchShort = `!${communityRes.community.name}@lemmy-beta:8551`;
  let search = await searchForCommunity(alpha, searchShort);
  let communityOnAlpha = search.communities[0];
  assertCommunityFederation(communityOnAlpha, communityRes.community);

  // Follow the community from alpha
  let follow = await followCommunity(alpha, true, communityOnAlpha.id);

  // Make sure the follow response went through
  expect(follow.community.local).toBe(false);

  let deleteCommunityRes = await deleteCommunity(
    beta,
    true,
    communityRes.community.id
  );
  expect(deleteCommunityRes.community.deleted).toBe(true);

  // Make sure it got deleted on A
  let communityOnAlphaDeleted = await getCommunity(alpha, communityOnAlpha.id);
  expect(communityOnAlphaDeleted.community.deleted).toBe(true);

  // Undelete
  let undeleteCommunityRes = await deleteCommunity(
    beta,
    false,
    communityRes.community.id
  );
  expect(undeleteCommunityRes.community.deleted).toBe(false);

  // Make sure it got undeleted on A
  let communityOnAlphaUnDeleted = await getCommunity(alpha, communityOnAlpha.id);
  expect(communityOnAlphaUnDeleted.community.deleted).toBe(false);
});

test('Remove community', async () => {
  let communityRes = await createCommunity(beta);

  // Cache the community on Alpha
  let searchShort = `!${communityRes.community.name}@lemmy-beta:8551`;
  let search = await searchForCommunity(alpha, searchShort);
  let communityOnAlpha = search.communities[0];
  assertCommunityFederation(communityOnAlpha, communityRes.community);

  // Follow the community from alpha
  let follow = await followCommunity(alpha, true, communityOnAlpha.id);

  // Make sure the follow response went through
  expect(follow.community.local).toBe(false);

  let removeCommunityRes = await removeCommunity(
    beta,
    true,
    communityRes.community.id
  );
  expect(removeCommunityRes.community.removed).toBe(true);

  // Make sure it got Removed on A
  let communityOnAlphaRemoved = await getCommunity(alpha, communityOnAlpha.id);
  expect(communityOnAlphaRemoved.community.removed).toBe(true);

  // unremove
  let unremoveCommunityRes = await removeCommunity(
    beta,
    false,
    communityRes.community.id
  );
  expect(unremoveCommunityRes.community.removed).toBe(false);

  // Make sure it got undeleted on A
  let communityOnAlphaUnRemoved = await getCommunity(alpha, communityOnAlpha.id);
  expect(communityOnAlphaUnRemoved.community.removed).toBe(false);
});

test('Search for beta community', async () => {
  let communityRes = await createCommunity(beta);
  expect(communityRes.community.name).toBeDefined();

  let searchShort = `!${communityRes.community.name}@lemmy-beta:8551`;
  let search = await searchForCommunity(alpha, searchShort);
  let communityOnAlpha = search.communities[0];
  assertCommunityFederation(communityOnAlpha, communityRes.community);
});
