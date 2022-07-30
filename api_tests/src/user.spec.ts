jest.setTimeout(120000);
import {None} from '@sniptt/monads';
import {
  PersonViewSafe,
} from 'lemmy-js-client';

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
} from './shared';

let apShortname: string;

function assertUserFederation(userOne: PersonViewSafe, userTwo: PersonViewSafe) {
  expect(userOne.person.name).toBe(userTwo.person.name);
  expect(userOne.person.display_name.unwrapOr("none")).toBe(userTwo.person.display_name.unwrapOr("none"));
  expect(userOne.person.bio.unwrapOr("none")).toBe(userTwo.person.bio.unwrapOr("none"));
  expect(userOne.person.actor_id).toBe(userTwo.person.actor_id);
  expect(userOne.person.avatar.unwrapOr("none")).toBe(userTwo.person.avatar.unwrapOr("none"));
  expect(userOne.person.banner.unwrapOr("none")).toBe(userTwo.person.banner.unwrapOr("none"));
  expect(userOne.person.published).toBe(userTwo.person.published);
}

test('Create user', async () => {
  let userRes = await registerUser(alpha);
  expect(userRes.jwt).toBeDefined();
  alpha.auth = userRes.jwt;
  
  let site = await getSite(alpha);
  expect(site.my_user).toBeDefined();
  apShortname = `@${site.my_user.unwrap().local_user_view.person.name}@lemmy-alpha:8541`;
});

test('Set some user settings, check that they are federated', async () => {
  await saveUserSettingsFederated(alpha);
  let alphaPerson = (await resolvePerson(alpha, apShortname)).person.unwrap();
  let betaPerson = (await resolvePerson(beta, apShortname)).person.unwrap();
  assertUserFederation(alphaPerson, betaPerson);
});

test('Delete user', async () => {
  let userRes = await registerUser(alpha);
  expect(userRes.jwt).toBeDefined();
  let user: API = {
    client: alpha.client,
    auth: userRes.jwt
  }

  // make a local post and comment
  let alphaCommunity = (await resolveCommunity(user, '!main@lemmy-alpha:8541')).community.unwrap();
  let localPost = (await createPost(user, alphaCommunity.community.id)).post_view.post;
  expect(localPost).toBeDefined();
  let localComment = (await createComment(user, localPost.id, None)).comment_view.comment;
  expect(localComment).toBeDefined();

  // make a remote post and comment
  let betaCommunity = (await resolveBetaCommunity(user)).community.unwrap();
  let remotePost = (await createPost(user, betaCommunity.community.id)).post_view.post;
  expect(remotePost).toBeDefined();
  let remoteComment = (await createComment(user, remotePost.id, None)).comment_view.comment;
  expect(remoteComment).toBeDefined();

  await deleteUser(user);

  expect((await resolvePost(alpha, localPost)).post.isNone()).toBe(true);
  expect((await resolveComment(alpha, localComment)).comment.isNone()).toBe(true)
  expect((await resolvePost(alpha, remotePost)).post.isNone()).toBe(true)
  expect((await resolveComment(alpha, remoteComment)).comment.isNone()).toBe(true)
});
