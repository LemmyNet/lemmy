jest.setTimeout(120000);

import { UploadImage, DeleteImage } from "lemmy-js-client";
import { alpha, setupLogins, unfollowRemotes } from "./shared";
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
  console.log(upload);
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

// TODO: add tests for image purging
