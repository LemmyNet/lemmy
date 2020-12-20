jest.setTimeout(120000);
import {
  alpha,
  beta,
  registerUser,
  searchForUser,
  saveUserSettingsBio,
  saveUserSettings,
  getSite,
} from './shared';
import {
  UserViewSafe,
  SaveUserSettings,
  SortType,
  ListingType,
} from 'lemmy-js-client';

let auth: string;
let apShortname: string;

function assertUserFederation(userOne: UserViewSafe, userTwo: UserViewSafe) {
  expect(userOne.user.name).toBe(userTwo.user.name);
  expect(userOne.user.preferred_username).toBe(userTwo.user.preferred_username);
  expect(userOne.user.bio).toBe(userTwo.user.bio);
  expect(userOne.user.actor_id).toBe(userTwo.user.actor_id);
  expect(userOne.user.avatar).toBe(userTwo.user.avatar);
  expect(userOne.user.banner).toBe(userTwo.user.banner);
  expect(userOne.user.published).toBe(userTwo.user.published);
}

test('Create user', async () => {
  let userRes = await registerUser(alpha);
  expect(userRes.jwt).toBeDefined();
  auth = userRes.jwt;

  let site = await getSite(alpha, auth);
  expect(site.my_user).toBeDefined();
  apShortname = `@${site.my_user.name}@lemmy-alpha:8541`;
});

test('Save user settings, check changed bio from beta', async () => {
  let bio = 'a changed bio';
  let userRes = await saveUserSettingsBio(alpha, auth);
  expect(userRes.jwt).toBeDefined();

  let site = await getSite(alpha, auth);
  expect(site.my_user.bio).toBe(bio);
  let searchAlpha = await searchForUser(alpha, site.my_user.actor_id);

  // Make sure beta sees this bio is changed
  let searchBeta = await searchForUser(beta, apShortname);
  assertUserFederation(searchAlpha.users[0], searchBeta.users[0]);
});

test('Set avatar and banner, check that they are federated', async () => {
  let avatar = 'https://image.flaticon.com/icons/png/512/35/35896.png';
  let banner = 'https://image.flaticon.com/icons/png/512/36/35896.png';
  let form: SaveUserSettings = {
    show_nsfw: false,
    theme: '',
    default_sort_type: SortType.Hot,
    default_listing_type: ListingType.All,
    lang: '',
    avatar,
    banner,
    preferred_username: 'user321',
    show_avatars: false,
    send_notifications_to_email: false,
    auth,
  };
  let _settingsRes = await saveUserSettings(alpha, form);

  let searchAlpha = await searchForUser(beta, apShortname);
  let userOnAlpha = searchAlpha.users[0];
  let searchBeta = await searchForUser(beta, apShortname);
  let userOnBeta = searchBeta.users[0];
  assertUserFederation(userOnAlpha, userOnBeta);
});
