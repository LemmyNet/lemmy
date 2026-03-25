// This is meant to be used with an already-filled / production db with lots of history.
// Requires env vars:
//
// LEMMY_SERVER_URL (ex http://localhost:8536)
// LEMMY_LOGIN
// LEMMY_PASSWORD

jest.setTimeout(120000);

import {
  CommentId,
  CommentSortType,
  CommunitySortType,
  LemmyHttp,
  LikeType,
  ListingType,
  Login,
  ModlogKindFilter,
  MultiCommunitySortType,
  NotificationTypeFilter,
  PersonContentType,
  PersonSortType,
  PostId,
  PostSortType,
} from "lemmy-js-client";
import { fetchFunction } from "./shared";
import * as fs from "fs";

const defaultServerUrl = "http://localhost:8536";
const defaultLogin = "lemmy";
const defaultPassword = "lemmylemmy";
const postCommentsMaxDepth = 8;

const samplePerson = "dessalines";
const sampleCommunity = "memes";
/// A multicommunity with several high-volume communities in it.
const sampleMultiCommunity = "test_1";
const searchTerm = "test";

// Post without a url
const textPost: PostId = 43615136;

// Post with a url
const postWithUrl: PostId = 43614333;

// A post with ~2.2k comments
const postWithLotsOfComments: PostId = 3192572;

const sampleComment: CommentId = 24109064;

const commentSortTypes: CommentSortType[] = [
  "hot",
  "top",
  "new",
  "old",
  "controversial",
];

const postSortTypes: PostSortType[] = [
  "active",
  "hot",
  "new",
  "old",
  "top",
  "most_comments",
  "new_comments",
  "controversial",
  "scaled",
];

const listingTypes: ListingType[] = [
  "all",
  "local",
  "subscribed",
  "moderator_view",
  "suggested",
];

const communitySortTypes: CommunitySortType[] = [
  "active_six_months",
  "active_monthly",
  "active_weekly",
  "active_daily",
  "hot",
  "new",
  "old",
  "name_asc",
  "name_desc",
  "comments",
  "posts",
  "subscribers",
  "subscribers_local",
];

const multiCommunitySortTypes: MultiCommunitySortType[] = [
  "new",
  "old",
  "name_asc",
  "name_desc",
  "communities",
  "subscribers",
  "subscribers_local",
];

const personSortTypes: PersonSortType[] = [
  "new",
  "old",
  "post_score",
  "comment_score",
];

const notificationTypes: NotificationTypeFilter[] = [
  "all",
  "mention",
  "reply",
  "subscribed",
  "private_message",
  "mod_action",
];

const personContentTypes: PersonContentType[] = ["all", "comments", "posts"];

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
  const reportMd = report.join("\n");
  fs.writeFileSync("speed_test_report.md", reportMd);
  console.log(reportMd);
});

test("List posts with different sorts", async () => {
  report.push("\n# List posts with different sorts \n");
  report.push("sort | time");
  report.push("--- | ---");
  for (let sort of postSortTypes) {
    const time = await timeApiCalls(() => api.getPosts({ sort }));
    report.push(`${sort} | ${formatMs(time)}`);
  }
});

test("List posts for a community with different sorts", async () => {
  report.push("\n# List posts for a community with different sorts \n");
  report.push("sort | time");
  report.push("--- | ---");
  for (let sort of postSortTypes) {
    const time = await timeApiCalls(() =>
      api.getPosts({ sort, community_name: sampleCommunity }),
    );
    report.push(`${sort} | ${formatMs(time)}`);
  }
});

test("List posts with different listing types", async () => {
  report.push("\n# List posts with different listing types \n");
  report.push("type | time");
  report.push("--- | ---");
  for (let type_ of listingTypes) {
    const time = await timeApiCalls(() => api.getPosts({ type_ }));
    report.push(`${type_} | ${formatMs(time)}`);
  }
});

test("List posts with show hidden", async () => {
  report.push("\n# List posts with show hidden \n");
  const time = await timeApiCalls(() => api.getPosts({ show_hidden: true }));
  report.push(`show hidden: ${formatMs(time)}`);
});

