jest.setTimeout(120000);
import {
  alpha,
  setupLogins,
  resolveBetaCommunity,
  followCommunity,
  unfollowRemotes,
  getSite,
} from './shared';

beforeAll(async () => {
  await setupLogins();
});

afterAll(async () => {
  await unfollowRemotes(alpha);
});

test('Follow federated community', async () => {
  let betaCommunity = (await resolveBetaCommunity(alpha)).community;
  let follow = await followCommunity(
    alpha,
    true,
    betaCommunity.community.id
  );

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(false);
  expect(follow.community_view.community.name).toBe('main');

  // Check it from local
  let site = await getSite(alpha);
  let remoteCommunityId = site.my_user.follows.find(
    c => c.community.local == false
  ).community.id;
  expect(remoteCommunityId).toBeDefined();

  // Test an unfollow
  let unfollow = await followCommunity(alpha, false, remoteCommunityId);
  expect(unfollow.community_view.community.local).toBe(false);

  // Make sure you are unsubbed locally
  let siteUnfollowCheck = await getSite(alpha);
  expect(siteUnfollowCheck.my_user.follows.length).toBeGreaterThanOrEqual(1);
});
