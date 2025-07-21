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
import { DeleteCommunityTag } from "lemmy-js-client/dist/types/DeleteCommunityTag";
import { EditPost } from "lemmy-js-client";

beforeAll(setupLogins);
afterAll(unfollows);

test("Create, and delete a community tag", async () => {
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
    name: tagName,
    community_id: communityId,
  };
  let createRes = await alpha.createCommunityTag(createForm);
  expect(createRes.id).toBeDefined();
  expect(createRes.name).toBe(tagName);
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

  // List tags
  alphaCommunity = (await alpha.getCommunity({ id: communityId }))
    .community_view;
  expect(alphaCommunity.post_tags.length).toBe(1);
  expect(alphaCommunity.post_tags.find(t => t.id === createRes.id)?.name).toBe(
    tagName,
  );

  // Verify tag update federated
  betaCommunity = (await waitUntil(
    () => resolveCommunity(beta, alphaCommunity.community.ap_id),
    g => g!.post_tags.find(t => t.ap_id === createRes.ap_id)?.name === tagName,
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

  // Create two tags
  const tag1Name = "news";
  let createForm1: CreateCommunityTag = {
    name: tag1Name,
    community_id: communityId,
  };
  let tag1Res = await alpha.createCommunityTag(createForm1);
  expect(tag1Res.id).toBeDefined();

  const tag2Name = "meme";
  let createForm2: CreateCommunityTag = {
    name: tag2Name,
    community_id: communityId,
  };
  let tag2Res = await alpha.createCommunityTag(createForm2);
  expect(tag2Res.id).toBeDefined();

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

  // Create a post
  let postRes = await beta.createPost({
    name: randomString(10),
    community_id: betaCommunity.community.id,
  });
  expect(postRes.post_view.post.id).toBeDefined();

  // Update post tags
  let updateForm: EditPost = {
    post_id: postRes.post_view.post.id,
    tags: [betaCommunity.post_tags[0].id, betaCommunity.post_tags[1].id],
  };
  let updateRes = await beta.editPost(updateForm);
  expect(updateRes.post_view.post.id).toBe(postRes.post_view.post.id);
  expect(updateRes.post_view.tags?.length).toBe(2);
  expect(updateRes.post_view.tags?.map(t => t.id).sort()).toEqual(
    [betaCommunity.post_tags[0].id, betaCommunity.post_tags[1].id].sort(),
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

  // Update post to remove one tag
  updateForm.tags = [betaCommunity.post_tags[0].id];
  updateRes = await beta.editPost(updateForm);
  expect(updateRes.post_view.post.id).toBe(postRes.post_view.post.id);
  expect(updateRes.post_view.tags?.length).toBe(1);
  expect(updateRes.post_view.tags?.[0].id).toBe(betaCommunity.post_tags[0].id);

  // wait post tags federated

  alphaPost = await waitForPost(
    beta,
    postRes.post_view.post,
    p => (p?.tags.length ?? 0) === 1,
  );
  expect(alphaPost?.tags.map(t => t.ap_id)).toEqual([tag1Res.ap_id]);
});
