jest.setTimeout(120000);
import {
  alpha,
  beta,
  registerUser,
  resolvePerson,
  saveUserSettings,
  getSite,
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
