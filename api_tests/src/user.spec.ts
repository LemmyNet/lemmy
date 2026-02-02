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
  betaUrl,
  saveUserSettings,
  getPost,
  getComments,
  fetchFunction,
  alphaImage,
  unfollows,
  getMyUser,
  getPersonDetails,
  banPersonFromSite,
  statusNotFound,
  statusUnauthorized,
  listPersonContent,
  waitUntil,
  password,
  jestLemmyError,
  statusBadRequest,
  randomString,
} from "./shared";
import {
  EditSite,
  LemmyError,
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
  expect(userOne?.person.published_at).toBe(userTwo?.person.published_at);
}

test("Create user", async () => {
  let user = await registerUser(alpha, alphaUrl);

  let myUser = await getMyUser(user);
  expect(myUser).toBeDefined();
  apShortname = `${myUser.local_user_view.person.name}@lemmy-alpha:8541`;
});

test("Set some user settings, check that they are federated", async () => {
  await saveUserSettingsFederated(alpha);
  let alphaPerson = await resolvePerson(alpha, apShortname);
  let betaPerson = await resolvePerson(beta, apShortname);
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
  let alphaCommunity = await resolveCommunity(user, "main@lemmy-alpha:8541");
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
  let betaCommunity = await resolveBetaCommunity(user);
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

  // Wait, in order to make sure it federates
  await jestLemmyError(
    () => getMyUser(user),
    new LemmyError("incorrect_login", statusUnauthorized),
  );

  await jestLemmyError(
    () => getPersonDetails(user, person_id),
    new LemmyError("not_found", statusNotFound),
  );

  // check that posts and comments are marked as deleted on other instances.
  // use get methods to avoid refetching from origin instance
  expect((await getPost(alpha, localPost.id)).post_view.post.deleted).toBe(
    true,
  );
  // Make sure the remote post is deleted.
  // TODO this fails occasionally
  // Probably because it could return a not_found
  // await waitUntil(
  //   () => getPost(alpha, remotePost.id),
  //   p => p.post_view.post.deleted === true || p.post_view.post === undefined,
  // );
  await waitUntil(
    () => getComments(alpha, localComment.post_id),
    c => c.items[0].comment.deleted,
  );
  await waitUntil(
    () => alpha.getComment({ id: remoteComment.id }),
    c => c.comment_view.comment.deleted,
  );
  await jestLemmyError(
    () => getPersonDetails(user, remoteComment.creator_id),
    new LemmyError("not_found", statusNotFound),
  );
});

test("Requests with invalid auth should be treated as unauthenticated", async () => {
  let invalid_auth = new LemmyHttp(alphaUrl, {
    headers: { Authorization: "Bearer foobar" },
    fetchFunction,
  });
  await jestLemmyError(
    () => getMyUser(invalid_auth),
    new LemmyError("incorrect_login", statusUnauthorized),
  );
  let site = await getSite(invalid_auth);
  expect(site.site_view).toBeDefined();

  let form: GetPosts = {};
  let posts = invalid_auth.getPosts(form);
  expect((await posts).items).toBeDefined();
});

test("Create user with Arabic name", async () => {
  // less than actor_name_max_length
  const name = "تجريب" + Math.random().toString().slice(2, 10);
  let user = await registerUser(alpha, alphaUrl, name);

  let my_user = await getMyUser(user);
  expect(my_user).toBeDefined();
  apShortname = `${my_user.local_user_view.person.name}@lemmy-alpha:8541`;

  let betaPerson1 = await resolvePerson(beta, apShortname);
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
    .map(l => l.code)
    .sort();
  // should have languages from accept header, as well as "undetermined"
  // which is automatically enabled by backend
  expect(langs).toStrictEqual(["de", "en", "fr"]);
});

test("Set a new avatar, old avatar is deleted", async () => {
  const listMediaRes = await alphaImage.listMedia();
  expect(listMediaRes.items.length).toBe(0);
  const upload_form1: UploadImage = {
    image: Buffer.from("test1"),
  };
  await alpha.uploadUserAvatar(upload_form1);
  const listMediaRes1 = await alphaImage.listMedia();
  expect(listMediaRes1.items.length).toBe(1);

  let my_user1 = await alpha.getMyUser();
  expect(my_user1.local_user_view.person.avatar).toBeDefined();

  const upload_form2: UploadImage = {
    image: Buffer.from("test2"),
  };
  await alpha.uploadUserAvatar(upload_form2);
  // make sure only the new avatar is kept
  const listMediaRes2 = await alphaImage.listMedia();
  expect(listMediaRes2.items.length).toBe(1);

  // Upload that same form2 avatar, make sure it isn't replaced / deleted
  await alpha.uploadUserAvatar(upload_form2);
  // make sure only the new avatar is kept
  const listMediaRes3 = await alphaImage.listMedia();
  expect(listMediaRes3.items.length).toBe(1);

  // make sure only the new avatar is kept
  const listMediaRes4 = await alphaImage.listMedia();
  expect(listMediaRes4.items.length).toBe(1);

  // delete the avatar
  await alpha.deleteUserAvatar();
  // make sure only the new avatar is kept
  const listMediaRes5 = await alphaImage.listMedia();
  expect(listMediaRes5.items.length).toBe(0);
  let my_user2 = await alpha.getMyUser();
  expect(my_user2.local_user_view.person.avatar).toBeUndefined();
});

