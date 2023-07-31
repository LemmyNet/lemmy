/*
With Lemmy 0.18.3 and earlier, performance has been a big concern.
Logging basic expectations of response times is the purpose of this module.
*/
jest.setTimeout(120 * 1000);

import { CreatePost, PostResponse } from "lemmy-js-client";
import {
  alpha,
  API,
  beta,
  createCommunity,
  followCommunity,
  resolveCommunity,
  setupLogins,
  createPost,
  createComment,
  likeComment,
  likePost,
  registerUser,
  getPosts,
  randomString,
} from "./shared";

beforeAll(async () => {
  await setupLogins();
});

afterAll(async () => {});

async function registerUserClient(withapi: API, username: string) {
  let registerUserRes = await registerUser(withapi, username);
  // this client being coppied from the other client, is that odd?
  let newUser: API = {
    client: withapi.client,
    auth: registerUserRes.jwt ?? "",
  };
  return newUser;
}

let alpha_user_casual0: API;

test("benchmark creating an account", async () => {
  alpha_user_casual0 = await registerUserClient(alpha, "alpha_casual0");
});

export async function createNoLinkPost(
  api: API,
  community_id: number,
): Promise<PostResponse> {
  let name = "Post without link " + randomString(5);
  let body = "Body of post without link " + randomString(10);
  let url = undefined;
  let form: CreatePost = {
    name,
    url,
    body,
    auth: api.auth,
    community_id,
  };
  return api.client.createPost(form);
}

// reference: https://stackoverflow.com/questions/58461792/timing-function-calls-in-jest
test("benchmark baseline, inserts: community, discovery, follow, post, comment, vote", async () => {
  let prevPost: PostResponse | undefined;
  let prevComment;

  const start = performance.now();

  // For the sake of woodpecker builds, only run 13 loops because these tests are slow
  // If performance improves,
  for (let i = 0; i < 13; i++) {
    const name = "series_" + i;
    let communityRes = await createCommunity(alpha, name);
    expect(communityRes.community_view.community.name).toBeDefined();

    // Cache the community on beta, make sure it has the other fields
    let searchShort = `!${name}@lemmy-alpha:8541`;
    let betaCommunity = (await resolveCommunity(beta, searchShort)).community;

    if (!betaCommunity) {
      throw "betaCommunity resolve failure";
    }
    await followCommunity(beta, true, betaCommunity.community.id);

    // NOTE: the default createPost is a URL post which does network connects outbound
    //   it is much slower to do url posts
    let postRes = await createNoLinkPost(
      alpha,
      communityRes.community_view.community.id,
    );
    let commentRes = await createComment(alpha, postRes.post_view.post.id);

    if (prevComment) {
      if (prevPost) {
        await createComment(
          alpha,
          prevPost?.post_view.post.id,
          prevComment.comment_view.comment.id,
          "reply to previous " + i,
        );
      }
    }

    // Other user upvotes.
    await likePost(alpha_user_casual0, 1, postRes.post_view.post);
    await likeComment(alpha_user_casual0, 1, commentRes.comment_view.comment);
    prevPost = postRes;
    prevComment = commentRes;
  }

  const end = performance.now();
  // 20 seconds is NOT good performance for 13 loops. I suggest 6 or even 1.3 seconds as a goal on empty database.
  expect(end - start).toBeLessThan(20 * 1000);
});

test("benchmark baseline, reading: list posts", async () => {
  const start = performance.now();

  for (let i = 0; i < 50; i++) {
    let posts = await getPosts(alpha);
    expect(posts.posts.length).toBeGreaterThanOrEqual(10);
  }

  const end = performance.now();
  expect(end - start).toBeLessThan(3 * 1000);
});
