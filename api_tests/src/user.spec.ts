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
  resolvePost,
  API,
  resolveComment,
  saveUserSettingsFederated,
  setupLogins,
  followCommunity,
} from "./shared";
import { LemmyHttp } from "lemmy-js-client";
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

  // subscribe to alpha community from user
  await followCommunity(user, true, alphaCommunity.community.id);

  // for aggregate checking, refresh community before account delete
  let beforeAlphaCommunity = (
    await resolveCommunity(alpha, "!main@lemmy-alpha:8541")
  ).community;
  if (!beforeAlphaCommunity) {
    throw "Missing alpha community";
  }
  // also getSite
  let beforeSite = await getSite(alpha);

  await deleteUser(user);

  await expect(resolvePost(alpha, localPost)).rejects.toBe(
    "couldnt_find_object",
  );
  await expect(resolveComment(alpha, localComment)).rejects.toBe(
    "couldnt_find_object",
  );
  await expect(resolvePost(alpha, remotePost)).rejects.toBe(
    "couldnt_find_object",
  );
  await expect(resolveComment(alpha, remoteComment)).rejects.toBe(
    "couldnt_find_object",
  );
  // try to use the account that is now deleted
  await expect(resolveBetaCommunity(user)).rejects.toBe("deleted");

  // let alpha view the aftermath
  let afterAlphaCommunity = (
    await resolveCommunity(alpha, "!main@lemmy-alpha:8541")
  ).community;
  if (!afterAlphaCommunity) {
    throw "Missing alpha community";
  }
  let afterSite = await getSite(alpha);

  // possible bug in 0.18.2
  // Does lemmy_server do a SQL DELETE of a person or only mark the deleted=true column? This trigger does not seem to actually fire:
  //    CREATE TRIGGER site_aggregates_person_delete AFTER DELETE ON public.person FOR EACH ROW WHEN ((old.local = true)) EXECUTE FUNCTION public.site_aggregates_person_delete();
  //  expect(beforeSite.site_view.counts.users).toBe(afterSite.site_view.counts.users + 1);

  console.log(
    "%d vs %d and %d vs %d",
    beforeSite.site_view.counts.posts,
    afterSite.site_view.counts.posts,
    beforeAlphaCommunity?.counts.subscribers,
    afterAlphaCommunity?.counts.subscribers,
  );
  expect(beforeSite.site_view.counts.posts).toBe(
    afterSite.site_view.counts.posts + 2,
  );
  expect(beforeSite.site_view.counts.comments).toBe(
    afterSite.site_view.counts.comments + 2,
  );
  expect(beforeAlphaCommunity?.counts.comments).toBe(
    afterAlphaCommunity?.counts.comments + 1,
  );
  expect(beforeAlphaCommunity?.counts.posts).toBe(
    afterAlphaCommunity?.counts.posts + 1,
  );

  // possible bug in 0.18.2
  // does deleting a user account unsubscribe from communities?
  //  expect(beforeAlphaCommunity?.counts.subscribers).toBe(afterAlphaCommunity?.counts.subscribers + 1);
});

test("Requests with invalid auth should be treated as unauthenticated", async () => {
  let invalid_auth: API = {
    client: new LemmyHttp("http://127.0.0.1:8541"),
    auth: "invalid",
  };
  let site = await getSite(invalid_auth);
  expect(site.my_user).toBeUndefined();
  expect(site.site_view).toBeDefined();

  let form: GetPosts = {
    auth: "invalid",
  };
  let posts = invalid_auth.client.getPosts(form);
  expect((await posts).posts).toBeDefined();
});
