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
  delta,
  epsilon,
  gamma,
  getSite,
  registerUser,
  resolveBetaCommunity,
  resolvePost,
  setupLogins,
  unfollowRemotes,
} from "./shared";
const downloadFileSync = require("download-file-sync");

const imageFetchLimit = 50;

beforeAll(setupLogins);

afterAll(() => {
  unfollowRemotes(alphaImage);
});

test("Upload image and delete it", async () => {
  // Upload test image. We use a simple string buffer as pictrs doesnt require an actual image
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
  const previousThumbnails = 30;
  expect(listAllMediaRes.images.length).toBe(previousThumbnails);

  // The deleteUrl is a combination of the endpoint, delete token, and alias
  let firstImage = listMediaRes.images[0];
  let deleteUrl = `${alphaUrl}/pictrs/image/delete/${firstImage.pictrs_delete_token}/${firstImage.pictrs_alias}`;
  expect(deleteUrl).toBe(upload.delete_url);

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
  const purge_form: PurgePerson = {
    person_id: site.my_user!.local_user_view.person.id,
  };
  const delete_ = await alphaImage.purgePerson(purge_form);
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
  const purge_form: PurgePost = {
    post_id: post.post_view.post.id,
  };
  const delete_ = await beta.purgePost(purge_form);
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

  // remote image doesnt get proxied after upload
  expect(
    post.post_view.post.url?.startsWith("http://127.0.0.1:8551/pictrs/image/"),
  ).toBeTruthy();
  expect(post.post_view.post.body).toBe("![](http://example.com/image2.png)");

  let gammaPost = await resolvePost(delta, post.post_view.post);
  expect(gammaPost.post).toBeDefined();

  // remote image doesnt get proxied after federation
  expect(
    gammaPost.post!.post.url?.startsWith("http://127.0.0.1:8551/pictrs/image/"),
  ).toBeTruthy();
  expect(gammaPost.post!.post.body).toBe("![](http://example.com/image2.png)");

  // Make sure the alt text got federated
  expect(post.post_view.post.alt_text).toBe(gammaPost.post!.post.alt_text);
});
