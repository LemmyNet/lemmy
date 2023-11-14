jest.setTimeout(120000);

import { UploadImage, DeleteImage } from "lemmy-js-client";
import {
  alpha,
  setupLogins,
  unfollowRemotes,
} from "./shared";
import fs = require('fs');

beforeAll(setupLogins);

afterAll(() => {unfollowRemotes(alpha);});

test("Upload image and delete it", async () => {
    // upload test image
    // TODO: this doesnt require separate auth anymore (same for delete)
    let upload_image = fs.readFileSync("test.png");
    let upload_form: UploadImage = {
        image: upload_image,
        auth: undefined,
    };
    let upload = await alpha.uploadImage(upload_form);
    console.log(upload);
    expect(upload.files![0].file).toBeDefined();
    expect(upload.files![0].delete_token).toBeDefined();
    expect(upload.url).toBeDefined();
    expect(upload.delete_url).toBeDefined();

    // ensure that image download is working. theres probably a better way to do this
    var downloadFileSync = require('download-file-sync');
    var content = downloadFileSync(upload.url);
    expect(content.length).toBeGreaterThan(0);

    // delete image
    let delete_form: DeleteImage =  {
      token: upload.files![0].delete_token,
      filename: upload.files![0].file,
      auth: undefined
    };
    // TODO: throws `FetchError: Invalid response body while trying to fetch http://127.0.0.1:8541/pictrs/image/delete/37095c51-b315-42ab-b7a2-86a299f3d913/3e273850-12b4-4fe4-86c6-a35990d2c5df.png: Parse Error: Expected HTTP/`
    let delete_ = await alpha.deleteImage(delete_form);
    console.log(delete_);

    // ensure that image is deleted
    var content2 = downloadFileSync(upload.url);
    expect(content2).toBe(0);
});

// TODO: add tests for image purging