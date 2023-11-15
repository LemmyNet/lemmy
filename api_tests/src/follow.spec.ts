jest.setTimeout(120000);

import {
  alpha,
  setupLogins,
  resolveBetaCommunity,
  followCommunity,
  unfollowRemotes,
  getSite,
  waitUntil,
} from "./shared";

beforeAll(setupLogins);

afterAll(() => {
  unfollowRemotes(alpha);
});

test("Follow federated community", async () => {
  let betaCommunity = (await resolveBetaCommunity(alpha)).community;
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  await followCommunity(alpha, true, betaCommunity.community.id);
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
});
