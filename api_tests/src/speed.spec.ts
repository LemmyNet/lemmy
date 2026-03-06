// Requires env vars:
//
// LEMMY_SERVER_URL (ex http://localhost:8536)
// LEMMY_LOGIN
// LEMMY_PASSWORD

jest.setTimeout(120000);

import { LemmyHttp, Login, PostSortType } from "lemmy-js-client";
import { fetchFunction } from "./shared";

const defaultServerUrl = "http://localhost:8536";
const defaultLogin = "lemmy";
const defaultPassword = "lemmylemmy";

const samplePostId = 43615136;
let api: LemmyHttp;

beforeAll(async () => {
  api = new LemmyHttp(process.env.LEMMY_SERVER_URL ?? defaultServerUrl, {
    fetchFunction,
  });
  const login: Login = {
    username_or_email: process.env.LEMMY_LOGIN ?? defaultLogin,
    password: process.env.LEMMY_PASSWORD ?? defaultPassword,
  };
  const res = await api.login(login);
  api.setHeaders({ Authorization: `Bearer ${res.jwt ?? ""}` });
});

test("List posts with different sorts", async () => {
  const sortTypes: PostSortType[] = [
    "active",
    "hot",
    "new",
    "top",
    "controversial",
  ];
  sortTypes.forEach(async sortType => {
    const time = await timeApiCalls(() => api.getPosts({ sort: "new" }));
    console.log(`list post | ${sortType} | ${time}`);
  });
});

test("Get a post", async () => {
  const getPost = await timeApiCalls(() => api.getPost({ id: samplePostId }));
  console.log(`getPost: ${getPost}`);
});

test("Get comments", async () => {
  const getComments = await timeApiCalls(() =>
    api.getComments({ post_id: samplePostId }),
  );
  console.log(`getComments: ${getComments}`);
});

test("Get comments slim", async () => {
  const getCommentsSlim = await timeApiCalls(() =>
    api.getCommentsSlim({ post_id: samplePostId }),
  );
  console.log(`getCommentsSlim: ${getCommentsSlim}`);
});

async function timeApiCall<T>(promise: () => Promise<T>): Promise<number> {
  const start = performance.now();
  await promise();
  const end = performance.now();
  return diff(start, end);
}

async function timeApiCalls<T>(promise: () => Promise<T>, times = 10) {
  let diffs = [];
  for (let i = 0; i < times; i++) {
    const diff = await timeApiCall(promise);
    diffs.push(diff);
  }
  return average(diffs);
}

function diff(start: number, end: number) {
  return end - start;
}

function average(arr: number[]) {
  return arr.reduce((p, c) => p + c, 0) / arr.length;
}
