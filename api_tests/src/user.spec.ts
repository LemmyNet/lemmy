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
  UserView,
  UserSettingsForm,
} from 'lemmy-js-client';

let auth: string;
let apShortname: string;

function assertUserFederation(
  userOne: UserView,
  userTwo: UserView) {
  expect(userOne.name).toBe(userTwo.name);
  expect(userOne.preferred_username).toBe(userTwo.preferred_username);
  expect(userOne.bio).toBe(userTwo.bio);
  expect(userOne.actor_id).toBe(userTwo.actor_id);
  expect(userOne.avatar).toBe(userTwo.avatar);
  expect(userOne.banner).toBe(userTwo.banner);
  expect(userOne.published).toBe(userTwo.published);
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
  let form: UserSettingsForm = {
    show_nsfw: false,
    theme: "",
    default_sort_type: 0,
    default_listing_type: 0,
    lang: "",
    avatar,
    banner,
    preferred_username: "user321",
    show_avatars: false,
    send_notifications_to_email: false,
    auth,
  }
  let settingsRes = await saveUserSettings(alpha, form);

  let searchAlpha = await searchForUser(beta, apShortname);
  let userOnAlpha = searchAlpha.users[0];
  let searchBeta = await searchForUser(beta, apShortname);
  let userOnBeta = searchBeta.users[0];
  assertUserFederation(userOnAlpha, userOnBeta);
});
