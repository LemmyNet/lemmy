// Requires env vars:
//
// LEMMY_SERVER_URL (ex http://localhost:8536)
// LEMMY_LOGIN
// LEMMY_PASSWORD

jest.setTimeout(120000);

import {
  CommentSortType,
  LemmyHttp,
  Login,
  PostSortType,
} from "lemmy-js-client";
import { fetchFunction } from "./shared";

const defaultServerUrl = "http://localhost:8536";
const defaultLogin = "lemmy";
const defaultPassword = "lemmylemmy";

// Post without a url
const textPost = 43615136;

// Post with a url
const postWithUrl = 43614333;

// A post with ~2.2k comments
const postWithLotsOfComments = 3192572;

let api: LemmyHttp;
let report: string[] = [];

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
afterAll(() => {
  console.log(report.join("\n"));
});

test("List posts with different sorts", async () => {
  report.push("\n# List posts with different sorts \n");
  report.push("sort | time");
  report.push("--- | ---");
  const sortTypes: PostSortType[] = [
    "active",
    "hot",
    "new",
    "top",
    "controversial",
  ];
  for (let sort of sortTypes) {
    const time = await timeApiCalls(() => api.getPosts({ sort }));
    report.push(`${sort} | ${formatMs(time)}`);
  }
});

test("List posts with higher pages", async () => {
  report.push("\n# List posts with higher pages\n");
  report.push("page # | time");
  report.push("--- | ---");
  let page_cursor: string | undefined = undefined;

  let diffs = [];
  for (let i = 0; i < 10; i++) {
    const res = await timeApiCall(() =>
      api.getPosts({ sort: "new", page_cursor }),
    );
    page_cursor = res.res.next_page;
    diffs.push(res.diff);
    report.push(`${i} | ${formatMs(res.diff)}`);
  }

  const avg = average(diffs);
  report.push(`avg | ${formatMs(avg)}`);
});

test.skip("Get a post", async () => {
  report.push("\n# Get a post\n");
  report.push("type | time");
  report.push("--- | ---");

  const getUrlPost = await timeApiCalls(() => api.getPost({ id: postWithUrl }));
  report.push(`url post | ${formatMs(getUrlPost)}`);

  const getTextPost = await timeApiCalls(() => api.getPost({ id: textPost }));
  report.push(`text post | ${formatMs(getTextPost)}`);
});

test("Get comments with different sorts", async () => {
  report.push("\n# Get comments\n");
  report.push("sort | time");
  report.push("--- | ---");

  const sortTypes: CommentSortType[] = [
    "hot",
    "new",
    "old",
    "top",
    "controversial",
  ];

  for (let sort of sortTypes) {
    const time = await timeApiCalls(() =>
      api.getComments({ post_id: postWithLotsOfComments, sort }),
    );
    report.push(`${sort} | ${formatMs(time)}`);
  }
});

test("Get comments slim", async () => {
  report.push("\n# Get comments slim\n");
  report.push("sort | time");
  report.push("--- | ---");
  const getCommentsSlim = await timeApiCalls(() =>
    api.getCommentsSlim({ post_id: postWithLotsOfComments }),
  );
  report.push(`getCommentsSlim: ${formatMs(getCommentsSlim)}`);
});

type Result<T> = {
  diff: number;
  res: T;
};

async function timeApiCall<T>(promise: () => Promise<T>): Promise<Result<T>> {
  const start = performance.now();
  const res = await promise();
  const end = performance.now();
  const diff = timeDiff(start, end);
  return {
    diff,
    res,
  };
}

async function timeApiCalls<T>(promise: () => Promise<T>, times = 10) {
  let diffs = [];
  for (let i = 0; i < times; i++) {
    const diff = (await timeApiCall(promise)).diff;
    diffs.push(diff);
  }
  return average(diffs);
}

function timeDiff(start: number, end: number) {
  return end - start;
}

function average(arr: number[]) {
  return arr.reduce((p, c) => p + c, 0) / arr.length;
}

function formatMs(time: number): string {
  return `${time.toFixed(0)}ms`;
}
