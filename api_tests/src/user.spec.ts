jest.setTimeout(120000);
import { PersonView } from "lemmy-js-client";

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
  resolvePost,
  API,
  resolveComment,
  saveUserSettingsFederated,
  setupLogins,
} from "./shared";

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
  alpha.auth = userRes.jwt ?? "";

  let site = await getSite(alpha);
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
});

test("Delete user", async () => {
  let userRes = await registerUser(alpha);
  expect(userRes.jwt).toBeDefined();
  let user: API = {
    client: alpha.client,
    auth: userRes.jwt ?? "",
  };

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

  expect((await resolvePost(alpha, localPost)).post).toBeUndefined();
  expect((await resolveComment(alpha, localComment)).comment).toBeUndefined();
  expect((await resolvePost(alpha, remotePost)).post).toBeUndefined();
  expect((await resolveComment(alpha, remoteComment)).comment).toBeUndefined();
});