test("List posts with hide read", async () => {
  report.push("\n# List posts with hide read \n");
  const time = await timeApiCalls(() => api.getPosts({ show_read: false }));
  report.push(`show read : ${formatMs(time)}`);
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

test("List posts for a multi-community with different sorts", async () => {
  report.push("\n# List posts for a multi-community with different sorts \n");
  report.push("sort | time");
  report.push("--- | ---");
  for (let sort of postSortTypes) {
    const time = await timeApiCalls(() =>
      api.getPosts({ sort, multi_community_name: sampleMultiCommunity }),
    );
    report.push(`${sort} | ${formatMs(time)}`);
  }
});

test("List communities with different sorts", async () => {
  report.push("\n# List communities with different sorts \n");
  report.push("sort | time");
  report.push("--- | ---");
  for (let sort of communitySortTypes) {
    const time = await timeApiCalls(() => api.listCommunities({ sort }));
    report.push(`${sort} | ${formatMs(time)}`);
  }
});

test("List communities with different listing types", async () => {
  report.push("\n# List communities with different listing types \n");
  report.push("type | time");
  report.push("--- | ---");
  for (let type_ of listingTypes) {
    const time = await timeApiCalls(() => api.listCommunities({ type_ }));
    report.push(`${type_} | ${formatMs(time)}`);
  }
});

test("List multi-communities with different sorts", async () => {
  report.push("\n# List multi-communities with different sorts \n");
  report.push("sort | time");
  report.push("--- | ---");
  for (let sort of multiCommunitySortTypes) {
    const time = await timeApiCalls(() => api.listMultiCommunities({ sort }));
    report.push(`${sort} | ${formatMs(time)}`);
  }
});

test("Get a community", async () => {
  report.push("\n# Get a community \n");
  const time = await timeApiCalls(() =>
    api.getCommunity({ name: sampleCommunity }),
  );
  report.push(`get community: ${formatMs(time)}`);
});

test("Get a post", async () => {
  report.push("\n# Get a post\n");
  report.push("type | time");
  report.push("--- | ---");

  const getUrlPost = await timeApiCalls(() => api.getPost({ id: postWithUrl }));
  report.push(`url post | ${formatMs(getUrlPost)}`);

  const getTextPost = await timeApiCalls(() => api.getPost({ id: textPost }));
  report.push(`text post | ${formatMs(getTextPost)}`);
});

// TODO SLOW
test("Get comments for a post with different sorts", async () => {
  report.push("\n# Get comments for a post with different sorts\n");
  report.push("sort | time");
  report.push("--- | ---");

  for (let sort of commentSortTypes) {
    const time = await timeApiCalls(() =>
      api.getComments({
        post_id: postWithLotsOfComments,
        sort,
        max_depth: postCommentsMaxDepth,
      }),
    );
    report.push(`${sort} | ${formatMs(time)}`);
  }
});

test("Get comments for a post slim", async () => {
  report.push("\n# Get comments for a post slim\n");
  report.push("sort | time");
  report.push("--- | ---");
  const getCommentsSlim = await timeApiCalls(() =>
    api.getCommentsSlim({
      post_id: postWithLotsOfComments,
      max_depth: postCommentsMaxDepth,
    }),
  );
  report.push(`getCommentsSlim: ${formatMs(getCommentsSlim)}`);
});

test("Get all comments with different sorts", async () => {
  report.push("\n# Get all comments with different sorts\n");
  report.push("sort | time");
  report.push("--- | ---");

  for (let sort of commentSortTypes) {
    const time = await timeApiCalls(() => api.getComments({ sort }));
    report.push(`${sort} | ${formatMs(time)}`);
  }
});

test("Get comments with different types", async () => {
  report.push("\n# Get comments with different types\n");
  report.push("type | time");
  report.push("--- | ---");

  for (let type_ of listingTypes) {
    const time = await timeApiCalls(() => api.getComments({ type_ }));
    report.push(`${type_} | ${formatMs(time)}`);
  }
});

test("List person content with types", async () => {
  report.push("\n# List person content with types\n");
  report.push("type | time");
  report.push("--- | ---");

  for (let type_ of personContentTypes) {
    const time = await timeApiCalls(() =>
      api.listPersonContent({ username: samplePerson, type_ }),
    );
    report.push(`${type_} | ${formatMs(time)}`);
  }
});

test("List person saved with types", async () => {
  report.push("\n# List person saved with types\n");
  report.push("type | time");
  report.push("--- | ---");

  for (let type_ of personContentTypes) {
    const time = await timeApiCalls(() => api.listPersonSaved({ type_ }));
    report.push(`${type_} | ${formatMs(time)}`);
  }
});

test("List person liked with types", async () => {
  report.push("\n# List person liked with types\n");
  report.push("type | time");
  report.push("--- | ---");

  for (let type_ of personContentTypes) {
    const time = await timeApiCalls(() => api.listPersonLiked({ type_ }));
    report.push(`${type_} | ${formatMs(time)}`);
  }

  const likeType: LikeType[] = ["all", "liked_only", "disliked_only"];

  for (let like_type of likeType) {
    const time = await timeApiCalls(() => api.listPersonLiked({ like_type }));
    report.push(`${like_type} | ${formatMs(time)}`);
  }
});

test("List person read", async () => {
  report.push("\n# List person read\n");

  const time = await timeApiCalls(() => api.listPersonRead({}));
  report.push(`list person read: ${formatMs(time)}`);
});

test("List person hidden", async () => {
  report.push("\n# List person hidden\n");

  const time = await timeApiCalls(() => api.listPersonHidden({}));
  report.push(`list person hidden: ${formatMs(time)}`);
});

test("List registration applications", async () => {
  report.push("\n# List registration applications\n");
  report.push("type | time");
  report.push("--- | ---");

  const unreadOnly = await timeApiCalls(() =>
    api.listRegistrationApplications({ unread_only: true }),
  );
  const all = await timeApiCalls(() => api.listRegistrationApplications({}));
  report.push(`unread only | ${formatMs(unreadOnly)}`);
  report.push(`all | ${formatMs(all)}`);
});

// TODO slow
test("List reports", async () => {
  report.push("\n# List reports\n");
  report.push("type | time");
  report.push("--- | ---");

  const unresolvedOnly = await timeApiCalls(() =>
    api.listReports({ unresolved_only: true }),
  );
  const all = await timeApiCalls(() => api.listReports({}));
  report.push(`unresolved only | ${formatMs(unresolvedOnly)}`);
  report.push(`all | ${formatMs(all)}`);
});

test("Search with sort types", async () => {
  report.push("\n# Search with types\n");
  report.push("type | sort | time");
  report.push("--- | --- | ---");

  const search_term = searchTerm;

  for (let sort of postSortTypes) {
    const postTime = await timeApiCalls(() =>
      api.getPosts({ sort, search_term }),
    );
    report.push(`post | ${sort} | ${formatMs(postTime)}`);
  }

  for (let sort of commentSortTypes) {
    const commentTime = await timeApiCalls(() =>
      api.getComments({ sort, search_term }),
    );
    report.push(`comment | ${sort} | ${formatMs(commentTime)}`);
  }

  for (let sort of communitySortTypes) {
    const communityTime = await timeApiCalls(() =>
      api.listCommunities({ sort, search_term }),
    );
    report.push(`community | ${sort} | ${formatMs(communityTime)}`);
  }

  for (let sort of multiCommunitySortTypes) {
    const multiCommunityTime = await timeApiCalls(() =>
      api.listMultiCommunities({ sort, search_term }),
    );
    report.push(`multi community | ${sort} | ${formatMs(multiCommunityTime)}`);
  }

  for (let sort of personSortTypes) {
    const personTime = await timeApiCalls(() =>
      api.listPersons({ sort, search_term }),
    );
    report.push(`person | ${sort} | ${formatMs(personTime)}`);
  }
});

// TODO slow
test("Notifications with types", async () => {
  report.push("\n# Notifications with types\n");
  report.push("type | time");
  report.push("--- | ---");

  for (let type_ of notificationTypes) {
    const time = await timeApiCalls(() => api.listNotifications({ type_ }));
    report.push(`${type_} | ${formatMs(time)}`);
  }
});

test("Notifications with unread only", async () => {
  report.push("\n# Notifications with unread only\n");
  report.push("type | time");
  report.push("--- | ---");

  const unreadOnly = await timeApiCalls(() =>
    api.listNotifications({ unread_only: true }),
  );
  const all = await timeApiCalls(() => api.listNotifications({}));
  report.push(`all | ${formatMs(all)}`);
  report.push(`unread_only | ${formatMs(unreadOnly)}`);
});

test("Liking a comment / post", async () => {
  report.push("\n# Liking a comment / post\n");
  report.push("type | time");
  report.push("--- | ---");

  const commentLike = await timeApiCall(() =>
    api.likeComment({ comment_id: sampleComment }),
  );
  const postLike = await timeApiCall(() =>
    api.likePost({ post_id: postWithUrl }),
  );
  report.push(`comment | ${formatMs(commentLike.diff)}`);
  report.push(`post | ${formatMs(postLike.diff)}`);
});

// TODO Many are slow
test("Get modlog with types", async () => {
  report.push("\n# Get modlog with types\n");
  report.push("type | time");
  report.push("--- | ---");

  const modlogKinds: ModlogKindFilter[] = [
    "all",
    "admin_add",
    "admin_ban",
    "admin_allow_instance",
    "admin_block_instance",
    "admin_purge_comment",
    "admin_purge_community",
    "admin_purge_person",
    "admin_purge_post",
    "mod_add_to_community",
    "mod_ban_from_community",
    "admin_feature_post_site",
    "mod_feature_post_community",
    "mod_change_community_visibility",
    "mod_lock_post",
    "mod_remove_comment",
    "admin_remove_community",
    "mod_remove_post",
    "mod_transfer_community",
    "mod_lock_comment",
  ];

  for (let type_ of modlogKinds) {
    const time = await timeApiCalls(() => api.getModlog({ type_ }));
    report.push(`${type_} | ${formatMs(time)}`);
  }
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
