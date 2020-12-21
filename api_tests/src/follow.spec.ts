jest.setTimeout(120000);
import {
  alpha,
  setupLogins,
  searchForBetaCommunity,
  followCommunity,
  checkFollowedCommunities,
  unfollowRemotes,
} from './shared';

beforeAll(async () => {
  await setupLogins();
});

afterAll(async () => {
  await unfollowRemotes(alpha);
});

test('Follow federated community', async () => {
  let search = await searchForBetaCommunity(alpha); // TODO sometimes this is returning null?
  let follow = await followCommunity(
    alpha,
    true,
    search.communities[0].community.id
  );

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(false);
  expect(follow.community_view.community.name).toBe('main');

  // Check it from local
  let followCheck = await checkFollowedCommunities(alpha);
  let remoteCommunityId = followCheck.communities.find(
    c => c.community.local == false
  ).community.id;
  expect(remoteCommunityId).toBeDefined();

  // Test an unfollow
  let unfollow = await followCommunity(alpha, false, remoteCommunityId);
  expect(unfollow.community_view.community.local).toBe(false);

  // Make sure you are unsubbed locally
  let unfollowCheck = await checkFollowedCommunities(alpha);
  expect(unfollowCheck.communities.length).toBeGreaterThanOrEqual(1);
});
