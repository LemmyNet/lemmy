jest.setTimeout(120000);

import {
  UploadImage,
  PurgePerson,
  PurgePost,
  DeleteImageParams,
} from "lemmy-js-client";
import {
  alpha,
  alphaImage,
  alphaUrl,
  beta,
  betaUrl,
  createCommunity,
  createPost,
  deleteAllMedia,
  epsilon,
  followCommunity,
  gamma,
  imageFetchLimit,
  registerUser,
  resolveBetaCommunity,
  resolveCommunity,
  resolvePost,
  setupLogins,
  waitForPost,
  unfollows,
  getPost,
  createPostWithThumbnail,
  sampleImage,
  sampleSite,
  getMyUser,
  expectSuccess,
  waitUntilSuccess,
} from "./shared";

beforeAll(setupLogins);

afterAll(async () => {
  await Promise.allSettled([unfollows(), deleteAllMedia(alpha)]);
});

async function expectProxiedImageContentDisposition(
  url: string,
  filename: string,
) {
  const expectedContentDisposition = `inline; filename="${encodeURIComponent(filename)}"`;
  // Strip max_size so Lemmy requests image/original?proxy= from pict-rs instead of
  // image/process.*?proxy=, which hangs in pict-rs danger-dummy-mode. The
  // Content-Disposition header is set by Lemmy from the URL filename and is
  // identical for both paths.
  const proxyUrl = new URL(url);
  proxyUrl.searchParams.delete("max_size");
  const proxyResponse = await waitUntilSuccess<Response>(
    async () => ({
      state: "success" as const,
      data: await fetch(proxyUrl),
    }),
    response =>
      response.ok &&
      response.headers.get("content-disposition") ===
        expectedContentDisposition,
  );

  expect(proxyResponse.headers.get("content-disposition")).toBe(
    expectedContentDisposition,
  );
}

test("Upload image and delete it", async () => {
  const health = await alpha.imageHealth().then(expectSuccess);
  expect(health.success).toBeTruthy();

  const baseImageCount = await alpha
    .listMediaAdmin({
      limit: imageFetchLimit,
    })
    .then(expectSuccess)
    .then(res => res.items.length);

  // Upload test image. We use a simple string buffer as pictrs doesn't require an actual image
  // in testing mode.
  const upload_form: UploadImage = {
    image: Buffer.from("test"),
  };
  const upload = await alphaImage.uploadImage(upload_form).then(expectSuccess);
  expect(upload.image_url).toBeDefined();
  expect(upload.filename).toBeDefined();

  // ensure that image download is working. theres probably a better way to do this
  const response = await fetch(upload.image_url ?? "");
  const content = await response.text();
  expect(content.length).toBeGreaterThan(0);

  // Ensure that it comes back with the list_media endpoint
  const listMediaRes = await alphaImage.listMedia().then(expectSuccess);
  expect(listMediaRes.items.length).toBe(1);

  // Ensure that it also comes back with the admin all images
  const listMediaAdminRes = await alpha
    .listMediaAdmin({
      limit: imageFetchLimit,
    })
    .then(expectSuccess);

  expect(listMediaAdminRes.items.length).toBeGreaterThanOrEqual(
    baseImageCount + 1,
  );

  // Make sure the uploader is correct
  expect(listMediaRes.items[0].person.ap_id).toBe(
    `http://lemmy-alpha:8541/u/lemmy_alpha`,
  );

  // delete image
  const delete_form: DeleteImageParams = {
    filename: upload.filename,
  };
  const delete_ = await alphaImage.deleteMedia(delete_form).then(expectSuccess);
  expect(delete_.success).toBe(true);

  // ensure that image is deleted
  const response2 = await fetch(upload.image_url ?? "");
  const content2 = await response2.text();
  expect(content2).toBe("");

  // Ensure that it shows the image is deleted
  const deletedListMediaRes = await alphaImage.listMedia().then(expectSuccess);
  expect(deletedListMediaRes.items.length).toBe(0);

  // Ensure that the admin shows its deleted
  const deletedListAllMediaRes = await alphaImage
    .listMediaAdmin({
      limit: imageFetchLimit,
    })
    .then(expectSuccess);
  expect(deletedListAllMediaRes.items.length).toBe(baseImageCount);
});

