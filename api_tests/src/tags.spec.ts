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
} from "./shared";
import { CreateCommunityTag } from "lemmy-js-client/dist/types/CreateCommunityTag";
import { UpdateCommunityTag } from "lemmy-js-client/dist/types/UpdateCommunityTag";
import { DeleteCommunityTag } from "lemmy-js-client/dist/types/DeleteCommunityTag";
import { EditPost } from "lemmy-js-client";

beforeAll(setupLogins);
afterAll(unfollows);

test("Create, update, delete community tag", async () => {
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
    g => g?.community_actions!.follow_state == "Accepted",
  );
  const communityId = alphaCommunity.community.id;

  // Create a tag
  const tagName = randomString(10);
  let createForm: CreateCommunityTag = {
    display_name: tagName,
    community_id: communityId,
  };
  let createRes = await alpha.createCommunityTag(createForm);
  expect(createRes.id).toBeDefined();
  expect(createRes.display_name).toBe(tagName);
  expect(createRes.community_id).toBe(communityId);

  alphaCommunity = (await alpha.getCommunity({ id: communityId }))
    .community_view;
  expect(alphaCommunity.post_tags.length).toBe(1);
  // verify tag federated

  betaCommunity = (await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.post_tags.length === 1,
  ))!;
  assertCommunityFederation(alphaCommunity, betaCommunity);

  // Update the tag
  const newTagName = randomString(10);
  let updateForm: UpdateCommunityTag = {
    tag_id: createRes.id,
    display_name: newTagName,
  };
  let updateRes = await alpha.updateCommunityTag(updateForm);
  expect(updateRes.id).toBe(createRes.id);
  expect(updateRes.display_name).toBe(newTagName);
  expect(updateRes.community_id).toBe(communityId);

  // List tags
  alphaCommunity = (await alpha.getCommunity({ id: communityId }))
    .community_view;
  expect(alphaCommunity.post_tags.length).toBe(1);
  expect(
    alphaCommunity.post_tags.find(t => t.id === createRes.id)?.display_name,
  ).toBe(newTagName);

  // Verify tag update federated
  betaCommunity = (await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g =>
      g!.post_tags.find(t => t.ap_id === createRes.ap_id)?.display_name ===
      newTagName,
  ))!;
  assertCommunityFederation(alphaCommunity, betaCommunity);

  // Delete the tag
  let deleteForm: DeleteCommunityTag = {
    tag_id: createRes.id,
  };
  let deleteRes = await alpha.deleteCommunityTag(deleteForm);
  expect(deleteRes.id).toBe(createRes.id);

  // Verify tag is deleted
  alphaCommunity = (await alpha.getCommunity({ id: communityId }))
    .community_view;
  expect(
    alphaCommunity.post_tags.find(t => t.id === createRes.id),
  ).toBeUndefined();
  expect(alphaCommunity.post_tags.length).toBe(0);

  // Verify tag deletion federated
  betaCommunity = (await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.post_tags.length === 0,
  ))!;
  assertCommunityFederation(alphaCommunity, betaCommunity);
});

test("Create and update post tags", async () => {
  // Create a community
  let communityRes = await createCommunity(alpha);
  const communityId = communityRes.community_view.community.id;

  // follow from beta
  const alphaCommunity = communityRes.community_view;
  let betaCommunity = await resolveCommunity(
    beta,
    alphaCommunity.community.ap_id,
  );
  if (!betaCommunity) {
    throw "Missing gamma community";
  }
  await followCommunity(beta, true, betaCommunity.community.id);
  await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.community_actions?.follow_state == "Accepted",
  );

  // Create two tags
  const tag1Name = randomString(10);
  let createForm1: CreateCommunityTag = {
    display_name: tag1Name,
    community_id: communityId,
  };
  let tag1Res = await alpha.createCommunityTag(createForm1);
  expect(tag1Res.id).toBeDefined();

  const tag2Name = randomString(10);
  let createForm2: CreateCommunityTag = {
    display_name: tag2Name,
    community_id: communityId,
  };
  let tag2Res = await alpha.createCommunityTag(createForm2);
  expect(tag2Res.id).toBeDefined();

  // Create a post
  let postRes = await alpha.createPost({
    name: randomString(10),
    community_id: communityId,
  });
  expect(postRes.post_view.post.id).toBeDefined();

  // Wait post federated
  await waitForPost(beta, postRes.post_view.post);

  // Update post tags
  let updateForm: EditPost = {
    post_id: postRes.post_view.post.id,
    tags: [tag1Res.id, tag2Res.id],
  };
  let updateRes = await alpha.editPost(updateForm);
  expect(updateRes.post_view.post.id).toBe(postRes.post_view.post.id);
  expect(updateRes.post_view.tags?.length).toBe(2);
  expect(updateRes.post_view.tags?.map(t => t.id).sort()).toEqual(
    [tag1Res.id, tag2Res.id].sort(),
  );

  // wait post tags federated

  let betaView = await waitForPost(
    beta,
    postRes.post_view.post,
    p => (p?.tags.length ?? 0) > 0,
  );
  expect(betaView?.tags.length).toBe(2);
  expect(betaView?.tags.map(t => t.ap_id).sort()).toEqual(
    [tag1Res.ap_id, tag2Res.ap_id].sort(),
  );

  // Update post to remove one tag
  updateForm.tags = [tag1Res.id];
  updateRes = await alpha.editPost(updateForm);
  expect(updateRes.post_view.post.id).toBe(postRes.post_view.post.id);
  expect(updateRes.post_view.tags?.length).toBe(1);
  expect(updateRes.post_view.tags?.[0].id).toBe(tag1Res.id);

  // wait post tags federated

  betaView = await waitForPost(
    beta,
    postRes.post_view.post,
    p => (p?.tags.length ?? 0) === 1,
  );
  expect(betaView?.tags.map(t => t.ap_id)).toEqual([tag1Res.ap_id]);
});
