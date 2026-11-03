jest.setTimeout(120000);

import {
  AddModToCommunity,
  Community,
  FollowCommunity,
  LemmyError,
  LemmyHttp,
} from "lemmy-js-client";
import {
  alpha,
  alphaUrl,
  setupLogins,
  createCommunity,
  unfollows,
  registerUser,
  listCommunityPendingFollows,
  getCommunity,
  approveCommunityPendingFollow,
  randomString,
  createPost,
  createComment,
  beta,
  resolveCommunity,
  resolvePerson,
  betaUrl,
  resolvePost,
  resolveComment,
  likeComment,
  waitUntil,
  gamma,
  getPosts,
  getComments,
  statusNotFound,
  jestLemmyError,
  statusBadRequest,
  getUnreadCounts,
  expectSuccess,
  waitUntilSuccess,
  resolvePostFailure,
  resolveCommentFailure,
  expectFailure,
  followCommunity,
} from "./shared";

beforeAll(setupLogins);
afterAll(unfollows);

test("Follow a private community", async () => {
  // create private community
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  expect(community.community_view.community.visibility).toBe("private");
  const alphaCommunityId = community.community_view.community.id;

  // No pending follows yet
  const pendingFollows0 =
    await listCommunityPendingFollows(alpha).then(expectSuccess);
  expect(pendingFollows0.items.length).toBe(0);
  const pendingFollowsCount0 = await getUnreadCounts(alpha).then(expectSuccess);
  expect(pendingFollowsCount0.pending_follow_count).toBe(0);

  // follow as new user
  const user = await registerUser(beta, betaUrl);
  const betaCommunity = await resolveCommunity(
    user,
    community.community_view.community.ap_id,
  );
  expect(betaCommunity).toBeDefined();
  expect(betaCommunity?.community.visibility).toBe("private");
  const betaCommunityId = betaCommunity!.community.id;
  const follow_form: FollowCommunity = {
    community_id: betaCommunityId,
    follow: true,
  };
  await user.followCommunity(follow_form);

  // Follow listed as pending
  const follow1 = await getCommunity(user, betaCommunityId).then(expectSuccess);
  expect(follow1.community_view.community_actions?.follow_state).toBe(
    "approval_required",
  );

  // Wait for follow to federate, shown as pending
  const pendingFollows1 = await waitUntilSuccess(
    () => listCommunityPendingFollows(alpha),
    f => f.items.length == 1,
  );
  expect(pendingFollows1.items[0].is_new_instance).toBe(true);
  const pendingFollowsCount1 = await getUnreadCounts(alpha).then(expectSuccess);
  expect(pendingFollowsCount1.pending_follow_count).toBe(1);

  // user still sees approval required at this point
  const betaCommunity2 = await getCommunity(user, betaCommunityId).then(
    expectSuccess,
  );
  expect(betaCommunity2.community_view.community_actions?.follow_state).toBe(
    "approval_required",
  );

  // Approve the follow
  const approve = await approveCommunityPendingFollow(
    alpha,
    alphaCommunityId,
    pendingFollows1.items[0].person.id,
  ).then(expectSuccess);
  expect(approve.success).toBe(true);

  // Follow is confirmed
  await waitUntilSuccess(
    () => getCommunity(user, betaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );
  const pendingFollows2 =
    await listCommunityPendingFollows(alpha).then(expectSuccess);
  expect(pendingFollows2.items.length).toBe(0);
  const pendingFollowsCount2 = await getUnreadCounts(alpha).then(expectSuccess);
  expect(pendingFollowsCount2.pending_follow_count).toBe(0);

  // follow with another user from that instance, is_new_instance should be false now
  const user2 = await registerUser(beta, betaUrl);
  await user2.followCommunity(follow_form);
  const pendingFollows3 = await waitUntilSuccess(
    () => listCommunityPendingFollows(alpha),
    f => f.items.length == 1,
  );
  expect(pendingFollows3.items[0].is_new_instance).toBe(false);

  // cleanup pending follow
  const approve2 = await approveCommunityPendingFollow(
    alpha,
    alphaCommunityId,
    pendingFollows3.items[0].person.id,
  ).then(expectSuccess);
  expect(approve2.success).toBe(true);
});

test("Only followers can view and interact with private community content", async () => {
  // create private community
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  expect(community.community_view.community.visibility).toBe("private");
  const alphaCommunityId = community.community_view.community.id;

  // create post and comment
  const post0 = await createPost(alpha, alphaCommunityId).then(expectSuccess);
  const post_id = post0.post_view.post.id;
  expect(post_id).toBeDefined();
  const comment = await createComment(alpha, post_id).then(expectSuccess);
  const comment_id = comment.comment_view.comment.id;
  expect(comment_id).toBeDefined();

  // user is not following the community and cannot view nor create posts
  const user = await registerUser(beta, betaUrl);
  const betaCommunity = (
    await resolveCommunity(user, community.community_view.community.ap_id)
  )?.community;
  await jestLemmyError(
    () => resolvePostFailure(user, post0.post_view.post),
    new LemmyError("resolve_object_failed", statusBadRequest),
    false,
  );
  await jestLemmyError(
    () => resolveCommentFailure(user, comment.comment_view.comment),
    new LemmyError("resolve_object_failed", statusBadRequest),
    false,
  );
  await jestLemmyError(
    () => createPost(user, betaCommunity!.id).then(expectFailure),
    new LemmyError("not_found", statusNotFound),
  );

  // follow the community and approve
  const follow_form: FollowCommunity = {
    community_id: betaCommunity!.id,
    follow: true,
  };
  await user.followCommunity(follow_form);
  await approveFollower(alpha, alphaCommunityId);

  // now user can fetch posts and comments in community (using signed fetch), and create posts
  await waitUntil(
    () => resolvePost(user, post0.post_view.post),
    p => p?.post.id != undefined,
  );
  const resolvedComment = await resolveComment(
    user,
    comment.comment_view.comment,
  );
  expect(resolvedComment?.comment.id).toBeDefined();

  const post1 = await createPost(user, betaCommunity!.id).then(expectSuccess);
  expect(post1.post_view).toBeDefined();
  const like = await likeComment(user, true, resolvedComment!.comment).then(
    expectSuccess,
  );
  expect(like.comment_view.comment_actions?.vote_is_upvote).toBe(true);
});

test("Reject follower", async () => {
  // create private community
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  expect(community.community_view.community.visibility).toBe("private");
  const alphaCommunityId = community.community_view.community.id;

  // user is not following the community and cannot view nor create posts
  const user = await registerUser(beta, betaUrl);
  const betaCommunity1 = (
    await resolveCommunity(user, community.community_view.community.ap_id)
  )?.community;

  // follow the community and reject
  const follow_form: FollowCommunity = {
    community_id: betaCommunity1!.id,
    follow: true,
  };
  const follow = await user.followCommunity(follow_form).then(expectSuccess);
  expect(follow.community_view.community_actions?.follow_state).toBe(
    "approval_required",
  );

  const pendingFollows1 = await waitUntilSuccess(
    () => listCommunityPendingFollows(alpha),
    f => f.items.length == 1,
  );
  const approve = await approveCommunityPendingFollow(
    alpha,
    alphaCommunityId,
    pendingFollows1.items[0].person.id,
    false,
  ).then(expectSuccess);
  expect(approve.success).toBe(true);

  await waitUntilSuccess(
    () => getCommunity(user, betaCommunity1!.id),
    c => c.community_view.community_actions?.follow_state === undefined,
  );
});

test("Follow a private community and receive activities", async () => {
  // create private community
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  expect(community.community_view.community.visibility).toBe("private");
  const alphaCommunityId = community.community_view.community.id;

  // follow with users from beta and gamma
  const betaCommunity = await resolveCommunity(
    beta,
    community.community_view.community.ap_id,
  );
  expect(betaCommunity).toBeDefined();
  const betaCommunityId = betaCommunity!.community.id;
  const follow_form_beta: FollowCommunity = {
    community_id: betaCommunityId,
    follow: true,
  };
  await beta.followCommunity(follow_form_beta);
  await approveFollower(alpha, alphaCommunityId);

  const gammaCommunityId = (await resolveCommunity(
    gamma,
    community.community_view.community.ap_id,
  ))!.community.id;
  const follow_form_gamma: FollowCommunity = {
    community_id: gammaCommunityId,
    follow: true,
  };
  await gamma.followCommunity(follow_form_gamma);
  await approveFollower(alpha, alphaCommunityId);

  // Follow is confirmed
  await waitUntilSuccess(
    () => getCommunity(beta, betaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );
  await waitUntilSuccess(
    () => getCommunity(gamma, gammaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );

  // create a post and comment from gamma
  const post = await createPost(gamma, gammaCommunityId).then(expectSuccess);
  const post_id = post.post_view.post.id;
  expect(post_id).toBeDefined();
  const comment = await createComment(gamma, post_id).then(expectSuccess);
  const comment_id = comment.comment_view.comment.id;
  expect(comment_id).toBeDefined();

  // post and comment were federated to beta
  const posts = await waitUntilSuccess(
    () => getPosts(beta, "all", betaCommunityId),
    c => c.items.length == 1,
  );
  expect(posts.items[0].post.ap_id).toBe(post.post_view.post.ap_id);
  expect(posts.items[0].post.name).toBe(post.post_view.post.name);
  const comments = await waitUntilSuccess(
    () => getComments(beta, posts.items[0].post.id),
    c => c.items.length == 1,
  );
  expect(comments.items[0].comment.ap_id).toBe(
    comment.comment_view.comment.ap_id,
  );
  expect(comments.items[0].comment.content).toBe(
    comment.comment_view.comment.content,
  );
});

test("Fetch remote content in private community", async () => {
  // create private community
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  expect(community.community_view.community.visibility).toBe("private");
  const alphaCommunityId = community.community_view.community.id;

  const betaCommunityId = (await resolveCommunity(
    beta,
    community.community_view.community.ap_id,
  ))!.community.id;
  const follow_form_beta: FollowCommunity = {
    community_id: betaCommunityId,
    follow: true,
  };
  await beta.followCommunity(follow_form_beta);
  await approveFollower(alpha, alphaCommunityId);

  // Follow is confirmed
  await waitUntilSuccess(
    () => getCommunity(beta, betaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );

  // beta creates post and comment
  const post = await createPost(beta, betaCommunityId).then(expectSuccess);
  const post_id = post.post_view.post.id;
  expect(post_id).toBeDefined();
  const comment = await createComment(beta, post_id).then(expectSuccess);
  const comment_id = comment.comment_view.comment.id;
  expect(comment_id).toBeDefined();

  // Wait for it to federate
  await waitUntil(
    () => resolveComment(alpha, comment.comment_view.comment),
    p => p?.comment.id != undefined,
  );

  // create gamma user
  const gammaCommunityId = (await resolveCommunity(
    gamma,
    community.community_view.community.ap_id,
  ))!.community.id;
  const follow_form: FollowCommunity = {
    community_id: gammaCommunityId,
    follow: true,
  };

  // cannot fetch post yet
  await jestLemmyError(
    () => resolvePostFailure(gamma, post.post_view.post),
    new LemmyError("resolve_object_failed", statusBadRequest),
    false,
  );
  // follow community and approve
  await gamma.followCommunity(follow_form);
  await approveFollower(alpha, alphaCommunityId);

  // now user can fetch posts and comments in community (using signed fetch), and create posts.
  // for this to work, beta checks with alpha if gamma is really an approved follower.
  const resolvedPost = await waitUntil(
    () => resolvePost(gamma, post.post_view.post),
    p => p?.post.id != undefined,
  );
  expect(resolvedPost?.post.ap_id).toBe(post.post_view.post.ap_id);
  const resolvedComment = await waitUntil(
    () => resolveComment(gamma, comment.comment_view.comment),
    p => p?.comment.id != undefined,
  );
  expect(resolvedComment?.comment.ap_id).toBe(
    comment.comment_view.comment.ap_id,
  );
});

test("Pending follow of remote user is accepted when appointed mod by local mod", async () => {
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  const alphaCommunityId = community.community_view.community.id;

  const user = await registerUser(beta, betaUrl);
  const betaCommunityId = (await resolveCommunity(
    user,
    community.community_view.community.ap_id,
  ))!.community.id;
  await user.followCommunity({ community_id: betaCommunityId, follow: true });
  const pendingFollows = await waitUntilSuccess(
    () => listCommunityPendingFollows(alpha),
    f => f.items.length == 1,
  );

  const form: AddModToCommunity = {
    community_id: alphaCommunityId,
    person_id: pendingFollows.items[0].person.id,
    added: true,
  };
  await alpha.addModToCommunity(form).then(expectSuccess);

  // The accept is federated to the user's instance, and the follow is no longer pending
  await waitUntilSuccess(
    () => getCommunity(user, betaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );
  await expectNoPendingFollows();
});

test("Pending follow of local user is accepted when appointed mod by remote mod", async () => {
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  const alphaCommunityId = community.community_view.community.id;
  const betaCommunityId = await makeBetaMod(community.community_view.community);

  const user = await registerUser(alpha, alphaUrl);
  await user.followCommunity({ community_id: alphaCommunityId, follow: true });
  const pendingFollows = await waitUntilSuccess(
    () => listCommunityPendingFollows(alpha),
    f => f.items.length == 1,
  );

  const userOnBeta = await resolvePerson(
    beta,
    pendingFollows.items[0].person.ap_id,
  );
  const form: AddModToCommunity = {
    community_id: betaCommunityId,
    person_id: userOnBeta!.person.id,
    added: true,
  };
  await beta.addModToCommunity(form).then(expectSuccess);

  await waitUntilSuccess(
    () => getCommunity(user, alphaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );
  await expectNoPendingFollows();
});

test("Pending follow of remote user is accepted when appointed mod by remote mod", async () => {
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  const betaCommunityId = await makeBetaMod(community.community_view.community);

  const user = await registerUser(beta, betaUrl);
  await user.followCommunity({ community_id: betaCommunityId, follow: true });
  const pendingFollows = await waitUntilSuccess(
    () => listCommunityPendingFollows(alpha),
    f => f.items.length == 1,
  );

  const userOnBeta = await resolvePerson(
    beta,
    pendingFollows.items[0].person.ap_id,
  );
  const form: AddModToCommunity = {
    community_id: betaCommunityId,
    person_id: userOnBeta!.person.id,
    added: true,
  };
  await beta.addModToCommunity(form).then(expectSuccess);

  // The accept is federated back to the user's instance, and the follow is no longer pending
  await waitUntilSuccess(
    () => getCommunity(user, betaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );
  await expectNoPendingFollows();
});

test("Remote user is followed automatically when appointed mod by local mod", async () => {
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  const alphaCommunityId = community.community_view.community.id;

  // A follower on beta, so that beta accepts activities in the community
  const betaCommunityId = (await resolveCommunity(
    beta,
    community.community_view.community.ap_id,
  ))!.community.id;
  await beta.followCommunity({ community_id: betaCommunityId, follow: true });
  await approveFollower(alpha, alphaCommunityId);

  // The user never requests to follow
  const username = randomString(10);
  const user = await registerUser(beta, betaUrl, username);
  const userOnAlpha = await resolvePerson(alpha, `${username}@lemmy-beta:8551`);
  const form: AddModToCommunity = {
    community_id: alphaCommunityId,
    person_id: userOnAlpha!.person.id,
    added: true,
  };
  await alpha.addModToCommunity(form).then(expectSuccess);

  await waitUntilSuccess(
    () => getCommunity(user, betaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );
  await expectNoPendingFollows();
});

test("Local user is followed automatically when appointed mod by remote mod", async () => {
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  const alphaCommunityId = community.community_view.community.id;
  const betaCommunityId = await makeBetaMod(community.community_view.community);

  // The user never requests to follow
  const username = randomString(10);
  const user = await registerUser(alpha, alphaUrl, username);
  const userOnBeta = await resolvePerson(
    beta,
    `http://lemmy-alpha:8541/u/${username}`,
  );
  const form: AddModToCommunity = {
    community_id: betaCommunityId,
    person_id: userOnBeta!.person.id,
    added: true,
  };
  await beta.addModToCommunity(form).then(expectSuccess);

  await waitUntilSuccess(
    () => getCommunity(user, alphaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );
  await expectNoPendingFollows();
});

test("Remote user is followed automatically when appointed mod by remote mod", async () => {
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  const betaCommunityId = await makeBetaMod(community.community_view.community);
  const user = await appointBetaUserAsMod(betaCommunityId);

  await waitUntilSuccess(
    () => getCommunity(user, betaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );

  // The community only accepts posts from approved followers, so this shows that alpha also has
  // the new mod as follower
  const post = await createPost(user, betaCommunityId).then(expectSuccess);
  await waitUntil(
    () => resolvePost(alpha, post.post_view.post),
    p => p?.post.id != undefined,
  );
  await expectNoPendingFollows();
});

test("Mod who unfollows and follows again is accepted without approval", async () => {
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  const betaCommunityId = await makeBetaMod(community.community_view.community);
  const user = await appointBetaUserAsMod(betaCommunityId);
  await waitUntilSuccess(
    () => getCommunity(user, betaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );

  await followCommunity(user, false, betaCommunityId);
  await followCommunity(user, true, betaCommunityId);
  await expectNoPendingFollows();
});

test("Remote user is not followed on their instance if it has no follower of the community", async () => {
  const community = await createCommunity(
    alpha,
    randomString(10),
    "private",
  ).then(expectSuccess);
  const alphaCommunityId = community.community_view.community.id;

  // Nobody on beta follows the community, and it has no local posts or comments there
  const username = randomString(10);
  const user = await registerUser(beta, betaUrl, username);
  const userOnAlpha = await resolvePerson(alpha, `${username}@lemmy-beta:8551`);
  const form: AddModToCommunity = {
    community_id: alphaCommunityId,
    person_id: userOnAlpha!.person.id,
    added: true,
  };
  await alpha.addModToCommunity(form).then(expectSuccess);

  // Give the Add time to arrive, and be rejected
  await new Promise(resolve => setTimeout(resolve, 10000));

  const betaCommunityId = (await resolveCommunity(
    user,
    community.community_view.community.ap_id,
  ))!.community.id;
  const betaCommunity = await getCommunity(user, betaCommunityId).then(
    expectSuccess,
  );
  console.log(
    "follow_state on beta:",
    betaCommunity.community_view.community_actions?.follow_state,
    "moderators on beta:",
    betaCommunity.moderators.map(m => m.moderator.name),
  );
  expect(
    betaCommunity.community_view.community_actions?.follow_state,
  ).toBeUndefined();

  // alpha has the new mod as follower regardless
  await expectNoPendingFollows();
});

// Appoints a new beta user as mod, without the user ever following the community.
async function appointBetaUserAsMod(
  betaCommunityId: number,
): Promise<LemmyHttp> {
  const username = randomString(10);
  const user = await registerUser(beta, betaUrl, username);
  const userOnBeta = await resolvePerson(
    beta,
    `http://lemmy-beta:8551/u/${username}`,
  );
  const form: AddModToCommunity = {
    community_id: betaCommunityId,
    person_id: userOnBeta!.person.id,
    added: true,
  };
  await beta.addModToCommunity(form).then(expectSuccess);
  return user;
}

// Makes the beta admin a mod of the alpha community, and returns the community id on beta.
async function makeBetaMod(community: Community): Promise<number> {
  const betaCommunityId = (await resolveCommunity(beta, community.ap_id))!
    .community.id;
  await beta.followCommunity({ community_id: betaCommunityId, follow: true });
  await approveFollower(alpha, community.id);
  await waitUntilSuccess(
    () => getCommunity(beta, betaCommunityId),
    c => c.community_view.community_actions?.follow_state == "accepted",
  );

  const betaOnAlpha = await resolvePerson(alpha, "lemmy_beta@lemmy-beta:8551");
  const form: AddModToCommunity = {
    community_id: community.id,
    person_id: betaOnAlpha!.person.id,
    added: true,
  };
  await alpha.addModToCommunity(form).then(expectSuccess);
  await waitUntilSuccess(
    () => getCommunity(beta, betaCommunityId),
    c => c.moderators.length == 2,
  );
  return betaCommunityId;
}

async function expectNoPendingFollows() {
  const pendingFollows =
    await listCommunityPendingFollows(alpha).then(expectSuccess);
  expect(pendingFollows.items.length).toBe(0);
  const counts = await getUnreadCounts(alpha).then(expectSuccess);
  expect(counts.pending_follow_count).toBe(0);
}

async function approveFollower(user: LemmyHttp, community_id: number) {
  const pendingFollows1 = await waitUntilSuccess(
    () => listCommunityPendingFollows(user),
    f => f.items.length == 1,
  );
  const approve = await approveCommunityPendingFollow(
    alpha,
    community_id,
    pendingFollows1.items[0].person.id,
  ).then(expectSuccess);
  expect(approve.success).toBe(true);
}
