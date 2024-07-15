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
  alphaImage,
  unfollows,
  saveUserSettingsBio,
} from "./shared";
import { LemmyHttp, SaveUserSettings, UploadImage } from "lemmy-js-client";
import { GetPosts } from "lemmy-js-client/dist/types/GetPosts";

beforeAll(setupLogins);
afterAll(unfollows);

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
  apShortname = `${site.my_user.local_user_view.person.name}@lemmy-alpha:8541`;
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
  let alphaCommunity = (await resolveCommunity(user, "main@lemmy-alpha:8541"))
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
  let user = await registerUser(
    alpha,
    alphaUrl,
    "تجريب" + Math.random().toString().slice(2),
  );

  let site = await getSite(user);
  expect(site.my_user).toBeDefined();
  if (!site.my_user) {
    throw "Missing site user";
  }
  apShortname = `${site.my_user.local_user_view.person.name}@lemmy-alpha:8541`;

  let alphaPerson = (await resolvePerson(alpha, apShortname)).person;
  expect(alphaPerson).toBeDefined();
});

test("Create user with accept-language", async () => {
  let lemmy_http = new LemmyHttp(alphaUrl, {
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Language#syntax
    headers: { "Accept-Language": "fr-CH, en;q=0.8, de;q=0.7, *;q=0.5" },
  });
  let user = await registerUser(lemmy_http, alphaUrl);

  let site = await getSite(user);
  expect(site.my_user).toBeDefined();
  expect(site.my_user?.local_user_view.local_user.interface_language).toBe(
    "fr",
  );
  let langs = site.all_languages
    .filter(a => site.my_user?.discussion_languages.includes(a.id))
    .map(l => l.code);
  // should have languages from accept header, as well as "undetermined"
  // which is automatically enabled by backend
  expect(langs).toStrictEqual(["und", "de", "en", "fr"]);
});

test("Set a new avatar, old avatar is deleted", async () => {
  const listMediaRes = await alphaImage.listMedia();
  expect(listMediaRes.images.length).toBe(0);
  const upload_form1: UploadImage = {
    image: Buffer.from("test1"),
  };
  const upload1 = await alphaImage.uploadImage(upload_form1);
  expect(upload1.url).toBeDefined();

  let form1 = {
    avatar: upload1.url,
  };
  await saveUserSettings(alpha, form1);
  const listMediaRes1 = await alphaImage.listMedia();
  expect(listMediaRes1.images.length).toBe(1);

  const upload_form2: UploadImage = {
    image: Buffer.from("test2"),
  };
  const upload2 = await alphaImage.uploadImage(upload_form2);
  expect(upload2.url).toBeDefined();

  let form2 = {
    avatar: upload2.url,
  };
  await saveUserSettings(alpha, form2);
  // make sure only the new avatar is kept
  const listMediaRes2 = await alphaImage.listMedia();
  expect(listMediaRes2.images.length).toBe(1);

  // Upload that same form2 avatar, make sure it isn't replaced / deleted
  await saveUserSettings(alpha, form2);
  // make sure only the new avatar is kept
  const listMediaRes3 = await alphaImage.listMedia();
  expect(listMediaRes3.images.length).toBe(1);

  // Now try to save a user settings, with the icon missing,
  // and make sure it doesn't clear the data, or delete the image
  await saveUserSettingsBio(alpha);
  let site = await getSite(alpha);
  expect(site.my_user?.local_user_view.person.avatar).toBe(upload2.url);

  // make sure only the new avatar is kept
  const listMediaRes4 = await alphaImage.listMedia();
  expect(listMediaRes4.images.length).toBe(1);
});
