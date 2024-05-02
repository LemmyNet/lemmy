jest.setTimeout(120000);

import {
  UploadImage,
  DeleteImage,
  PurgePerson,
  PurgePost,
} from "lemmy-js-client";
import {
  alpha,
  alphaImage,
  alphaUrl,
  beta,
  betaUrl,
  createCommunity,
  createPost,
  deleteAllImages,
  epsilon,
  followCommunity,
  gamma,
  getSite,
  imageFetchLimit,
  registerUser,
  resolveBetaCommunity,
  resolveCommunity,
  resolvePost,
  setupLogins,
  waitForPost,
  unfollows,
  getPost,
  waitUntil,
  randomString,
  createPostWithThumbnail,
} from "./shared";
const downloadFileSync = require("download-file-sync");

beforeAll(setupLogins);

afterAll(unfollows);

test("Upload image and delete it", async () => {
  // Before running this test, you need to delete all previous images in the DB
  await deleteAllImages(alpha);

  // Upload test image. We use a simple string buffer as pictrs doesn't require an actual image
  // in testing mode.
  const upload_form: UploadImage = {
    image: Buffer.from("test"),
  };
  const upload = await alphaImage.uploadImage(upload_form);
  expect(upload.files![0].file).toBeDefined();
  expect(upload.files![0].delete_token).toBeDefined();
  expect(upload.url).toBeDefined();
  expect(upload.delete_url).toBeDefined();

  // ensure that image download is working. theres probably a better way to do this
  const content = downloadFileSync(upload.url);
  expect(content.length).toBeGreaterThan(0);

  // Ensure that it comes back with the list_media endpoint
  const listMediaRes = await alphaImage.listMedia();
  expect(listMediaRes.images.length).toBe(1);

  // Ensure that it also comes back with the admin all images
  const listAllMediaRes = await alphaImage.listAllMedia({
    limit: imageFetchLimit,
  });

  // This number comes from all the previous thumbnails fetched in other tests.
  const previousThumbnails = 1;
  expect(listAllMediaRes.images.length).toBe(previousThumbnails);

  // The deleteUrl is a combination of the endpoint, delete token, and alias
  let firstImage = listMediaRes.images[0];
  let deleteUrl = `${alphaUrl}/pictrs/image/delete/${firstImage.local_image.pictrs_delete_token}/${firstImage.local_image.pictrs_alias}`;
  expect(deleteUrl).toBe(upload.delete_url);

  // Make sure the uploader is correct
  expect(firstImage.person.actor_id).toBe(
    `http://lemmy-alpha:8541/u/lemmy_alpha`,
  );

  // delete image
  const delete_form: DeleteImage = {
    token: upload.files![0].delete_token,
    filename: upload.files![0].file,
  };
  const delete_ = await alphaImage.deleteImage(delete_form);
  expect(delete_).toBe(true);

  // ensure that image is deleted
  const content2 = downloadFileSync(upload.url);
  expect(content2).toBe("");

  // Ensure that it shows the image is deleted
  const deletedListMediaRes = await alphaImage.listMedia();
  expect(deletedListMediaRes.images.length).toBe(0);

  // Ensure that the admin shows its deleted
  const deletedListAllMediaRes = await alphaImage.listAllMedia({
    limit: imageFetchLimit,
  });
  expect(deletedListAllMediaRes.images.length).toBe(previousThumbnails - 1);
});

test("Purge user, uploaded image removed", async () => {
  let user = await registerUser(alphaImage, alphaUrl);

  // upload test image
  const upload_form: UploadImage = {
    image: Buffer.from("test"),
  };
  const upload = await user.uploadImage(upload_form);
  expect(upload.files![0].file).toBeDefined();
  expect(upload.files![0].delete_token).toBeDefined();
  expect(upload.url).toBeDefined();
  expect(upload.delete_url).toBeDefined();

  // ensure that image download is working. theres probably a better way to do this
  const content = downloadFileSync(upload.url);
  expect(content.length).toBeGreaterThan(0);

  // purge user
  let site = await getSite(user);
  const purgeForm: PurgePerson = {
    person_id: site.my_user!.local_user_view.person.id,
  };
  const delete_ = await alphaImage.purgePerson(purgeForm);
  expect(delete_.success).toBe(true);

  // ensure that image is deleted
  const content2 = downloadFileSync(upload.url);
  expect(content2).toBe("");
});

test("Purge post, linked image removed", async () => {
  let user = await registerUser(beta, betaUrl);

  // upload test image
  const upload_form: UploadImage = {
    image: Buffer.from("test"),
  };
  const upload = await user.uploadImage(upload_form);
  expect(upload.files![0].file).toBeDefined();
  expect(upload.files![0].delete_token).toBeDefined();
  expect(upload.url).toBeDefined();
  expect(upload.delete_url).toBeDefined();

  // ensure that image download is working. theres probably a better way to do this
  const content = downloadFileSync(upload.url);
  expect(content.length).toBeGreaterThan(0);

  let community = await resolveBetaCommunity(user);
  let post = await createPost(
    user,
    community.community!.community.id,
    upload.url,
  );
  expect(post.post_view.post.url).toBe(upload.url);

  // purge post

  const purgeForm: PurgePost = {
    post_id: post.post_view.post.id,
  };
  const delete_ = await beta.purgePost(purgeForm);
  expect(delete_.success).toBe(true);

  // ensure that image is deleted
  const content2 = downloadFileSync(upload.url);
  expect(content2).toBe("");
});

