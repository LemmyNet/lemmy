jest.setTimeout(120000);

import { PersonView } from "lemmy-js-client/dist/types/PersonView";
import {
  alpha,
  beta,
  registerUser,
  resolvePerson,
  getSite,
  createPost,
  resolveCommunity,
  createComment,
  resolveBetaCommunity,
  deleteUser,
  saveUserSettingsFederated,
  setupLogins,
  alphaUrl,
  saveUserSettings,
  getPost,
  getComments,
  fetchFunction,
} from "./shared";
import { LemmyHttp, SaveUserSettings } from "lemmy-js-client";
import { GetPosts } from "lemmy-js-client/dist/types/GetPosts";

beforeAll(setupLogins);

let apShortname: string;

function assertUserFederation(userOne?: PersonView, userTwo?: PersonView) {
  expect(userOne?.person.name).toBe(userTwo?.person.name);
  expect(userOne?.person.display_name).toBe(userTwo?.person.display_name);
  expect(userOne?.person.bio).toBe(userTwo?.person.bio);
  expect(userOne?.person.actor_id).toBe(userTwo?.person.actor_id);
  expect(userOne?.person.avatar).toBe(userTwo?.person.avatar);
  expect(userOne?.person.banner).toBe(userTwo?.person.banner);
  expect(userOne?.person.published).toBe(userTwo?.person.published);
}

test("Create user", async () => {
  let user = await registerUser(alpha, alphaUrl);

  let site = await getSite(user);
  expect(site.my_user).toBeDefined();
  if (!site.my_user) {
    throw "Missing site user";
  }
  apShortname = `@${site.my_user.local_user_view.person.name}@lemmy-alpha:8541`;
});

test("Set some user settings, check that they are federated", async () => {
  await saveUserSettingsFederated(alpha);
  let alphaPerson = (await resolvePerson(alpha, apShortname)).person;
  let betaPerson = (await resolvePerson(beta, apShortname)).person;
  assertUserFederation(alphaPerson, betaPerson);

  // Catches a bug where when only the person or local_user changed
  let form: SaveUserSettings = {
    theme: "test",
  };
  await saveUserSettings(beta, form);

  let site = await getSite(beta);
  expect(site.my_user?.local_user_view.local_user.theme).toBe("test");
});

test("Delete user", async () => {
  let user = await registerUser(alpha, alphaUrl);

  // make a local post and comment
  let alphaCommunity = (await resolveCommunity(user, "!main@lemmy-alpha:8541"))
    .community;
  if (!alphaCommunity) {
    throw "Missing alpha community";
  }
  let localPost = (await createPost(user, alphaCommunity.community.id))
    .post_view.post;
  expect(localPost).toBeDefined();
  let localComment = (await createComment(user, localPost.id)).comment_view
    .comment;
  expect(localComment).toBeDefined();

  // make a remote post and comment
  let betaCommunity = (await resolveBetaCommunity(user)).community;
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let remotePost = (await createPost(user, betaCommunity.community.id))
    .post_view.post;
  expect(remotePost).toBeDefined();
  let remoteComment = (await createComment(user, remotePost.id)).comment_view
    .comment;
  expect(remoteComment).toBeDefined();

  await deleteUser(user);

  // check that posts and comments are marked as deleted on other instances.
  // use get methods to avoid refetching from origin instance
  expect((await getPost(alpha, localPost.id)).post_view.post.deleted).toBe(
    true,
  );
  expect((await getPost(alpha, remotePost.id)).post_view.post.deleted).toBe(
    true,
  );
  expect(
    (await getComments(alpha, localComment.post_id)).comments[0].comment
      .deleted,
  ).toBe(true);
  expect(
    (await getComments(alpha, remoteComment.post_id)).comments[0].comment
      .deleted,
  ).toBe(true);
});

test("Requests with invalid auth should be treated as unauthenticated", async () => {
  let invalid_auth = new LemmyHttp(alphaUrl, {
    headers: { Authorization: "Bearer foobar" },
    fetchFunction,
  });
  let site = await getSite(invalid_auth);
  expect(site.my_user).toBeUndefined();
  expect(site.site_view).toBeDefined();

  let form: GetPosts = {};
  let posts = invalid_auth.getPosts(form);
  expect((await posts).posts).toBeDefined();
});

test("Create user with Arabic name", async () => {
  let user = await registerUser(alpha, alphaUrl, "تجريب");

  let site = await getSite(user);
  expect(site.my_user).toBeDefined();
  if (!site.my_user) {
    throw "Missing site user";
  }
  apShortname = `@${site.my_user.local_user_view.person.name}@lemmy-alpha:8541`;

  let alphaPerson = (await resolvePerson(alpha, apShortname)).person;
  expect(alphaPerson).toBeDefined();
});
