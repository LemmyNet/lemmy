jest.setTimeout(120000);
import { SubscribedType } from "lemmy-js-client";

import {
  alpha,
  setupLogins,
  resolveBetaCommunity,
  followCommunity,
  unfollowRemotes,
  getSite,
} from "./shared";

beforeAll(async () => {
  await setupLogins();
});

afterAll(async () => {
  await unfollowRemotes(alpha);
});

test("Follow federated community", async () => {
  let betaCommunity = (await resolveBetaCommunity(alpha)).community.unwrap();
  await followCommunity(alpha, true, betaCommunity.community.id);
  betaCommunity = (await resolveBetaCommunity(alpha)).community.unwrap();

  // Make sure the follow response went through
  expect(betaCommunity.community.local).toBe(false);
  expect(betaCommunity.community.name).toBe("main");
  expect(betaCommunity.subscribed).toBe(SubscribedType.Subscribed);

  // Check it from local
  let site = await getSite(alpha);
  let remoteCommunityId = site.my_user
    .unwrap()
    .follows.find(c => c.community.local == false).community.id;
  expect(remoteCommunityId).toBeDefined();
  expect(site.my_user.unwrap().follows.length).toBe(2);

  // Test an unfollow
  let unfollow = await followCommunity(alpha, false, remoteCommunityId);
  expect(unfollow.community_view.subscribed).toBe(SubscribedType.NotSubscribed);

  // Make sure you are unsubbed locally
  let siteUnfollowCheck = await getSite(alpha);
  expect(siteUnfollowCheck.my_user.unwrap().follows.length).toBe(1);
});
