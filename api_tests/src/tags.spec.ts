jest.setTimeout(120000);

import {
  alpha,
  setupLogins,
  createCommunity,
  unfollows,
  randomString,
  createPost,
} from "./shared";
import { CreateCommunityTag } from "lemmy-js-client/dist/types/CreateCommunityTag";
import { UpdateCommunityTag } from "lemmy-js-client/dist/types/UpdateCommunityTag";
import { DeleteCommunityTag } from "lemmy-js-client/dist/types/DeleteCommunityTag";
import { EditPost } from "lemmy-js-client";

beforeAll(setupLogins);
afterAll(unfollows);

test("Create, update, delete community tag", async () => {
  // Create a community first
  let communityRes = await createCommunity(alpha);
  const communityId = communityRes.community_view.community.id;

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
  let listRes = await alpha.getCommunity({ id: communityId });
  expect(listRes.community_view.post_tags.length).toBe(1);
  expect(
    listRes.community_view.post_tags.find(t => t.id === createRes.id)
      ?.display_name,
  ).toBe(newTagName);

  // Delete the tag
  let deleteForm: DeleteCommunityTag = {
    tag_id: createRes.id,
  };
  let deleteRes = await alpha.deleteCommunityTag(deleteForm);
  expect(deleteRes.id).toBe(createRes.id);

  // Verify tag is deleted
  listRes = await alpha.getCommunity({ id: communityId });
  expect(
    listRes.community_view.post_tags.find(t => t.id === createRes.id),
  ).toBeUndefined();
  expect(listRes.community_view.post_tags.length).toBe(0);
});

test("Update post tags", async () => {
  // Create a community
  let communityRes = await createCommunity(alpha);
  const communityId = communityRes.community_view.community.id;

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

  // Update post to remove one tag
  updateForm.tags = [tag1Res.id];
  updateRes = await alpha.editPost(updateForm);
  expect(updateRes.post_view.post.id).toBe(postRes.post_view.post.id);
  expect(updateRes.post_view.tags?.length).toBe(1);
  expect(updateRes.post_view.tags?.[0].id).toBe(tag1Res.id);
});

test("Post author can update post tags", async () => {
  // Create a community
  let communityRes = await createCommunity(alpha);
  const communityId = communityRes.community_view.community.id;

  // Create a tag
  const tagName = randomString(10);
  let createForm: CreateCommunityTag = {
    display_name: tagName,
    community_id: communityId,
  };
  let tagRes = await alpha.createCommunityTag(createForm);
  expect(tagRes.id).toBeDefined();

  let postRes = await createPost(
    alpha,
    communityId,
    "https://example.com/",
    "post with tags",
  );
  expect(postRes.post_view.post.id).toBeDefined();

  // Alpha should be able to update tags on their own post
  let updateForm: EditPost = {
    post_id: postRes.post_view.post.id,
    tags: [tagRes.id],
  };
  let updateRes = await alpha.editPost(updateForm);
  expect(updateRes.post_view.post.id).toBe(postRes.post_view.post.id);
  expect(updateRes.post_view.tags?.length).toBe(1);
  expect(updateRes.post_view.tags?.[0].id).toBe(tagRes.id);
});
