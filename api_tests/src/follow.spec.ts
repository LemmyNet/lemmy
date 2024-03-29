jest.setTimeout(120000);

import {
  alpha,
  setupLogins,
  resolveBetaCommunity,
  followCommunity,
  getSite,
  waitUntil,
  beta,
  betaUrl,
  registerUser,
  unfollows,
} from "./shared";

beforeAll(setupLogins);

afterAll(() => {
  unfollows();
});

test("Follow local community", async () => {
  let user = await registerUser(beta, betaUrl);

  let community = (await resolveBetaCommunity(user)).community!;
  expect(community.counts.subscribers).toBe(1);
  expect(community.counts.subscribers_local).toBe(1);
  let follow = await followCommunity(user, true, community.community.id);

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(true);
  expect(follow.community_view.subscribed).toBe("Subscribed");
  expect(follow.community_view.counts.subscribers).toBe(2);
  expect(follow.community_view.counts.subscribers_local).toBe(2);

  // Test an unfollow
  let unfollow = await followCommunity(user, false, community.community.id);
  expect(unfollow.community_view.subscribed).toBe("NotSubscribed");
  expect(unfollow.community_view.counts.subscribers).toBe(1);
  expect(unfollow.community_view.counts.subscribers_local).toBe(1);
});

test("Follow federated community", async () => {
  // It takes about 1 second for the community aggregates to federate
  let betaCommunity = (
    await waitUntil(
      () => resolveBetaCommunity(alpha),
      c =>
        c.community?.counts.subscribers === 1 &&
        c.community.counts.subscribers_local === 0,
    )
  ).community;
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let follow = await followCommunity(alpha, true, betaCommunity.community.id);
  expect(follow.community_view.subscribed).toBe("Pending");
  betaCommunity = (
    await waitUntil(
      () => resolveBetaCommunity(alpha),
      c => c.community?.subscribed === "Subscribed",
    )
  ).community;

  // Make sure the follow response went through
  expect(betaCommunity?.community.local).toBe(false);
  expect(betaCommunity?.community.name).toBe("main");
  expect(betaCommunity?.subscribed).toBe("Subscribed");
  expect(betaCommunity?.counts.subscribers_local).toBe(1);

  // check that unfollow was federated
  let communityOnBeta1 = await resolveBetaCommunity(beta);
  expect(communityOnBeta1.community?.counts.subscribers).toBe(2);
  expect(communityOnBeta1.community?.counts.subscribers_local).toBe(1);

  // Check it from local
  let site = await getSite(alpha);
  let remoteCommunityId = site.my_user?.follows.find(
    c => c.community.local == false,
  )?.community.id;
  expect(remoteCommunityId).toBeDefined();
  expect(site.my_user?.follows.length).toBe(2);

  if (!remoteCommunityId) {
    throw "Missing remote community id";
  }

  // Test an unfollow
  let unfollow = await followCommunity(alpha, false, remoteCommunityId);
  expect(unfollow.community_view.subscribed).toBe("NotSubscribed");

  // Make sure you are unsubbed locally
  let siteUnfollowCheck = await getSite(alpha);
  expect(siteUnfollowCheck.my_user?.follows.length).toBe(1);

  // check that unfollow was federated
  let communityOnBeta2 = await resolveBetaCommunity(beta);
  expect(communityOnBeta2.community?.counts.subscribers).toBe(1);
  expect(communityOnBeta2.community?.counts.subscribers_local).toBe(1);
});
