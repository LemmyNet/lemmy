jest.setTimeout(120000);

import { FollowCommunity } from "lemmy-js-client";
import {
  alpha,
  setupLogins,
  createCommunity,
  unfollows,
  registerUser,
  listCommunityPendingFollows,
  getCommunity,
  getCommunityPendingFollowsCount,
  approveCommunityPendingFollow,
  randomString,
  createPost,
  createComment,
  beta,
  resolveCommunity,
  betaUrl,
  delay,
  resolvePost,
  resolveComment,
  likeComment,
} from "./shared";

beforeAll(setupLogins);
afterAll(unfollows);

test("Follow a private community", async () => {
  // create private community
  const community = await createCommunity(alpha, randomString(10), "Private");
  expect(community.community_view.community.visibility).toBe("Private");
  const alphaCommunityId = community.community_view.community.id;

  // No pending follows yet
  const pendingFollows0 = await listCommunityPendingFollows(
    alpha,
    alphaCommunityId,
  );
  expect(pendingFollows0.items.length).toBe(0);
  const pendingFollowsCount0 = await getCommunityPendingFollowsCount(
    alpha,
    alphaCommunityId,
  );
  expect(pendingFollowsCount0.count).toBe(0);

  // follow as new user
  const user = await registerUser(beta, betaUrl);
  const betaCommunity = (
    await resolveCommunity(user, community.community_view.community.actor_id)
  ).community;
  expect(betaCommunity).toBeDefined();
  const betaCommunityId = betaCommunity!.community.id;
  const follow_form: FollowCommunity = {
    community_id: betaCommunityId,
    follow: true,
  };
  await user.followCommunity(follow_form);

  // Follow listed as pending
  const follow1 = getCommunity(user, betaCommunityId);
  expect((await follow1).community_view.subscribed).toBe("ApprovalRequired");

  // Wait for follow to federate, shown as pending
  await delay(1000);
  const pendingFollows1 = await listCommunityPendingFollows(
    alpha,
    alphaCommunityId,
  );
  expect(pendingFollows1.items.length).toBe(1);
  const pendingFollowsCount1 = await getCommunityPendingFollowsCount(
    alpha,
    alphaCommunityId,
  );
  expect(pendingFollowsCount1.count).toBe(1);

  // user still sees approval required at this point
  const betaCommunity2 = await getCommunity(user, betaCommunityId);
  expect(betaCommunity2.community_view.subscribed).toBe("ApprovalRequired");

  // Approve the follow
  const approve = await approveCommunityPendingFollow(
    alpha,
    alphaCommunityId,
    pendingFollows1.items[0].id,
  );
  expect(approve.success).toBe(true);

  // Follow is confirmed
  await delay(1000);
  const follow2 = getCommunity(user, betaCommunityId);
  expect((await follow2).community_view.subscribed).toBe("Subscribed");
  const pendingFollows2 = await listCommunityPendingFollows(
    alpha,
    alphaCommunityId,
  );
  expect(pendingFollows2.items.length).toBe(0);
  const pendingFollowsCount2 = await getCommunityPendingFollowsCount(
    alpha,
    alphaCommunityId,
  );
  expect(pendingFollowsCount2.count).toBe(0);
});

test("Only followers can view and interact with private community content", async () => {
  // create private community
  const community = await createCommunity(alpha, randomString(10), "Private");
  expect(community.community_view.community.visibility).toBe("Private");
  const alphaCommunityId = community.community_view.community.id;

  // create post and comment
  const post0 = await createPost(alpha, alphaCommunityId);
  const post_id = post0.post_view.post.id;
  expect(post_id).toBeDefined();
  const comment = await createComment(alpha, post_id);
  const comment_id = comment.comment_view.comment.id;
  expect(comment_id).toBeDefined();

  // user is not following the community and cannot view nor create posts
  const user = await registerUser(beta, betaUrl);
  const betaCommunity = (
    await resolveCommunity(user, community.community_view.community.actor_id)
  ).community!.community;
  await expect(resolvePost(user, post0.post_view.post)).rejects.toStrictEqual(
    Error("not_found"),
  );
  await expect(
    resolveComment(user, comment.comment_view.comment),
  ).rejects.toStrictEqual(Error("not_found"));
  await expect(createPost(user, betaCommunity.id)).rejects.toStrictEqual(
    Error("not_found"),
  );

  // follow the community and approve
  const follow_form: FollowCommunity = {
    community_id: betaCommunity.id,
    follow: true,
  };
  await user.followCommunity(follow_form);
  await delay(1000);
  const pendingFollows1 = await listCommunityPendingFollows(
    alpha,
    alphaCommunityId,
  );
  expect(pendingFollows1.items.length).toBe(1);
  const approve = await approveCommunityPendingFollow(
    alpha,
    alphaCommunityId,
    pendingFollows1.items[0].id,
  );
  expect(approve.success).toBe(true);

  // now user can fetch posts and comments in community (using signed fetch), and create posts
  await delay(1000);
  const resolvedPost = (await resolvePost(user, post0.post_view.post)).post;
  expect(resolvedPost?.post.id).toBeDefined();
  const resolvedComment = (
    await resolveComment(user, comment.comment_view.comment)
  ).comment;
  expect(resolvedComment?.comment.id).toBeDefined();

  const post1 = await createPost(user, betaCommunity.id);
  expect(post1.post_view).toBeDefined();
  const like = await likeComment(user, 1, resolvedComment!.comment);
  expect(like.comment_view.my_vote).toBe(1);
});

test("Reject follower", async () => {
  // create private community
  const community = await createCommunity(alpha, randomString(10), "Private");
  expect(community.community_view.community.visibility).toBe("Private");
  const alphaCommunityId = community.community_view.community.id;

  // user is not following the community and cannot view nor create posts
  const user = await registerUser(beta, betaUrl);
  const betaCommunity1 = (
    await resolveCommunity(user, community.community_view.community.actor_id)
  ).community!.community;

  // follow the community and reject
  const follow_form: FollowCommunity = {
    community_id: betaCommunity1.id,
    follow: true,
  };
  const follow = await user.followCommunity(follow_form);
  expect(follow.community_view.subscribed).toBe("ApprovalRequired");

  await delay(1000);
  const pendingFollows1 = await listCommunityPendingFollows(
    alpha,
    alphaCommunityId,
  );
  expect(pendingFollows1.items.length).toBe(1);
  const approve = await approveCommunityPendingFollow(
    alpha,
    alphaCommunityId,
    pendingFollows1.items[0].id,
    false,
  );
  expect(approve.success).toBe(true);
  await delay(1000);

  const betaCommunity2 = await getCommunity(user, betaCommunity1.id);
  expect(betaCommunity2.community_view.subscribed).toBe("NotSubscribed");
});
