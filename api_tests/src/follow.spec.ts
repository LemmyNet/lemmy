jest.setTimeout(120000);

import {
  alpha,
  setupLogins,
  resolveBetaCommunity,
  followCommunity,
  waitUntil,
  beta,
  betaUrl,
  registerUser,
  unfollows,
  getMyUser,
  alphaUrl,
} from "./shared";

beforeAll(setupLogins);

afterAll(unfollows);

test("Follow local community", async () => {
  let user = await registerUser(beta, betaUrl);

  let community = await resolveBetaCommunity(user);
  let follow = await followCommunity(user, true, community!.community.id);

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(true);
  expect(follow.community_view.community_actions?.follow_state).toBe(
    "accepted",
  );
  expect(follow.community_view.community.subscribers).toBe(
    community!.community.subscribers + 1,
  );
  expect(follow.community_view.community.subscribers_local).toBe(
    community!.community.subscribers_local + 1,
  );

  // Test an unfollow
  let unfollow = await followCommunity(user, false, community!.community.id);
  expect(
    unfollow.community_view.community_actions?.follow_state,
  ).toBeUndefined();
  expect(unfollow.community_view.community.subscribers).toBe(
    community?.community.subscribers,
  );
  expect(unfollow.community_view.community.subscribers_local).toBe(
    community?.community.subscribers_local,
  );
});

test("Follow federated community", async () => {
  let user = await registerUser(alpha, alphaUrl);

  const betaCommunityInitial = await waitUntil(
    () => resolveBetaCommunity(user),
    c => !!c?.community && c.community.subscribers >= 1,
  );
  if (!betaCommunityInitial) {
    throw "Missing beta community";
  }

  let follow = await followCommunity(
    user,
    true,
    betaCommunityInitial.community.id,
  );
  expect(follow.community_view.community_actions?.follow_state).toBe("pending");
  const betaCommunity = await waitUntil(
    () => resolveBetaCommunity(user),
    c => c?.community_actions?.follow_state === "accepted",
  );

  // Make sure the follow response went through
  expect(betaCommunity?.community.local).toBe(false);
  expect(betaCommunity?.community.name).toBe("main");
  expect(betaCommunity?.community_actions?.follow_state).toBe("accepted");
  expect(betaCommunity?.community.subscribers_local).toBe(
    betaCommunityInitial.community.subscribers_local + 1,
  );

  // check that unfollow was federated
  let communityOnBeta1 = await resolveBetaCommunity(beta);
  expect(communityOnBeta1?.community.subscribers).toBe(
    betaCommunityInitial.community.subscribers + 1,
  );

  // Check it from local
  let my_user = await getMyUser(user);
  let remoteCommunityId = my_user?.follows.find(
    c =>
      c.community.local == false &&
      c.community.id === betaCommunityInitial.community.id,
  )?.community.id;
  expect(remoteCommunityId).toBeDefined();

  if (!remoteCommunityId) {
    throw "Missing remote community id";
  }

  // Test an unfollow
  let unfollow = await followCommunity(user, false, remoteCommunityId);
  expect(
    unfollow.community_view.community_actions?.follow_state,
  ).toBeUndefined();

  // Make sure you are unsubbed locally
  let siteUnfollowCheck = await getMyUser(user);
  expect(
    siteUnfollowCheck.follows.find(
      c => c.community.id === betaCommunityInitial.community.id,
    ),
  ).toBe(undefined);

  // check that unfollow was federated
  let communityOnBeta2 = await waitUntil(
    () => resolveBetaCommunity(beta),
    c =>
      c?.community.subscribers === betaCommunityInitial.community.subscribers,
  );
  expect(communityOnBeta2?.community.subscribers).toBe(
    betaCommunityInitial.community.subscribers,
  );
  expect(communityOnBeta2?.community.subscribers_local).toBe(1);
});
