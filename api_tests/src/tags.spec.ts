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
} from "./shared";
import { CreateCommunityTag } from "lemmy-js-client/dist/types/CreateCommunityTag";
import { DeleteCommunityTag } from "lemmy-js-client/dist/types/DeleteCommunityTag";
import { AddModToCommunity } from "lemmy-js-client";

beforeAll(setupLogins);
afterAll(unfollows);

test("Create, delete and restore a community tag", async () => {
  // Create a community first
  const communityRes = await createCommunity(alpha);
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
  let createForm: CreateCommunityTag = {
    name: tagName,
    community_id: communityId,
  };
  let createRes = await alpha.createCommunityTag(createForm);
  expect(createRes.id).toBeDefined();
  expect(createRes.name).toBe(tagName);
  expect(createRes.community_id).toBe(communityId);

  alphaCommunity = (await alpha.getCommunity({ id: communityId }))
    .community_view;
  expect(alphaCommunity.tags.length).toBe(1);
  // verify tag federated

  betaCommunity = (await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.tags.length === 1,
  ))!;
  assertCommunityFederation(alphaCommunity, betaCommunity);

  // List tags
  alphaCommunity = (await alpha.getCommunity({ id: communityId }))
    .community_view;
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
  let deleteForm: DeleteCommunityTag = {
    tag_id: createRes.id,
    delete: true,
  };
  let deleteRes = await alpha.deleteCommunityTag(deleteForm);
  expect(deleteRes.id).toBe(createRes.id);

  // Verify tag is deleted
  alphaCommunity = (await alpha.getCommunity({ id: communityId }))
    .community_view;
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
  let deleteFormRestoration: DeleteCommunityTag = {
    tag_id: createRes.id,
    delete: false,
  };
  let deleteRestorationRes = await alpha.deleteCommunityTag(
    deleteFormRestoration,
  );
  expect(deleteRestorationRes.id).toBe(createRes.id);

  // Verify tag is restored
  alphaCommunity = (await alpha.getCommunity({ id: communityId }))
    .community_view;
  expect(alphaCommunity.tags.length).toBe(1);
  // verify tag federated

  betaCommunity = (await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.tags.length === 1,
  ))!;
  assertCommunityFederation(alphaCommunity, betaCommunity);

  // List tags
  alphaCommunity = (await alpha.getCommunity({ id: communityId }))
    .community_view;
  expect(alphaCommunity.tags.length).toBe(1);
  expect(alphaCommunity.tags.find(t => t.id === createRes.id)?.name).toBe(
    tagName,
  );
});

test("Create and update post tags", async () => {
  // Create a community
  let communityRes = await createCommunity(alpha);
  let alphaCommunity = communityRes.community_view;

  // add gamma as remote mod
  let gammaOnAlpha = await resolvePerson(alpha, "lemmy_gamma@lemmy-gamma:8561");

  let form: AddModToCommunity = {
    community_id: communityRes.community_view.community.id,
    person_id: gammaOnAlpha?.person.id as number,
    added: true,
  };
  alpha.addModToCommunity(form);

  let gammaCommunity = await resolveCommunity(
    gamma,
    alphaCommunity.community.ap_id,
  );

  // Remote mod gamma create two tags
  const tag1Name = "news";
  let createForm1: CreateCommunityTag = {
    name: tag1Name,
    community_id: gammaCommunity!.community.id,
  };
  let tag1Res = await gamma.createCommunityTag(createForm1);
  expect(tag1Res.id).toBeDefined();

  const tag2Name = "meme";
  let createForm2: CreateCommunityTag = {
    name: tag2Name,
    community_id: gammaCommunity!.community.id,
  };
  let tag2Res = await gamma.createCommunityTag(createForm2);
  expect(tag2Res.id).toBeDefined();

  await waitUntil(
    () => getCommunity(alpha, communityRes.community_view.community.id),
    c => c.community_view.tags.length > 0,
  );

  let betaCommunity = await resolveCommunity(
    beta,
    alphaCommunity.community.ap_id,
  );

  // follow from beta
  await followCommunity(beta, true, betaCommunity!.community.id);
  await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.community_actions?.follow_state == "accepted",
  );

  // Create a post with tags
  let postRes = await beta.createPost({
    name: randomString(10),
    community_id: betaCommunity!.community.id,
    tags: [betaCommunity!.tags[0].id, betaCommunity!.tags[1].id],
  });
  expect(postRes.post_view.post.id).toBeDefined();
  expect(postRes.post_view.post.id).toBe(postRes.post_view.post.id);
  expect(postRes.post_view.tags?.length).toBe(2);
  expect(postRes.post_view.tags?.map(t => t.id).sort()).toEqual(
    [betaCommunity!.tags[0].id, betaCommunity!.tags[1].id].sort(),
  );

  // wait post tags federated
  let alphaPost = await waitForPost(
    alpha,
    postRes.post_view.post,
    p => (p?.tags.length ?? 0) > 0,
  );
  expect(alphaPost?.tags.length).toBe(2);
  expect(alphaPost?.tags.map(t => t.ap_id).sort()).toEqual(
    [tag1Res.ap_id, tag2Res.ap_id].sort(),
  );

  // Mod on alpha updates post to remove one tag
  communityRes = await getCommunity(
    alpha,
    communityRes.community_view.community.id,
  );
  alphaCommunity = communityRes.community_view;
  let updateRes = await alpha.modEditPost({
    post_id: alphaPost.post.id,
    tags: [alphaCommunity!.tags[0].id],
  });
  expect(updateRes.post_view.post.ap_id).toBe(postRes.post_view.post.ap_id);
  expect(updateRes.post_view.tags?.length).toBe(1);
  expect(updateRes.post_view.tags?.[0].id).toBe(alphaCommunity!.tags[0].id);

  // wait post tags federated
  let betaPost = await waitForPost(beta, postRes.post_view.post, p => {
    return (p?.tags.length ?? 0) === 1;
  });
  expect(betaPost?.tags.map(t => t.ap_id)).toEqual([tag1Res.ap_id]);
});