test("Make sure banned user can delete their account", async () => {
  let user = await registerUser(alpha, alphaUrl);
  let myUser = await getMyUser(user);

  // make a local post
  let alphaCommunity = await resolveCommunity(user, "main@lemmy-alpha:8541");
  if (!alphaCommunity) {
    throw "Missing alpha community";
  }
  let localPost = (await createPost(user, alphaCommunity.community.id))
    .post_view.post;
  let postId = localPost.id;
  expect(localPost).toBeDefined();

  // Ban the user, keep data
  let banUser = await banPersonFromSite(
    alpha,
    myUser.local_user_view.person.id,
    true,
    false,
  );
  expect(banUser.person_view.banned).toBe(true);

  // Make sure post is there
  let postAfterBan = await getPost(alpha, postId);
  expect(postAfterBan.post_view.post.deleted).toBe(false);

  // Delete account
  let deleteAccount = await deleteUser(user);
  expect(deleteAccount).toBeDefined();

  // Make sure post is gone
  let postAfterDelete = await getPost(alpha, postId);
  expect(postAfterDelete.post_view.post.deleted).toBe(true);
  expect(postAfterDelete.post_view.post.name).toBe("*Permanently Deleted*");
});

test("Admins can view and ban deleted accounts", async () => {
  let user = await registerUser(beta, betaUrl);
  let myUser = await getMyUser(user);
  let apShortname = `${myUser.local_user_view.person.name}@lemmy-beta:8551`;
  let userOnAlpha = await resolvePerson(alpha, apShortname);

  let alphaCommunity = await resolveCommunity(user, "main@lemmy-alpha:8541");
  if (!alphaCommunity) {
    throw "Missing alpha community";
  }

  // Make a post and then delete the account
  let postRes = await createPost(user, alphaCommunity.community.id);
  let deletedUser = await deleteUser(user, false);
  expect(deletedUser).toBeDefined();
  // Make sure the post is still visible
  let postAfterDelete = await getPost(beta, postRes.post_view.post.id);
  expect(postAfterDelete.post_view.post.deleted).toBe(false);

  // Ensure admins can still resolve the user
  let getDeletedUser = await getPersonDetails(
    beta,
    myUser.local_user_view.person.id,
  );
  expect(getDeletedUser).toBeDefined();

  // Make sure the delete federates
  await waitUntil(
    () => getPersonDetails(alpha, userOnAlpha!.person.id),
    p => p.person_view.person.deleted,
  );

  // Ban the user
  let banUser = await banPersonFromSite(
    beta,
    myUser.local_user_view.person.id,
    true,
    true,
  );
  expect(banUser.person_view.banned).toBe(true);
  // Make sure the post is removed
  let postAfterBan = await getPost(beta, postRes.post_view.post.id);
  expect(postAfterBan.post_view.post.removed).toBe(true);

  // Make sure the ban federates properly
  let getDeletedUserAlpha = await waitUntil(
    () => getPersonDetails(alpha, userOnAlpha!.person.id),
    p => p.person_view.banned,
  );
  // Make sure content removal also went through
  let userContent = await listPersonContent(
    alpha,
    getDeletedUserAlpha.person_view.person.id,
    "posts",
  );
  expect(userContent.items[0].post.removed).toBe(true);
});

test("Make sure a denied user is given denial reason", async () => {
  const username = randomString(10);
  const appAnswer = "My application answer";
  const denyReason = "Bad application given";

  // Make registrations approval only
  await alpha.editSite({ registration_mode: "require_application" });

  // Create an account with an answer
  const login = await alpha.register({
    username,
    password,
    password_verify: password,
    show_nsfw: true,
    answer: appAnswer,
  });
  expect(login.registration_created).toBeTruthy();
  expect(login.jwt).toBeUndefined();

  // Try to login with a bad password first
  await jestLemmyError(
    () =>
      alpha.login({ username_or_email: username, password: "wrong_password" }),
    new LemmyError("incorrect_login", statusUnauthorized),
  );

  // Try to login without approval yet, should return is pending
  await jestLemmyError(
    () => alpha.login({ username_or_email: username, password }),
    new LemmyError("registration_application_is_pending", statusBadRequest),
  );

  // Fetch the applications
  const apps = await alpha.listRegistrationApplications({});
  const app = apps.items[0];
  expect(apps.items.length).toBeGreaterThanOrEqual(1);
  expect(app.registration_application.answer).toBe(appAnswer);

  // Deny the application
  await alpha.approveRegistrationApplication({
    id: app.registration_application.id,
    approve: false,
    deny_reason: denyReason,
  });

  // Should give the denial reason in the error.
  await jestLemmyError(
    () => alpha.login({ username_or_email: username, password }),
    new LemmyError("registration_denied", statusBadRequest, denyReason),
  );

  // Re-open alpha
  await alpha.editSite({ registration_mode: "open" });
});