test("Purge user, uploaded image removed", async () => {
  const user = await registerUser(alphaImage, alphaUrl);

  // upload test image
  const upload_form: UploadImage = {
    image: Buffer.from("test"),
  };
  const upload = await user.uploadImage(upload_form).then(expectSuccess);
  expect(upload.filename).toBeDefined();
  expect(upload.image_url).toBeDefined();

  // ensure that image download is working. theres probably a better way to do this
  const response = await fetch(upload.image_url ?? "");
  const content = await response.text();
  expect(content.length).toBeGreaterThan(0);

  // purge user
  const my_user = await getMyUser(user).then(expectSuccess);
  const purgeForm: PurgePerson = {
    person_id: my_user.local_user_view.person.id,
    reason: "purge",
  };
  const delete_ = await alphaImage.purgePerson(purgeForm).then(expectSuccess);
  expect(delete_.success).toBe(true);

  // ensure that image is deleted
  const response2 = await fetch(upload.image_url ?? "");
  const content2 = await response2.text();
  expect(content2).toBe("");
});

test("Purge post, linked image removed", async () => {
  const user = await registerUser(beta, betaUrl);

  // upload test image
  const upload_form: UploadImage = {
    image: Buffer.from("test"),
  };
  const upload = await user.uploadImage(upload_form).then(expectSuccess);
  expect(upload.filename).toBeDefined();
  expect(upload.image_url).toBeDefined();

  // ensure that image download is working. theres probably a better way to do this
  const response = await fetch(upload.image_url ?? "");
  const content = await response.text();
  expect(content.length).toBeGreaterThan(0);

  const community = await resolveBetaCommunity(user);
  const post = await createPost(
    user,
    community!.community.id,
    upload.image_url,
  ).then(expectSuccess);
  expect(post.post_view.post.url).toBe(upload.image_url);
  expect(post.post_view.image_details).toBeDefined();

  // purge post
  const purgeForm: PurgePost = {
    post_id: post.post_view.post.id,
    reason: "purge",
  };
  const delete_ = await beta.purgePost(purgeForm).then(expectSuccess);
  expect(delete_.success).toBe(true);

  // ensure that image is deleted
  const response2 = await fetch(upload.image_url ?? "");
  const content2 = await response2.text();
  expect(content2).toBe("");
});

test("Images in remote image post are proxied if setting enabled", async () => {
  const expectedFilename = decodeURIComponent(
    new URL(sampleImage).pathname.split("/").pop()!,
  );

  const community = await createCommunity(gamma).then(expectSuccess);
  const postRes = await createPost(
    gamma,
    community.community_view.community.id,
    sampleImage,
    `![](${sampleImage})`,
  ).then(expectSuccess);
  const post = postRes.post_view.post;
  expect(post).toBeDefined();

  // Make sure it fetched the image details
  expect(postRes.post_view.image_details).toBeDefined();

  // remote image gets proxied after upload
  expect(
    post.thumbnail_url?.startsWith(
      "http://lemmy-gamma:8561/api/v4/image/proxy?url",
    ),
  ).toBeTruthy();
  expect(
    post.body?.startsWith("![](http://lemmy-gamma:8561/api/v4/image/proxy?url"),
  ).toBeTruthy();

  // Make sure that it contains `jpg`, to be sure its an image
  expect(post.thumbnail_url?.includes(".jpg")).toBeTruthy();

  // Proxied image should include a Content-Disposition: inline header
  await expectProxiedImageContentDisposition(
    post.thumbnail_url!,
    expectedFilename,
  );

  const epsilonPostRes = await resolvePost(epsilon, postRes.post_view.post);
  expect(epsilonPostRes?.post).toBeDefined();

  // Fetch the post again, the metadata should be backgrounded now
  // Wait for the metadata to get fetched, since this is backgrounded now
  const epsilonPostRes2 = await waitUntilSuccess(
    () => getPost(epsilon, epsilonPostRes!.post.id),
    p => p.post_view.post.thumbnail_url != undefined,
  );
  const epsilonPost = epsilonPostRes2.post_view.post;

  expect(
    epsilonPost.thumbnail_url?.startsWith(
      "http://lemmy-epsilon:8581/api/v4/image/proxy?url",
    ),
  ).toBeTruthy();
  expect(
    epsilonPost.body?.startsWith(
      "![](http://lemmy-epsilon:8581/api/v4/image/proxy?url",
    ),
  ).toBeTruthy();

  // Make sure that it contains `jpg`, to be sure its an image
  expect(epsilonPost.thumbnail_url?.includes(".jpg")).toBeTruthy();

  await expectProxiedImageContentDisposition(
    epsilonPost.thumbnail_url!,
    expectedFilename,
  );
});

