jest.setTimeout(120000);

import { FollowCommunity, LemmyHttp } from "lemmy-js-client";
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
  resolvePost,
  resolveComment,
  likeComment,
  waitUntil,
  gamma,
  getPosts,
  getComments,
} from "./shared";

beforeAll(setupLogins);
afterAll(unfollows);

test("Follow a private community", async () => {
  // create private community
  const community = await createCommunity(alpha, randomString(10), "Private");
  expect(community.community_view.community.visibility).toBe("Private");
  const alphaCommunityId = community.community_view.community.id;

  // No pending follows yet
  const pendingFollows0 = await listCommunityPendingFollows(alpha);
  expect(pendingFollows0.items.length).toBe(0);
  const pendingFollowsCount0 = await getCommunityPendingFollowsCount(
    alpha,
    alphaCommunityId,
  );
  expect(pendingFollowsCount0.count).toBe(0);

  // follow as new user
  const user = await registerUser(beta, betaUrl);
  const betaCommunity = (
    await resolveCommunity(user, community.community_view.community.ap_id)
  ).community;
  expect(betaCommunity).toBeDefined();
  expect(betaCommunity?.community.visibility).toBe("Private");
  const betaCommunityId = betaCommunity!.community.id;
  const follow_form: FollowCommunity = {
    community_id: betaCommunityId,
    follow: true,
  };
  await user.followCommunity(follow_form);

  // Follow listed as pending
  const follow1 = await getCommunity(user, betaCommunityId);
  expect(follow1.community_view.subscribed).toBe("ApprovalRequired");

  // Wait for follow to federate, shown as pending
  let pendingFollows1 = await waitUntil(
    () => listCommunityPendingFollows(alpha),
    f => f.items.length == 1,
  );
  expect(pendingFollows1.items[0].is_new_instance).toBe(true);
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
    pendingFollows1.items[0].person.id,
  );
  expect(approve.success).toBe(true);

  // Follow is confirmed
  await waitUntil(
    () => getCommunity(user, betaCommunityId),
    c => c.community_view.subscribed == "Subscribed",
  );
  const pendingFollows2 = await listCommunityPendingFollows(alpha);
  expect(pendingFollows2.items.length).toBe(0);
  const pendingFollowsCount2 = await getCommunityPendingFollowsCount(
    alpha,
    alphaCommunityId,
  );
  expect(pendingFollowsCount2.count).toBe(0);

  // follow with another user from that instance, is_new_instance should be false now
  const user2 = await registerUser(beta, betaUrl);
  await user2.followCommunity(follow_form);
  let pendingFollows3 = await waitUntil(
    () => listCommunityPendingFollows(alpha),
    f => f.items.length == 1,
  );
  expect(pendingFollows3.items[0].is_new_instance).toBe(false);

  // cleanup pending follow
  const approve2 = await approveCommunityPendingFollow(
    alpha,
    alphaCommunityId,
    pendingFollows3.items[0].person.id,
  );
  expect(approve2.success).toBe(true);
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
    await resolveCommunity(user, community.community_view.community.ap_id)
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
  approveFollower(alpha, alphaCommunityId);

  // now user can fetch posts and comments in community (using signed fetch), and create posts
  await waitUntil(
    () => resolvePost(user, post0.post_view.post),
    p => p?.post?.post.id != undefined,
  );
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
    await resolveCommunity(user, community.community_view.community.ap_id)
  ).community!.community;

  // follow the community and reject
  const follow_form: FollowCommunity = {
    community_id: betaCommunity1.id,
    follow: true,
  };
  const follow = await user.followCommunity(follow_form);
  expect(follow.community_view.subscribed).toBe("ApprovalRequired");

  const pendingFollows1 = await waitUntil(
    () => listCommunityPendingFollows(alpha),
    f => f.items.length == 1,
  );
  const approve = await approveCommunityPendingFollow(
    alpha,
    alphaCommunityId,
    pendingFollows1.items[0].person.id,
    false,
  );
  expect(approve.success).toBe(true);

  await waitUntil(
    () => getCommunity(user, betaCommunity1.id),
    c => c.community_view.subscribed == "NotSubscribed",
  );
});

