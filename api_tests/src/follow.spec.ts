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

afterAll(unfollows);

test("Follow local community", async () => {
  let user = await registerUser(beta, betaUrl);

  let community = (await resolveBetaCommunity(user)).community!;
  let follow = await followCommunity(user, true, community.community.id);

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(true);
  expect(follow.community_view.subscribed).toBe("Subscribed");
  expect(follow.community_view.counts.subscribers).toBe(
    community.counts.subscribers + 1,
  );
  expect(follow.community_view.counts.subscribers_local).toBe(
    community.counts.subscribers_local + 1,
  );

  // Test an unfollow
  let unfollow = await followCommunity(user, false, community.community.id);
  expect(unfollow.community_view.subscribed).toBe("NotSubscribed");
  expect(unfollow.community_view.counts.subscribers).toBe(
    community.counts.subscribers,
  );
  expect(unfollow.community_view.counts.subscribers_local).toBe(
    community.counts.subscribers_local,
  );
});

test("Follow federated community", async () => {
  // It takes about 1 second for the community aggregates to federate
  const betaCommunityInitial = (
    await waitUntil(
      () => resolveBetaCommunity(alpha),
      c => !!c.community && c.community?.counts.subscribers >= 1,
    )
  ).community;
  if (!betaCommunityInitial) {
    throw "Missing beta community";
  }
  let follow = await followCommunity(
    alpha,
    true,
    betaCommunityInitial.community.id,
  );
  expect(follow.community_view.subscribed).toBe("Pending");
  const betaCommunity = (
    await waitUntil(
      () => resolveBetaCommunity(alpha),
      c => c.community?.subscribed === "Subscribed",
    )
  ).community;

  // Make sure the follow response went through
  expect(betaCommunity?.community.local).toBe(false);
  expect(betaCommunity?.community.name).toBe("main");
  expect(betaCommunity?.subscribed).toBe("Subscribed");
  expect(betaCommunity?.counts.subscribers_local).toBe(
    betaCommunityInitial.counts.subscribers_local + 1,
  );

  // check that unfollow was federated
  let communityOnBeta1 = await resolveBetaCommunity(beta);
  expect(communityOnBeta1.community?.counts.subscribers).toBe(
    betaCommunityInitial.counts.subscribers + 1,
  );

  // Check it from local
  let site = await getSite(alpha);
  let remoteCommunityId = site.my_user?.follows.find(
    c =>
      c.community.local == false &&
      c.community.id === betaCommunityInitial.community.id,
  )?.community.id;
  expect(remoteCommunityId).toBeDefined();

  if (!remoteCommunityId) {
    throw "Missing remote community id";
  }

  // Test an unfollow
  let unfollow = await followCommunity(alpha, false, remoteCommunityId);
  expect(unfollow.community_view.subscribed).toBe("NotSubscribed");

  // Make sure you are unsubbed locally
  let siteUnfollowCheck = await getSite(alpha);
  expect(
    siteUnfollowCheck.my_user?.follows.find(
      c => c.community.id === betaCommunityInitial.community.id,
    ),
  ).toBe(undefined);

  // check that unfollow was federated
  let communityOnBeta2 = await waitUntil(
    () => resolveBetaCommunity(beta),
    c =>
      c.community?.counts.subscribers ===
      betaCommunityInitial.counts.subscribers,
  );
  expect(communityOnBeta2.community?.counts.subscribers).toBe(
    betaCommunityInitial.counts.subscribers,
  );
  expect(communityOnBeta2.community?.counts.subscribers_local).toBe(1);
});