test("Thumbnail of remote image link is proxied if setting enabled", async () => {
  const community = await createCommunity(gamma).then(expectSuccess);
  const postRes = await createPost(
    gamma,
    community.community_view.community.id,
    // The sample site metadata thumbnail ends in png
    sampleSite,
  ).then(expectSuccess);
  const post = postRes.post_view.post;
  expect(post).toBeDefined();

  // Wait for the thumbnail (since its backgrounded)
  await waitUntilSuccess(
    () => getPost(gamma, post.id),
    p => p.post_view.post.thumbnail_url != undefined,
  );

  // remote image gets proxied after upload
  expect(
    post.thumbnail_url?.startsWith(
      "http://lemmy-gamma:8561/api/v4/image/proxy?url",
    ),
  ).toBeTruthy();

  // Make sure that it contains `png`, to be sure its an image
  expect(post.thumbnail_url?.includes(".png")).toBeTruthy();

  const epsilonPostRes = await resolvePost(epsilon, postRes.post_view.post);
  expect(epsilonPostRes?.post).toBeDefined();

  const epsilonPostRes2 = await waitUntilSuccess(
    () => getPost(epsilon, epsilonPostRes!.post.id),
    p => p.post_view.post.thumbnail_url != undefined,
  );
  const epsilonPost = epsilonPostRes2.post_view.post;

  expect(
    epsilonPost.thumbnail_url?.startsWith(
      "http://lemmy-epsilon:8581/api/v4/image/proxy?url",
    ),
  ).toBeTruthy();

  // Make sure that it contains `png`, to be sure its an image
  expect(epsilonPost.thumbnail_url?.includes(".png")).toBeTruthy();
});

test("No image proxying if setting is disabled", async () => {
  const user = await registerUser(beta, betaUrl);
  const community = await createCommunity(alpha).then(expectSuccess);
  const betaCommunity = await resolveCommunity(
    beta,
    community.community_view.community.ap_id,
  );
  await followCommunity(beta, true, betaCommunity!.community.id);

  const upload_form: UploadImage = {
    image: Buffer.from("test"),
  };
  const upload = await user.uploadImage(upload_form).then(expectSuccess);
  const post = await createPost(
    alpha,
    community.community_view.community.id,
    upload.image_url,
    `![](${sampleImage})`,
  ).then(expectSuccess);
  expect(post.post_view.post).toBeDefined();

  // remote image doesn't get proxied after upload
  expect(
    post.post_view.post.url?.startsWith("http://lemmy-beta:8551/api/v4/image/"),
  ).toBeTruthy();
  expect(post.post_view.post.body).toBe(`![](${sampleImage})`);

  const betaPost = await waitForPost(beta, post.post_view.post, res => {
    return res?.post.alt_text != null;
  });
  expect(betaPost!.post).toBeDefined();

  // remote image doesn't get proxied after federation
  expect(
    betaPost!.post.url?.startsWith("http://lemmy-beta:8551/api/v4/image/"),
  ).toBeTruthy();
  expect(betaPost!.post.body).toBe(`![](${sampleImage})`);
  // Make sure the alt text got federated
  expect(post.post_view.post.alt_text).toBe(betaPost!.post.alt_text);
});

test("Make regular post, and give it a custom thumbnail", async () => {
  const uploadForm1: UploadImage = {
    image: Buffer.from("testRegular1"),
  };
  const upload1 = await alphaImage.uploadImage(uploadForm1).then(expectSuccess);

  const community = await createCommunity(alphaImage).then(expectSuccess);

  // Use wikipedia since it has an opengraph image
  const wikipediaUrl = "https://wikipedia.org/";

  let post = await createPostWithThumbnail(
    alphaImage,
    community.community_view.community.id,
    wikipediaUrl,
    upload1.image_url,
  ).then(expectSuccess);

  // Wait for the metadata to get fetched, since this is backgrounded now
  post = await waitUntilSuccess(
    () => getPost(alphaImage, post.post_view.post.id),
    p => p.post_view.post.thumbnail_url != undefined,
  );
  expect(post.post_view.post.url).toBe(wikipediaUrl);
  // Make sure it uses custom thumbnail
  expect(post.post_view.post.thumbnail_url).toBe(upload1.image_url);
});

test("Create an image post, and make sure a custom thumbnail doesn't overwrite it", async () => {
  const uploadForm1: UploadImage = {
    image: Buffer.from("test1"),
  };
  const upload1 = await alphaImage.uploadImage(uploadForm1).then(expectSuccess);

  const uploadForm2: UploadImage = {
    image: Buffer.from("test2"),
  };
  const upload2 = await alphaImage.uploadImage(uploadForm2).then(expectSuccess);

  const community = await createCommunity(alphaImage).then(expectSuccess);

  let post = await createPostWithThumbnail(
    alphaImage,
    community.community_view.community.id,
    upload1.image_url,
    upload2.image_url,
  ).then(expectSuccess);
  post = await waitUntilSuccess(
    () => getPost(alphaImage, post.post_view.post.id),
    p => p.post_view.post.thumbnail_url != undefined,
  );
  expect(post.post_view.post.url).toBe(upload1.image_url);
  // Make sure the custom thumbnail is ignored
  expect(post.post_view.post.thumbnail_url == upload2.image_url).toBe(false);
});
