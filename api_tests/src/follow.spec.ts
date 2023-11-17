jest.setTimeout(120000);

import { LemmyHttp } from "lemmy-js-client";
import {
  alpha,
  setupLogins,
  resolveBetaCommunity,
  followCommunity,
  unfollowRemotes,
  getSite,
  waitUntil,
  beta,
  registerUser,
  betaUrl,
} from "./shared";

beforeAll(setupLogins);

afterAll(() => {
  unfollowRemotes(alpha);
});

test("Follow local community", async () => {
  let userRes = await registerUser(beta);
  expect(userRes.jwt).toBeDefined();
  let user = new LemmyHttp(betaUrl, {
    headers: { Authorization: `Bearer ${userRes.jwt ?? ""}` },
  });

  let community = (await resolveBetaCommunity(user)).community!;
  expect(community.counts.subscribers).toBe(1);
  let follow = await followCommunity(user, true, community.community.id);

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(true);
  expect(follow.community_view.subscribed).toBe("Subscribed");
  expect(follow.community_view.counts.subscribers).toBe(2);

  // Test an unfollow
  let unfollow = await followCommunity(user, false, community.community.id);
  expect(unfollow.community_view.subscribed).toBe("NotSubscribed");
  expect(unfollow.community_view.counts.subscribers).toBe(1);
});

test("Follow federated community", async () => {
  let betaCommunity = (await resolveBetaCommunity(alpha)).community;
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

  // check that unfollow was federated
  let communityOnBeta1 = await resolveBetaCommunity(beta);
  expect(communityOnBeta1.community?.counts.subscribers).toBe(2);

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
});
