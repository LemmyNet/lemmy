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
  getMyUser,
  getPersonDetails,
} from "./shared";
import {
  EditSite,
  LemmyHttp,
  SaveUserSettings,
  UploadImage,
} from "lemmy-js-client";
import { GetPosts } from "lemmy-js-client/dist/types/GetPosts";

beforeAll(setupLogins);
afterAll(unfollows);

let apShortname: string;

function assertUserFederation(userOne?: PersonView, userTwo?: PersonView) {
  expect(userOne?.person.name).toBe(userTwo?.person.name);
  expect(userOne?.person.display_name).toBe(userTwo?.person.display_name);
  expect(userOne?.person.bio).toBe(userTwo?.person.bio);
  expect(userOne?.person.ap_id).toBe(userTwo?.person.ap_id);
  expect(userOne?.person.avatar).toBe(userTwo?.person.avatar);
  expect(userOne?.person.banner).toBe(userTwo?.person.banner);
  expect(userOne?.person.published).toBe(userTwo?.person.published);
}

test("Create user", async () => {
  let user = await registerUser(alpha, alphaUrl);

  let my_user = await getMyUser(user);
  expect(my_user).toBeDefined();
  apShortname = `${my_user.local_user_view.person.name}@lemmy-alpha:8541`;
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

  let my_user = await getMyUser(beta);
  expect(my_user.local_user_view.local_user.theme).toBe("test");
});

test("Delete user", async () => {
  let user = await registerUser(alpha, alphaUrl);
  let user_profile = await getMyUser(user);
  let person_id = user_profile.local_user_view.person.id;

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
  await expect(getMyUser(user)).rejects.toStrictEqual(Error("incorrect_login"));
  await expect(getPersonDetails(user, person_id)).rejects.toStrictEqual(
    Error("not_found"),
  );

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
  await expect(
    getPersonDetails(user, remoteComment.creator_id),
  ).rejects.toStrictEqual(Error("not_found"));
});

test("Requests with invalid auth should be treated as unauthenticated", async () => {
  let invalid_auth = new LemmyHttp(alphaUrl, {
    headers: { Authorization: "Bearer foobar" },
    fetchFunction,
  });
  await expect(getMyUser(invalid_auth)).rejects.toStrictEqual(
    Error("incorrect_login"),
  );
  let site = await getSite(invalid_auth);
  expect(site.site_view).toBeDefined();

  let form: GetPosts = {};
  let posts = invalid_auth.getPosts(form);
  expect((await posts).posts).toBeDefined();
});

test("Create user with Arabic name", async () => {
  // less than actor_name_max_length
  const name = "تجريب" + Math.random().toString().slice(2, 10);
  let user = await registerUser(alpha, alphaUrl, name);

  let my_user = await getMyUser(user);
  expect(my_user).toBeDefined();
  apShortname = `${my_user.local_user_view.person.name}@lemmy-alpha:8541`;

  let betaPerson1 = (await resolvePerson(beta, apShortname)).person;
  expect(betaPerson1!.person.name).toBe(name);

  let betaPerson2 = await getPersonDetails(beta, betaPerson1!.person.id);
  expect(betaPerson2!.person_view.person.name).toBe(name);
});

test("Create user with accept-language", async () => {
  const edit: EditSite = {
    discussion_languages: [32],
  };
  await alpha.editSite(edit);

  let lemmy_http = new LemmyHttp(alphaUrl, {
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Language#syntax
    headers: { "Accept-Language": "fr-CH, en;q=0.8, *;q=0.5" },
  });
  let user = await registerUser(lemmy_http, alphaUrl);

  let my_user = await getMyUser(user);
  expect(my_user).toBeDefined();
  expect(my_user?.local_user_view.local_user.interface_language).toBe("fr");
  let site = await getSite(user);
  let langs = site.all_languages
    .filter(a => my_user.discussion_languages.includes(a.id))
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
  await alpha.uploadUserAvatar(upload_form1);
  const listMediaRes1 = await alphaImage.listMedia();
  expect(listMediaRes1.images.length).toBe(1);

  let my_user1 = await alpha.getMyUser();
  expect(my_user1.local_user_view.person.avatar).toBeDefined();

  const upload_form2: UploadImage = {
    image: Buffer.from("test2"),
  };
  await alpha.uploadUserAvatar(upload_form2);
  // make sure only the new avatar is kept
  const listMediaRes2 = await alphaImage.listMedia();
  expect(listMediaRes2.images.length).toBe(1);

  // Upload that same form2 avatar, make sure it isn't replaced / deleted
  await alpha.uploadUserAvatar(upload_form2);
  // make sure only the new avatar is kept
  const listMediaRes3 = await alphaImage.listMedia();
  expect(listMediaRes3.images.length).toBe(1);

  // make sure only the new avatar is kept
  const listMediaRes4 = await alphaImage.listMedia();
  expect(listMediaRes4.images.length).toBe(1);

  // delete the avatar
  await alpha.deleteUserAvatar();
  // make sure only the new avatar is kept
  const listMediaRes5 = await alphaImage.listMedia();
  expect(listMediaRes5.images.length).toBe(0);
  let my_user2 = await alpha.getMyUser();
  expect(my_user2.local_user_view.person.avatar).toBeUndefined();
});
