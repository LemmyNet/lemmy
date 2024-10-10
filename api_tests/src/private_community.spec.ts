jest.setTimeout(120000);

import { CommunityId, FollowCommunity, LemmyHttp } from "lemmy-js-client";
import {
  alpha,
  setupLogins,
  createCommunity,
  unfollows,
  registerUser,
  alphaUrl,
  listCommunityPendingFollows,
  getCommunity,
  getCommunityPendingFollowsCount,
  approveCommunityPendingFollow,
  randomString,
  createPost,
  createComment,
  getPost,
  getComments,
  getPosts,
} from "./shared";

beforeAll(setupLogins);
afterAll(unfollows);

async function follow_and_approve(user: LemmyHttp, community_id: CommunityId) {
  // follow the community and approve
  const follow_form: FollowCommunity = {
    community_id,
    follow: true,
  };
  await user.followCommunity(follow_form);
  const pendingFollows = await listCommunityPendingFollows(alpha, community_id);
  const approve = await approveCommunityPendingFollow(
    alpha,
    community_id,
    pendingFollows.items[0].id,
  );
  expect(approve.success).toBe(true);
}

test("Follow a private community", async () => {
  // create private community
  const community = await createCommunity(alpha, randomString(10), "Private");
  expect(community.community_view.community.visibility).toBe("Private");
  const community_id = community.community_view.community.id;

  // No pending follows yet
  const pendingFollows0 = await listCommunityPendingFollows(
    alpha,
    community_id,
  );
  expect(pendingFollows0.items.length).toBe(0);
  const pendingFollowsCount0 = await getCommunityPendingFollowsCount(
    alpha,
    community_id,
  );
  expect(pendingFollowsCount0.count).toBe(0);

  // follow as new user
  const user = await registerUser(alpha, alphaUrl);
  const follow_form: FollowCommunity = {
    community_id,
    follow: true,
  };
  await user.followCommunity(follow_form);

  // Follow listed as pending
  const follow1 = getCommunity(user, community_id);
  expect((await follow1).community_view.subscribed).toBe("Pending");
  const pendingFollows1 = await listCommunityPendingFollows(
    alpha,
    community_id,
  );
  expect(pendingFollows1.items.length).toBe(1);
  const pendingFollowsCount1 = await getCommunityPendingFollowsCount(
    alpha,
    community_id,
  );
  expect(pendingFollowsCount1.count).toBe(1);

  // Approve the follow
  const approve = await approveCommunityPendingFollow(
    alpha,
    community_id,
    pendingFollows1.items[0].id,
  );
  expect(approve.success).toBe(true);

  // Follow is confirmed
  const follow2 = getCommunity(user, community_id);
  expect((await follow2).community_view.subscribed).toBe("Subscribed");
  const pendingFollows2 = await listCommunityPendingFollows(
    alpha,
    community_id,
  );
  expect(pendingFollows2.items.length).toBe(0);
  const pendingFollowsCount2 = await getCommunityPendingFollowsCount(
    alpha,
    community_id,
  );
  expect(pendingFollowsCount2.count).toBe(0);
});

test("Posts and comments in private community can only be seen by followers", async () => {
  // create private community
  const community = await createCommunity(alpha, randomString(10), "Private");
  expect(community.community_view.community.visibility).toBe("Private");
  const community_id = community.community_view.community.id;

  // create post and comment
  const post0 = await createPost(alpha, community_id);
  const post_id = post0.post_view.post.id;
  expect(post_id).toBeDefined();
  const comment = await createComment(alpha, post_id);
  const comment_id = comment.comment_view.comment.id;
  expect(comment_id).toBeDefined();

  // user is not following the community and cannot view its posts
  const user = await registerUser(alpha, alphaUrl);
  await expect(getPost(user, post_id)).rejects.toStrictEqual(
    Error("not_found"),
  );
  const comments1 = await getComments(user, post_id);
  expect(comments1.comments.length).toBe(0);
  const posts1 = await getPosts(user);
  expect(posts1.posts.length).toBe(0);

  // follow the community and approve
  await follow_and_approve(user, community_id);

  // now user can view posts and comments in community
  const post2 = await getPost(user, post_id);
  expect(post2.post_view.post.id).toBe(post_id);
  const comments2 = await getComments(user, post_id);
  expect(comments2.comments.length).toBe(1);
  expect(comments2.comments[0].comment.id).toBe(comment_id);
  const posts2 = await getPosts(user);
  expect(posts2.posts.length).toBe(1);
  expect(posts2.posts[0].post.id).toBe(post_id);
});

test("Only followers can post/comment in private community", async () => {
  // create private community and post
  const community = await createCommunity(alpha, randomString(10), "Private");
  expect(community.community_view.community.visibility).toBe("Private");
  const community_id = community.community_view.community.id;
  const post0 = await createPost(alpha, community_id);
  const post_id = post0.post_view.post.id;
  expect(post_id).toBeDefined();

  // user is not following the community and cannot post in it
  const user = await registerUser(alpha, alphaUrl);
  await expect(createPost(user, community_id)).rejects.toStrictEqual(
    Error("private_community"),
  );
  await expect(createComment(user, post_id)).rejects.toStrictEqual(
    Error("private_community"),
  );

  // make sure post and comment really were not created
  const posts1 = await getPosts(alpha, "All", community_id);
  expect(posts1.posts.length).toBe(1);
  const comments1 = await getComments(user, post_id);
  expect(comments1.comments.length).toBe(0);

  // follow the community, now user can post
  await follow_and_approve(user, community_id);
  const post2 = await createPost(user, community_id);
  expect(post2.post_view.post.id).toBeDefined();
  const comment2 = await createComment(user, post_id);
  expect(comment2.comment_view.comment.id).toBeDefined();
  const posts2 = await getPosts(alpha, "All", community_id);
  expect(posts2.posts.length).toBe(2);
  const comments2 = await getComments(user, post_id);
  expect(comments2.comments.length).toBe(1);
});
