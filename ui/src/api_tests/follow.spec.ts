import {
  alpha,
  setupLogins,
  searchForBetaCommunity,
  followCommunity,
  checkFollowedCommunities,
  unfollowRemotes,
  delay,
} from './shared';

beforeAll(async () => {
  await setupLogins();
});

afterAll(async () => {
  await unfollowRemotes(alpha);
});

test('Follow federated community', async () => {
  let search = await searchForBetaCommunity(alpha); // TODO sometimes this is returning null?
  let follow = await followCommunity(alpha, true, search.communities[0].id);

  // Make sure the follow response went through
  expect(follow.community.local).toBe(false);
  expect(follow.community.name).toBe('main');
  await delay();

  // Check it from local
  let followCheck = await checkFollowedCommunities(alpha);
  let remoteCommunityId = followCheck.communities.filter(
    c => c.community_local == false
  )[0].community_id;
  expect(remoteCommunityId).toBeDefined();

  // Test an unfollow
  let unfollow = await followCommunity(alpha, false, remoteCommunityId);
  expect(unfollow.community.local).toBe(false);
  await delay();

  // Make sure you are unsubbed locally
  let unfollowCheck = await checkFollowedCommunities(alpha);
  expect(unfollowCheck.communities.length).toBeGreaterThanOrEqual(1);
});
