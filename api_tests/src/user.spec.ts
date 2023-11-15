jest.setTimeout(120000);

import { PersonView } from "lemmy-js-client/dist/types/PersonView";
import {
  alpha,
  beta,
  gamma,
  registerUser,
  resolvePerson,
  getSite,
  createPost,
  resolveCommunity,
  createComment,
  resolveBetaCommunity,
  deleteUser,
  resolvePost,
  resolveComment,
  saveUserSettingsFederated,
  setupLogins,
  alphaUrl,
  saveUserSettings,
  getPost,
  getComments,
  followCommunity,
} from "./shared";
import { LemmyHttp, SaveUserSettings } from "lemmy-js-client";
import { GetPosts } from "lemmy-js-client/dist/types/GetPosts";

beforeAll(async () => {
  await setupLogins();
});

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
  let userRes = await registerUser(alpha);
  expect(userRes.jwt).toBeDefined();
  let user = new LemmyHttp(alphaUrl, {
    headers: { Authorization: `Bearer ${userRes.jwt ?? ""}` },
  });

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
  let userRes = await registerUser(alpha);
  expect(userRes.jwt).toBeDefined();
  let user = new LemmyHttp(alphaUrl, {
    headers: { Authorization: `Bearer ${userRes.jwt ?? ""}` },
  });

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

  // Fetch user content before deletion
  let betaPost1 = (await resolvePost(beta, localPost)).post;
  expect(betaPost1).toBeDefined();
  let betaPost2 = (await resolvePost(beta, remotePost)).post;
  expect(betaPost2).toBeDefined();
  let follow = await followCommunity(beta, true, betaCommunity.community.id);
  expect(follow.community_view.community).toBeDefined();

  // Delete user account and content
  await deleteUser(user);

  // Attempt to fetch user content from original instance, fails because its deleted
  await expect(resolvePost(gamma, localPost)).rejects.toBe(
    "couldnt_find_object",
  );
  await expect(resolveComment(gamma, localComment)).rejects.toBe(
    "couldnt_find_object",
  );
  await expect(resolvePost(gamma, remotePost)).rejects.toBe(
    "couldnt_find_object",
  );
  await expect(resolveComment(gamma, remoteComment)).rejects.toBe(
    "couldnt_find_object",
  );

  // Attempt to read user content from federated instance, fails because of federated
  // deletion
  expect((await getPost(beta, betaPost1!.post.id)).post_view.post.deleted).toBe(
    true,
  );
  expect((await getPost(beta, betaPost2!.post.id)).post_view.post.deleted).toBe(
    true,
  );
});

test("Requests with invalid auth should be treated as unauthenticated", async () => {
  let invalid_auth = new LemmyHttp(alphaUrl, {
    headers: { Authorization: "Bearer foobar" },
  });
  let site = await getSite(invalid_auth);
  expect(site.my_user).toBeUndefined();
  expect(site.site_view).toBeDefined();

  let form: GetPosts = {};
  let posts = invalid_auth.getPosts(form);
  expect((await posts).posts).toBeDefined();
});

test("Create user with Arabic name", async () => {
  let userRes = await registerUser(alpha, "تجريب");
  expect(userRes.jwt).toBeDefined();
  let user = new LemmyHttp(alphaUrl, {
    headers: { Authorization: `Bearer ${userRes.jwt ?? ""}` },
  });

  let site = await getSite(user);
  expect(site.my_user).toBeDefined();
  if (!site.my_user) {
    throw "Missing site user";
  }
  apShortname = `@${site.my_user.local_user_view.person.name}@lemmy-alpha:8541`;

  let alphaPerson = (await resolvePerson(alpha, apShortname)).person;
  expect(alphaPerson).toBeDefined();
});