test("Images in remote post are proxied if setting enabled", async () => {
  let user = await registerUser(beta, betaUrl);
  let community = await createCommunity(gamma);

  const upload_form: UploadImage = {
    image: Buffer.from("test"),
  };
  const upload = await user.uploadImage(upload_form);
  let post = await createPost(
    gamma,
    community.community_view.community.id,
    upload.url,
    "![](http://example.com/image2.png)",
  );
  expect(post.post_view.post).toBeDefined();

  // remote image gets proxied after upload
  expect(
    post.post_view.post.url?.startsWith(
      "http://lemmy-gamma:8561/api/v3/image_proxy?url",
    ),
  ).toBeTruthy();
  expect(
    post.post_view.post.body?.startsWith(
      "![](http://lemmy-gamma:8561/api/v3/image_proxy?url",
    ),
  ).toBeTruthy();

  let epsilonPost = await resolvePost(epsilon, post.post_view.post);
  expect(epsilonPost.post).toBeDefined();

  // remote image gets proxied after federation
  expect(
    epsilonPost.post!.post.url?.startsWith(
      "http://lemmy-epsilon:8581/api/v3/image_proxy?url",
    ),
  ).toBeTruthy();
  expect(
    epsilonPost.post!.post.body?.startsWith(
      "![](http://lemmy-epsilon:8581/api/v3/image_proxy?url",
    ),
  ).toBeTruthy();
});

test("No image proxying if setting is disabled", async () => {
  let user = await registerUser(beta, betaUrl);
  let community = await createCommunity(alpha);
  let betaCommunity = await resolveCommunity(
    beta,
    community.community_view.community.actor_id,
  );
  await followCommunity(beta, true, betaCommunity.community!.community.id);

  const upload_form: UploadImage = {
    image: Buffer.from("test"),
  };
  const upload = await user.uploadImage(upload_form);
  let post = await createPost(
    alpha,
    community.community_view.community.id,
    upload.url,
    "![](http://example.com/image2.png)",
  );
  expect(post.post_view.post).toBeDefined();

  // remote image doesn't get proxied after upload
  expect(
    post.post_view.post.url?.startsWith("http://127.0.0.1:8551/pictrs/image/"),
  ).toBeTruthy();
  expect(post.post_view.post.body).toBe("![](http://example.com/image2.png)");

  let betaPost = await waitForPost(
    beta,
    post.post_view.post,
    res => res?.post.alt_text != null,
  );
  expect(betaPost.post).toBeDefined();

  // remote image doesn't get proxied after federation
  expect(
    betaPost.post.url?.startsWith("http://127.0.0.1:8551/pictrs/image/"),
  ).toBeTruthy();
  expect(betaPost.post.body).toBe("![](http://example.com/image2.png)");

  // Make sure the alt text got federated
  expect(post.post_view.post.alt_text).toBe(betaPost.post.alt_text);
});

test("Make regular post, and give it a custom thumbnail", async () => {
  const uploadForm1: UploadImage = {
    image: Buffer.from("testRegular1"),
  };
  const upload1 = await alphaImage.uploadImage(uploadForm1);

  const community = await createCommunity(alphaImage);

  // Use wikipedia since it has an opengraph image
  const wikipediaUrl = "https://wikipedia.org/";

  let post = await createPostWithThumbnail(
    alphaImage,
    community.community_view.community.id,
    wikipediaUrl,
    upload1.url!,
  );

  // Wait for the metadata to get fetched, since this is backgrounded now
  post = await waitUntil(
    () => getPost(alphaImage, post.post_view.post.id),
    p => p.post_view.post.thumbnail_url != undefined,
  );
  expect(post.post_view.post.url).toBe(wikipediaUrl);
  // Make sure it uses custom thumbnail
  expect(post.post_view.post.thumbnail_url).toBe(upload1.url);
});

test("Create an image post, and make sure a custom thumbnail doesn't overwrite it", async () => {
  const uploadForm1: UploadImage = {
    image: Buffer.from("test1"),
  };
  const upload1 = await alphaImage.uploadImage(uploadForm1);

  const uploadForm2: UploadImage = {
    image: Buffer.from("test2"),
  };
  const upload2 = await alphaImage.uploadImage(uploadForm2);

  const community = await createCommunity(alphaImage);

  let post = await createPostWithThumbnail(
    alphaImage,
    community.community_view.community.id,
    upload1.url!,
    upload2.url!,
  );
  post = await waitUntil(
    () => getPost(alphaImage, post.post_view.post.id),
    p => p.post_view.post.thumbnail_url != undefined,
  );
  expect(post.post_view.post.url).toBe(upload1.url);
  // Make sure the custom thumbnail is ignored
  expect(post.post_view.post.thumbnail_url == upload2.url).toBe(false);
});