test("Follow a private community and receive activities", async () => {
  // create private community
  const community = await createCommunity(alpha, randomString(10), "Private");
  expect(community.community_view.community.visibility).toBe("Private");
  const alphaCommunityId = community.community_view.community.id;

  // follow with users from beta and gamma
  const betaCommunity = (
    await resolveCommunity(beta, community.community_view.community.ap_id)
  ).community;
  expect(betaCommunity).toBeDefined();
  const betaCommunityId = betaCommunity!.community.id;
  const follow_form_beta: FollowCommunity = {
    community_id: betaCommunityId,
    follow: true,
  };
  await beta.followCommunity(follow_form_beta);
  await approveFollower(alpha, alphaCommunityId);

  const gammaCommunityId = (
    await resolveCommunity(gamma, community.community_view.community.ap_id)
  ).community!.community.id;
  const follow_form_gamma: FollowCommunity = {
    community_id: gammaCommunityId,
    follow: true,
  };
  await gamma.followCommunity(follow_form_gamma);
  await approveFollower(alpha, alphaCommunityId);

  // Follow is confirmed
  await waitUntil(
    () => getCommunity(beta, betaCommunityId),
    c => c.community_view.subscribed == "Subscribed",
  );
  await waitUntil(
    () => getCommunity(gamma, gammaCommunityId),
    c => c.community_view.subscribed == "Subscribed",
  );

  // create a post and comment from gamma
  const post = await createPost(gamma, gammaCommunityId);
  const post_id = post.post_view.post.id;
  expect(post_id).toBeDefined();
  const comment = await createComment(gamma, post_id);
  const comment_id = comment.comment_view.comment.id;
  expect(comment_id).toBeDefined();

  // post and comment were federated to beta
  let posts = await waitUntil(
    () => getPosts(beta, "All", betaCommunityId),
    c => c.posts.length == 1,
  );
  expect(posts.posts[0].post.ap_id).toBe(post.post_view.post.ap_id);
  expect(posts.posts[0].post.name).toBe(post.post_view.post.name);
  let comments = await waitUntil(
    () => getComments(beta, posts.posts[0].post.id),
    c => c.comments.length == 1,
  );
  expect(comments.comments[0].comment.ap_id).toBe(
    comment.comment_view.comment.ap_id,
  );
  expect(comments.comments[0].comment.content).toBe(
    comment.comment_view.comment.content,
  );
});

test("Fetch remote content in private community", async () => {
  // create private community
  const community = await createCommunity(alpha, randomString(10), "Private");
  expect(community.community_view.community.visibility).toBe("Private");
  const alphaCommunityId = community.community_view.community.id;

  const betaCommunityId = (
    await resolveCommunity(beta, community.community_view.community.ap_id)
  ).community!.community.id;
  const follow_form_beta: FollowCommunity = {
    community_id: betaCommunityId,
    follow: true,
  };
  await beta.followCommunity(follow_form_beta);
  await approveFollower(alpha, alphaCommunityId);

  // Follow is confirmed
  await waitUntil(
    () => getCommunity(beta, betaCommunityId),
    c => c.community_view.subscribed == "Subscribed",
  );

  // beta creates post and comment
  const post = await createPost(beta, betaCommunityId);
  const post_id = post.post_view.post.id;
  expect(post_id).toBeDefined();
  const comment = await createComment(beta, post_id);
  const comment_id = comment.comment_view.comment.id;
  expect(comment_id).toBeDefined();

  // Wait for it to federate
  await waitUntil(
    () => resolveComment(alpha, comment.comment_view.comment),
    p => p?.comment?.comment.id != undefined,
  );

  // create gamma user
  const gammaCommunityId = (
    await resolveCommunity(gamma, community.community_view.community.ap_id)
  ).community!.community.id;
  const follow_form: FollowCommunity = {
    community_id: gammaCommunityId,
    follow: true,
  };

  // cannot fetch post yet
  await expect(resolvePost(gamma, post.post_view.post)).rejects.toStrictEqual(
    Error("not_found"),
  );
  // follow community and approve
  await gamma.followCommunity(follow_form);
  await approveFollower(alpha, alphaCommunityId);

  // now user can fetch posts and comments in community (using signed fetch), and create posts.
  // for this to work, beta checks with alpha if gamma is really an approved follower.
  let resolvedPost = await waitUntil(
    () => resolvePost(gamma, post.post_view.post),
    p => p?.post?.post.id != undefined,
  );
  expect(resolvedPost.post?.post.ap_id).toBe(post.post_view.post.ap_id);
  const resolvedComment = await waitUntil(
    () => resolveComment(gamma, comment.comment_view.comment),
    p => p?.comment?.comment.id != undefined,
  );
  expect(resolvedComment?.comment?.comment.ap_id).toBe(
    comment.comment_view.comment.ap_id,
  );
});

async function approveFollower(user: LemmyHttp, community_id: number) {
  let pendingFollows1 = await waitUntil(
    () => listCommunityPendingFollows(user),
    f => f.items.length == 1,
  );
  const approve = await approveCommunityPendingFollow(
    alpha,
    community_id,
    pendingFollows1.items[0].person.id,
  );
  expect(approve.success).toBe(true);
}
