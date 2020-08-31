import {
  alpha,
  beta,
  setupLogins,
  searchForBetaCommunity,
  createCommunity,
  deleteCommunity,
  removeCommunity,
  delay,
} from './shared';

beforeAll(async () => {
  await setupLogins();
});

test('Create community', async () => {
  let communityRes = await createCommunity(alpha);
  expect(communityRes.community.name).toBeDefined();

  // A dupe check
  let prevName = communityRes.community.name;
  let communityRes2 = await createCommunity(alpha, prevName);
  expect(communityRes2['error']).toBe('community_already_exists');
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
});

test('Search for beta community', async () => {
  let search = await searchForBetaCommunity(alpha);
  expect(search.communities[0].name).toBe('main');
});
