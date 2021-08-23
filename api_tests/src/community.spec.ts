jest.setTimeout(120000);
import {
  alpha,
  beta,
  setupLogins,
  resolveCommunity,
  createCommunity,
  deleteCommunity,
  removeCommunity,
  getCommunity,
  followCommunity,
} from './shared';
import { CommunityView } from 'lemmy-js-client';

beforeAll(async () => {
  await setupLogins();
});

function assertCommunityFederation(
  communityOne: CommunityView,
  communityTwo: CommunityView
) {
  expect(communityOne.community.actor_id).toBe(communityTwo.community.actor_id);
  expect(communityOne.community.name).toBe(communityTwo.community.name);
  expect(communityOne.community.title).toBe(communityTwo.community.title);
  expect(communityOne.community.description).toBe(
    communityTwo.community.description
  );
  expect(communityOne.community.icon).toBe(communityTwo.community.icon);
  expect(communityOne.community.banner).toBe(communityTwo.community.banner);
  expect(communityOne.community.published).toBe(
    communityTwo.community.published
  );
  expect(communityOne.community.nsfw).toBe(communityTwo.community.nsfw);
  expect(communityOne.community.removed).toBe(communityTwo.community.removed);
  expect(communityOne.community.deleted).toBe(communityTwo.community.deleted);
}

test('Create community', async () => {
  let communityRes = await createCommunity(alpha);
  expect(communityRes.community_view.community.name).toBeDefined();

  // A dupe check
  let prevName = communityRes.community_view.community.name;
  let communityRes2: any = await createCommunity(alpha, prevName);
  expect(communityRes2['error']).toBe('community_already_exists');

  // Cache the community on beta, make sure it has the other fields
  let searchShort = `!${prevName}@lemmy-alpha:8541`;
  let betaCommunity = (await resolveCommunity(beta, searchShort)).community;
  assertCommunityFederation(betaCommunity, communityRes.community_view);
});

test('Delete community', async () => {
  let communityRes = await createCommunity(beta);

  // Cache the community on Alpha
  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = (await resolveCommunity(alpha, searchShort)).community;
  assertCommunityFederation(alphaCommunity, communityRes.community_view);

  // Follow the community from alpha
  let follow = await followCommunity(
    alpha,
    true,
    alphaCommunity.community.id
  );

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(false);

  let deleteCommunityRes = await deleteCommunity(
    beta,
    true,
    communityRes.community_view.community.id
  );
  expect(deleteCommunityRes.community_view.community.deleted).toBe(true);
  expect(deleteCommunityRes.community_view.community.title).toBe("");

  // Make sure it got deleted on A
  let communityOnAlphaDeleted = await getCommunity(
    alpha,
    alphaCommunity.community.id
  );
  expect(communityOnAlphaDeleted.community_view.community.deleted).toBe(true);

  // Undelete
  let undeleteCommunityRes = await deleteCommunity(
    beta,
    false,
    communityRes.community_view.community.id
  );
  expect(undeleteCommunityRes.community_view.community.deleted).toBe(false);

  // Make sure it got undeleted on A
  let communityOnAlphaUnDeleted = await getCommunity(
    alpha,
    alphaCommunity.community.id
  );
  expect(communityOnAlphaUnDeleted.community_view.community.deleted).toBe(
    false
  );
});

test('Remove community', async () => {
  let communityRes = await createCommunity(beta);

  // Cache the community on Alpha
  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = (await resolveCommunity(alpha, searchShort)).community;
  assertCommunityFederation(alphaCommunity, communityRes.community_view);

  // Follow the community from alpha
  let follow = await followCommunity(
    alpha,
    true,
    alphaCommunity.community.id
  );

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(false);

  let removeCommunityRes = await removeCommunity(
    beta,
    true,
    communityRes.community_view.community.id
  );
  expect(removeCommunityRes.community_view.community.removed).toBe(true);
  expect(removeCommunityRes.community_view.community.title).toBe("");

  // Make sure it got Removed on A
  let communityOnAlphaRemoved = await getCommunity(
    alpha,
    alphaCommunity.community.id
  );
  expect(communityOnAlphaRemoved.community_view.community.removed).toBe(true);

  // unremove
  let unremoveCommunityRes = await removeCommunity(
    beta,
    false,
    communityRes.community_view.community.id
  );
  expect(unremoveCommunityRes.community_view.community.removed).toBe(false);

  // Make sure it got undeleted on A
  let communityOnAlphaUnRemoved = await getCommunity(
    alpha,
    alphaCommunity.community.id
  );
  expect(communityOnAlphaUnRemoved.community_view.community.removed).toBe(
    false
  );
});

test('Search for beta community', async () => {
  let communityRes = await createCommunity(beta);
  expect(communityRes.community_view.community.name).toBeDefined();

  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = (await resolveCommunity(alpha, searchShort)).community;
  assertCommunityFederation(alphaCommunity, communityRes.community_view);
});
