jest.setTimeout(120000);

import {
  UploadImage,
  DeleteImage,
  PurgePerson,
  PurgePost,
} from "lemmy-js-client";
import {
  alpha,
  alphaUrl,
  beta,
  betaUrl,
  createPost,
  getSite,
  registerUser,
  resolveBetaCommunity,
  setupLogins,
  unfollowRemotes,
} from "./shared";
import fs = require("fs");
const downloadFileSync = require("download-file-sync");

beforeAll(setupLogins);

afterAll(() => {
  unfollowRemotes(alpha);
});

test("Upload image and delete it", async () => {
  // upload test image
  const upload_image = fs.readFileSync("test.png");
  const upload_form: UploadImage = {
    image: upload_image,
  };
  const upload = await alpha.uploadImage(upload_form);
  expect(upload.files![0].file).toBeDefined();
  expect(upload.files![0].delete_token).toBeDefined();
  expect(upload.url).toBeDefined();
  expect(upload.delete_url).toBeDefined();

  // ensure that image download is working. theres probably a better way to do this
  const content = downloadFileSync(upload.url);
  expect(content.length).toBeGreaterThan(0);

  // delete image
  const delete_form: DeleteImage = {
    token: upload.files![0].delete_token,
    filename: upload.files![0].file,
  };
  const delete_ = await alpha.deleteImage(delete_form);
  expect(delete_).toBe(true);

  // ensure that image is deleted
  const content2 = downloadFileSync(upload.url);
  expect(content2).toBe("");
});

test("Purge user, uploaded image removed", async () => {
  let user = await registerUser(alpha, alphaUrl);

  // upload test image
  const upload_image = fs.readFileSync("test.png");
  const upload_form: UploadImage = {
    image: upload_image,
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
  const delete_ = await alpha.purgePerson(purge_form);
  expect(delete_.success).toBe(true);

  // ensure that image is deleted
  const content2 = downloadFileSync(upload.url);
  expect(content2).toBe("");
});

test.only("Purge post, linked image removed", async () => {
  let user = await registerUser(beta, betaUrl);

  // upload test image
  const upload_image = fs.readFileSync("test.png");
  const upload_form: UploadImage = {
    image: upload_image,
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
  const delete_ = await alpha.purgePost(purge_form);
  expect(delete_.success).toBe(true);

  // ensure that image is deleted
  const content2 = downloadFileSync(upload.url);
  expect(content2).toBe("");
});

// TODO: add tests for image purging
