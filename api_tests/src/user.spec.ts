jest.setTimeout(120000);
import {
  alpha,
  beta,
  registerUser,
  resolvePerson,
  saveUserSettings,
  getSite,
  createPost,
  gamma,
  resolveCommunity,
  createComment,
  resolveBetaCommunity,
  deleteUser,
  resolvePost,
  API,
  resolveComment,
} from './shared';
import {
  PersonViewSafe,
  SaveUserSettings,
  SortType,
  ListingType,
} from 'lemmy-js-client';

let apShortname: string;

function assertUserFederation(userOne: PersonViewSafe, userTwo: PersonViewSafe) {
  expect(userOne.person.name).toBe(userTwo.person.name);
  expect(userOne.person.display_name).toBe(userTwo.person.display_name);
  expect(userOne.person.bio).toBe(userTwo.person.bio);
  expect(userOne.person.actor_id).toBe(userTwo.person.actor_id);
  expect(userOne.person.avatar).toBe(userTwo.person.avatar);
  expect(userOne.person.banner).toBe(userTwo.person.banner);
  expect(userOne.person.published).toBe(userTwo.person.published);
}

test('Create user', async () => {
  let userRes = await registerUser(alpha);
  expect(userRes.jwt).toBeDefined();
  alpha.auth = userRes.jwt;
  
  let site = await getSite(alpha);
  expect(site.my_user).toBeDefined();
  apShortname = `@${site.my_user.local_user_view.person.name}@lemmy-alpha:8541`;
});

test('Set some user settings, check that they are federated', async () => {
  let avatar = 'https://image.flaticon.com/icons/png/512/35/35896.png';
  let banner = 'https://image.flaticon.com/icons/png/512/36/35896.png';
  let bio = 'a changed bio';
  let form: SaveUserSettings = {
    show_nsfw: false,
    theme: '',
    default_sort_type: Object.keys(SortType).indexOf(SortType.Hot),
    default_listing_type: Object.keys(ListingType).indexOf(ListingType.All),
    lang: '',
    avatar,
    banner,
    display_name: 'user321',
    show_avatars: false,
    send_notifications_to_email: false,
    bio,
    auth: alpha.auth,
  };
  await saveUserSettings(alpha, form);

  let alphaPerson = (await resolvePerson(alpha, apShortname)).person;
  let betaPerson = (await resolvePerson(beta, apShortname)).person;
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
  let alphaCommunity = (await resolveCommunity(user, '!main@lemmy-alpha:8541')).community;
  let localPost = (await createPost(user, alphaCommunity.community.id)).post_view.post;
  expect(localPost).toBeDefined();
  let localComment = (await createComment(user, localPost.id)).comment_view.comment;
  expect(localComment).toBeDefined();

  // make a remote post and comment
  let betaCommunity = (await resolveBetaCommunity(user)).community;
  let remotePost = (await createPost(user, betaCommunity.community.id)).post_view.post;
  expect(remotePost).toBeDefined();
  let remoteComment = (await createComment(user, remotePost.id)).comment_view.comment;
  expect(remoteComment).toBeDefined();

  await deleteUser(user);

  expect((await resolvePost(alpha, localPost)).post).toBeUndefined();
  expect((await resolveComment(alpha, localComment)).comment).toBeUndefined();
  expect((await resolvePost(alpha, remotePost)).post).toBeUndefined();
  expect((await resolveComment(alpha, remoteComment)).comment).toBeUndefined();
});
