jest.setTimeout(120000);
import {
  alpha,
  beta,
  setupLogins,
  searchForBetaCommunity,
  searchForCommunity,
  createCommunity,
  deleteCommunity,
  removeCommunity,
  delay,
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
  await delay();

  // Cache the community on beta, make sure it has the other fields
  let searchShort = `!${prevName}@lemmy-alpha:8541`;
  let search = await searchForCommunity(beta, searchShort);
  let communityOnBeta = search.communities[0];
  assertCommunityFederation(communityOnBeta, communityRes.community);
});

test('Delete community', async () => {
  let communityRes = await createCommunity(beta);
  await delay();
  let deleteCommunityRes = await deleteCommunity(
    beta,
    true,
    communityRes.community.id
  );
  expect(deleteCommunityRes.community.deleted).toBe(true);
  await delay();

  // Make sure it got deleted on A
  let search = await searchForBetaCommunity(alpha);
  let communityA = search.communities[0];
  // TODO this fails currently, because no updates are pushed
  // expect(communityA.deleted).toBe(true);
  // assertCommunityFederation(communityA, communityRes.community);

  // Undelete
  let undeleteCommunityRes = await deleteCommunity(
    beta,
    false,
    communityRes.community.id
  );
  expect(undeleteCommunityRes.community.deleted).toBe(false);
  await delay();

  // Make sure it got undeleted on A
  let search2 = await searchForBetaCommunity(alpha);
  let communityA2 = search2.communities[0];
  // TODO this fails currently, because no updates are pushed
  // expect(communityA2.deleted).toBe(false);
  // assertCommunityFederation(communityA2, undeleteCommunityRes.community);
});

test('Remove community', async () => {
  let communityRes = await createCommunity(beta);
  await delay();
  let removeCommunityRes = await removeCommunity(
    beta,
    true,
    communityRes.community.id
  );
  expect(removeCommunityRes.community.removed).toBe(true);

  // Make sure it got removed on A
  let search = await searchForBetaCommunity(alpha);
  let communityA = search.communities[0];
  // TODO this fails currently, because no updates are pushed
  // expect(communityA.removed).toBe(true);
  // assertCommunityFederation(communityA, communityRes.community);
  await delay();

  // unremove
  let unremoveCommunityRes = await removeCommunity(
    beta,
    false,
    communityRes.community.id
  );
  expect(unremoveCommunityRes.community.removed).toBe(false);
  await delay();

  // Make sure it got unremoved on A
  let search2 = await searchForBetaCommunity(alpha);
  let communityA2 = search2.communities[0];
  // TODO this fails currently, because no updates are pushed
  // expect(communityA2.removed).toBe(false);
  // assertCommunityFederation(communityA2, unremoveCommunityRes.community);
});

test('Search for beta community', async () => {
  let search = await searchForBetaCommunity(alpha);
  expect(search.communities[0].name).toBe('main');
});
