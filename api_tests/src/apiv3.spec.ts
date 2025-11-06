jest.setTimeout(180000);

import {
  LemmyHttp,
  Login,
  CreatePost,
  ResolveObject,
} from "lemmy-js-client-019";
import { beta, betaUrl, setupLogins, unfollows } from "./shared";
import { CreateComment } from "lemmy-js-client";

beforeAll(async () => {
  await setupLogins();
});

afterAll(unfollows);

test("API v3", async () => {
  let login_form: Login = {
    username_or_email: "lemmy_beta",
    password: "lemmylemmy",
  };
  const login = await beta.login(login_form);
  expect(login.jwt).toBeDefined();

  let user = new LemmyHttp(betaUrl, {
    headers: { Authorization: `Bearer ${login.jwt ?? ""}` },
  });

  let resolve_form: ResolveObject = {
    q: "!main@lemmy-beta:8551",
  };
  const community = await user
    .resolveObject(resolve_form)
    .then(a => a.community);
  expect(community?.community).toBeDefined();

  const post_form: CreatePost = {
    name: "post from api v3",
    community_id: community!.community.id,
  };
  const post = await user.createPost(post_form);
  expect(post.post_view.post).toBeDefined();
  const post_id = post.post_view.post.id;

  const post_listing = await user.getPosts();
  expect(
    post_listing.posts.find(p => {
      return p.post.id === post_id;
    })?.post,
  ).toStrictEqual(post.post_view.post);

  const comment_form: CreateComment = {
    content: "comment from api v3",
    post_id,
  };
  const comment = await user.createComment(comment_form);
  expect(comment.comment_view.comment).toBeDefined();

  const comment_listing = await user.getComments({ post_id });
  expect(comment_listing.comments[0].comment).toStrictEqual(
    comment.comment_view.comment,
  );
});
