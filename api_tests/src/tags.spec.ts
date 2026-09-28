jest.setTimeout(120000);

import {
  alpha,
  beta,
  setupLogins,
  createCommunity,
  unfollows,
  randomString,
  followCommunity,
  resolveCommunity,
  waitUntil,
  assertCommunityFederation,
  waitForPost,
  gamma,
  resolvePerson,
  getCommunity,
  expectSuccess,
  waitUntilSuccess,
} from "./shared";
import { CreateCommunityTag } from "lemmy-js-client/dist/types/CreateCommunityTag";
import { DeleteCommunityTag } from "lemmy-js-client/dist/types/DeleteCommunityTag";
import { AddModToCommunity } from "lemmy-js-client";

beforeAll(setupLogins);
afterAll(unfollows);

test("Create, delete and restore a community tag", async () => {
  // Create a community first
  const communityRes = await createCommunity(alpha).then(expectSuccess);
  let alphaCommunity = communityRes.community_view;
  let betaCommunity = (await resolveCommunity(
    beta,
    alphaCommunity.community.ap_id,
  ))!;
  await followCommunity(beta, true, betaCommunity.community.id);
  await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g?.community_actions!.follow_state == "accepted",
  );
  const communityId = alphaCommunity.community.id;

  // Create a tag
  const tagName = randomString(10);
  const createForm: CreateCommunityTag = {
    name: tagName,
    community_id: communityId,
  };
  const createRes = await alpha
    .createCommunityTag(createForm)
    .then(expectSuccess);
  expect(createRes.id).toBeDefined();
  expect(createRes.name).toBe(tagName);
  expect(createRes.community_id).toBe(communityId);

  alphaCommunity = (
    await alpha.getCommunity({ id: communityId }).then(expectSuccess)
  ).community_view;
  expect(alphaCommunity.tags.length).toBe(1);
  // verify tag federated

  betaCommunity = (await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.tags.length === 1,
  ))!;
  assertCommunityFederation(alphaCommunity, betaCommunity);

  // List tags
  alphaCommunity = (
    await alpha.getCommunity({ id: communityId }).then(expectSuccess)
  ).community_view;
  expect(alphaCommunity.tags.length).toBe(1);
  expect(alphaCommunity.tags.find(t => t.id === createRes.id)?.name).toBe(
    tagName,
  );

  // Verify tag update federated
  betaCommunity = (await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.tags.find(t => t.ap_id === createRes.ap_id)?.name === tagName,
  ))!;
  assertCommunityFederation(alphaCommunity, betaCommunity);

  // Delete the tag
  const deleteForm: DeleteCommunityTag = {
    tag_id: createRes.id,
    delete: true,
  };
  const deleteRes = await alpha
    .deleteCommunityTag(deleteForm)
    .then(expectSuccess);
  expect(deleteRes.id).toBe(createRes.id);

  // Verify tag is deleted
  alphaCommunity = (
    await alpha.getCommunity({ id: communityId }).then(expectSuccess)
  ).community_view;
  expect(
    alphaCommunity.tags.find(t => t.id === createRes.id)!.deleted,
  ).toBeTruthy();
  // It should still list one tag
  expect(alphaCommunity.tags.length).toBe(1);

  // Verify tag deletion federated
  betaCommunity = (await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.tags.at(0)?.deleted === true,
  ))!;
  assertCommunityFederation(alphaCommunity, betaCommunity);

  // Restore the tag
  const deleteFormRestoration: DeleteCommunityTag = {
    tag_id: createRes.id,
    delete: false,
  };
  const deleteRestorationRes = await alpha
    .deleteCommunityTag(deleteFormRestoration)
    .then(expectSuccess);
  expect(deleteRestorationRes.id).toBe(createRes.id);

  // Verify tag is restored
  alphaCommunity = (
    await alpha.getCommunity({ id: communityId }).then(expectSuccess)
  ).community_view;
  expect(alphaCommunity.tags.length).toBe(1);
  // verify tag federated

  betaCommunity = (await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.tags.length === 1,
  ))!;
  assertCommunityFederation(alphaCommunity, betaCommunity);

  // List tags
  alphaCommunity = (
    await alpha.getCommunity({ id: communityId }).then(expectSuccess)
  ).community_view;
  expect(alphaCommunity.tags.length).toBe(1);
  expect(alphaCommunity.tags.find(t => t.id === createRes.id)?.name).toBe(
    tagName,
  );
});

test("Remote mod creates and updates post tag", async () => {
  // Create a community
  let communityRes = await createCommunity(alpha).then(expectSuccess);
  let alphaCommunity = communityRes.community_view;

  // add gamma as remote mod
  const gammaOnAlpha = await resolvePerson(
    alpha,
    "lemmy_gamma@lemmy-gamma:8561",
  );

  const form: AddModToCommunity = {
    community_id: communityRes.community_view.community.id,
    person_id: gammaOnAlpha?.person.id as number,
    added: true,
  };
  await alpha.addModToCommunity(form);

  const gammaCommunity = await resolveCommunity(
    gamma,
    alphaCommunity.community.ap_id,
  );

  // Remote mod gamma creates tag
  const tag1Name = "news";
  const createForm1: CreateCommunityTag = {
    name: tag1Name,
    community_id: gammaCommunity!.community.id,
  };
  const tag1Res = await gamma
    .createCommunityTag(createForm1)
    .then(expectSuccess);
  expect(tag1Res.id).toBeDefined();

  await waitUntilSuccess(
    () => getCommunity(alpha, communityRes.community_view.community.id),
    c => c.community_view.tags.length == 1,
  );

  const betaCommunity = await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    c => c?.tags.length == 1,
  );

  // follow from beta
  await followCommunity(beta, true, betaCommunity!.community.id);
  await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.community_actions?.follow_state == "accepted",
  );

  // Create a post with tag
  const postRes = await beta
    .createPost({
      name: randomString(10),
      community_id: betaCommunity!.community.id,
      tags: [betaCommunity!.tags[0].id],
    })
    .then(expectSuccess);
  expect(postRes.post_view.post.id).toBeDefined();
  expect(postRes.post_view.post.id).toBe(postRes.post_view.post.id);
  expect(postRes.post_view.tags?.length).toBe(1);
  expect(postRes.post_view.tags?.map(t => t.id)).toEqual([
    betaCommunity!.tags[0].id,
  ]);

  // wait post tags federated
  const alphaPost = await waitForPost(
    alpha,
    postRes.post_view.post,
    p => (p?.tags.length ?? 0) > 0,
  );
  expect(alphaPost?.tags.length).toBe(1);
  expect(alphaPost?.tags.map(t => t.ap_id)).toEqual([tag1Res.ap_id]);

  // Mod on alpha updates post to remove one tag
  communityRes = await getCommunity(
    alpha,
    communityRes.community_view.community.id,
  ).then(expectSuccess);
  alphaCommunity = communityRes.community_view;
  const updateRes = await alpha
    .modEditPost({
      post_id: alphaPost!.post.id,
      tags: [alphaCommunity.tags[0].id],
    })
    .then(expectSuccess);
  expect(updateRes.post_view.post.ap_id).toBe(postRes.post_view.post.ap_id);
  expect(updateRes.post_view.tags?.length).toBe(1);
  expect(updateRes.post_view.tags?.[0].id).toBe(alphaCommunity.tags[0].id);

  // wait post tags federated
  const betaPost = await waitForPost(beta, postRes.post_view.post, p => {
    return (p?.tags.length ?? 0) === 1;
  });
  expect(betaPost?.tags.map(t => t.ap_id)).toEqual([tag1Res.ap_id]);
});
