import {
  alpha,
  beta,
  registerUser,
  searchForUser,
  saveUserSettingsBio,
  getSite,
} from './shared';

let auth: string;
let apShortname: string;

test('Create user', async () => {
  let userRes = await registerUser(alpha);
  expect(userRes.jwt).toBeDefined();
  auth = userRes.jwt;

  let site = await getSite(alpha, auth);
  expect(site.my_user).toBeDefined();
  apShortname = `@${site.my_user.name}@lemmy-alpha:8540`;
});

test('Save user settings, check changed bio from beta', async () => {
  let bio = 'a changed bio';
  let userRes = await saveUserSettingsBio(alpha, auth);
  expect(userRes.jwt).toBeDefined();

  let site = await getSite(alpha, auth);
  expect(site.my_user.bio).toBe(bio);

  // Make sure beta sees this bio is changed
  let search = await searchForUser(beta, apShortname);
  expect(search.users[0].bio).toBe(bio);
});
