import {
  GetReplies,
  GetRepliesResponse,
  GetUnreadCount,
  GetUnreadCountResponse,
  LemmyHttp,
} from "lemmy-js-client";
import { CreatePost } from "lemmy-js-client/dist/types/CreatePost";
import { DeletePost } from "lemmy-js-client/dist/types/DeletePost";
import { EditPost } from "lemmy-js-client/dist/types/EditPost";
import { EditSite } from "lemmy-js-client/dist/types/EditSite";
import { FeaturePost } from "lemmy-js-client/dist/types/FeaturePost";
import { GetComments } from "lemmy-js-client/dist/types/GetComments";
import { GetCommentsResponse } from "lemmy-js-client/dist/types/GetCommentsResponse";
import { GetPost } from "lemmy-js-client/dist/types/GetPost";
import { GetPostResponse } from "lemmy-js-client/dist/types/GetPostResponse";
import { LockPost } from "lemmy-js-client/dist/types/LockPost";
import { Login } from "lemmy-js-client/dist/types/Login";
import { Post } from "lemmy-js-client/dist/types/Post";
import { PostResponse } from "lemmy-js-client/dist/types/PostResponse";
import { RemovePost } from "lemmy-js-client/dist/types/RemovePost";
import { ResolveObject } from "lemmy-js-client/dist/types/ResolveObject";
import { ResolveObjectResponse } from "lemmy-js-client/dist/types/ResolveObjectResponse";
import { Search } from "lemmy-js-client/dist/types/Search";
import { SearchResponse } from "lemmy-js-client/dist/types/SearchResponse";
import { Comment } from "lemmy-js-client/dist/types/Comment";
import { BanPersonResponse } from "lemmy-js-client/dist/types/BanPersonResponse";
import { BanPerson } from "lemmy-js-client/dist/types/BanPerson";
import { BanFromCommunityResponse } from "lemmy-js-client/dist/types/BanFromCommunityResponse";
import { BanFromCommunity } from "lemmy-js-client/dist/types/BanFromCommunity";
import { CommunityResponse } from "lemmy-js-client/dist/types/CommunityResponse";
import { FollowCommunity } from "lemmy-js-client/dist/types/FollowCommunity";
import { CreatePostLike } from "lemmy-js-client/dist/types/CreatePostLike";
import { CommentResponse } from "lemmy-js-client/dist/types/CommentResponse";
import { CreateComment } from "lemmy-js-client/dist/types/CreateComment";
import { EditComment } from "lemmy-js-client/dist/types/EditComment";
import { DeleteComment } from "lemmy-js-client/dist/types/DeleteComment";
import { RemoveComment } from "lemmy-js-client/dist/types/RemoveComment";
import { GetPersonMentionsResponse } from "lemmy-js-client/dist/types/GetPersonMentionsResponse";
import { GetPersonMentions } from "lemmy-js-client/dist/types/GetPersonMentions";
import { CreateCommentLike } from "lemmy-js-client/dist/types/CreateCommentLike";
import { CreateCommunity } from "lemmy-js-client/dist/types/CreateCommunity";
import { GetCommunity } from "lemmy-js-client/dist/types/GetCommunity";
import { DeleteCommunity } from "lemmy-js-client/dist/types/DeleteCommunity";
import { RemoveCommunity } from "lemmy-js-client/dist/types/RemoveCommunity";
import { PrivateMessageResponse } from "lemmy-js-client/dist/types/PrivateMessageResponse";
import { CreatePrivateMessage } from "lemmy-js-client/dist/types/CreatePrivateMessage";
import { EditPrivateMessage } from "lemmy-js-client/dist/types/EditPrivateMessage";
import { DeletePrivateMessage } from "lemmy-js-client/dist/types/DeletePrivateMessage";
import { LoginResponse } from "lemmy-js-client/dist/types/LoginResponse";
import { Register } from "lemmy-js-client/dist/types/Register";
import { SaveUserSettings } from "lemmy-js-client/dist/types/SaveUserSettings";
import { DeleteAccount } from "lemmy-js-client/dist/types/DeleteAccount";
import { GetSiteResponse } from "lemmy-js-client/dist/types/GetSiteResponse";
import { DeleteAccountResponse } from "lemmy-js-client/dist/types/DeleteAccountResponse";
import { GetSite } from "lemmy-js-client/dist/types/GetSite";
import { PrivateMessagesResponse } from "lemmy-js-client/dist/types/PrivateMessagesResponse";
import { GetPrivateMessages } from "lemmy-js-client/dist/types/GetPrivateMessages";
import { PostReportResponse } from "lemmy-js-client/dist/types/PostReportResponse";
import { CreatePostReport } from "lemmy-js-client/dist/types/CreatePostReport";
import { ListPostReportsResponse } from "lemmy-js-client/dist/types/ListPostReportsResponse";
import { ListPostReports } from "lemmy-js-client/dist/types/ListPostReports";
import { CommentReportResponse } from "lemmy-js-client/dist/types/CommentReportResponse";
import { CreateCommentReport } from "lemmy-js-client/dist/types/CreateCommentReport";
import { ListCommentReportsResponse } from "lemmy-js-client/dist/types/ListCommentReportsResponse";
import { ListCommentReports } from "lemmy-js-client/dist/types/ListCommentReports";
import { GetPostsResponse } from "lemmy-js-client/dist/types/GetPostsResponse";
import { GetPosts } from "lemmy-js-client/dist/types/GetPosts";
import { GetPersonDetailsResponse } from "lemmy-js-client/dist/types/GetPersonDetailsResponse";
import { GetPersonDetails } from "lemmy-js-client/dist/types/GetPersonDetails";
import { ListingType } from "lemmy-js-client/dist/types/ListingType";

export interface API {
  client: LemmyHttp;
  auth: string;
}

export let alpha: API = {
  client: new LemmyHttp("http://127.0.0.1:8541"),
  auth: "",
};

export let beta: API = {
  client: new LemmyHttp("http://127.0.0.1:8551"),
  auth: "",
};

export let gamma: API = {
  client: new LemmyHttp("http://127.0.0.1:8561"),
  auth: "",
};

export let delta: API = {
  client: new LemmyHttp("http://127.0.0.1:8571"),
  auth: "",
};

export let epsilon: API = {
  client: new LemmyHttp("http://127.0.0.1:8581"),
  auth: "",
};

const password = "lemmylemmy";

export async function setupLogins() {
  let formAlpha: Login = {
    username_or_email: "lemmy_alpha",
    password,
  };
  let resAlpha = alpha.client.login(formAlpha);

  let formBeta: Login = {
    username_or_email: "lemmy_beta",
    password,
  };
  let resBeta = beta.client.login(formBeta);

  let formGamma: Login = {
    username_or_email: "lemmy_gamma",
    password,
  };
  let resGamma = gamma.client.login(formGamma);

  let formDelta: Login = {
    username_or_email: "lemmy_delta",
    password,
  };
  let resDelta = delta.client.login(formDelta);

  let formEpsilon: Login = {
    username_or_email: "lemmy_epsilon",
    password,
  };
  let resEpsilon = epsilon.client.login(formEpsilon);

  let res = await Promise.all([
    resAlpha,
    resBeta,
    resGamma,
    resDelta,
    resEpsilon,
  ]);

  alpha.auth = res[0].jwt ?? "";
  beta.auth = res[1].jwt ?? "";
  gamma.auth = res[2].jwt ?? "";
  delta.auth = res[3].jwt ?? "";
  epsilon.auth = res[4].jwt ?? "";

  // Registration applications are now enabled by default, need to disable them
  let editSiteForm: EditSite = {
    registration_mode: "Open",
    rate_limit_message: 999,
    rate_limit_post: 999,
    rate_limit_register: 999,
    rate_limit_image: 999,
    rate_limit_comment: 999,
    rate_limit_search: 999,
    auth: "",
  };

  // Set the blocks and auths for each
  editSiteForm.auth = alpha.auth;
  editSiteForm.allowed_instances = [
    "lemmy-beta",
    "lemmy-gamma",
    "lemmy-delta",
    "lemmy-epsilon",
  ];
  await alpha.client.editSite(editSiteForm);

  editSiteForm.auth = beta.auth;
  editSiteForm.allowed_instances = [
    "lemmy-alpha",
    "lemmy-gamma",
    "lemmy-delta",
    "lemmy-epsilon",
  ];
  await beta.client.editSite(editSiteForm);

  editSiteForm.auth = gamma.auth;
  editSiteForm.allowed_instances = [
    "lemmy-alpha",
    "lemmy-beta",
    "lemmy-delta",
    "lemmy-epsilon",
  ];
  await gamma.client.editSite(editSiteForm);

  editSiteForm.allowed_instances = ["lemmy-beta"];
  editSiteForm.auth = delta.auth;
  await delta.client.editSite(editSiteForm);

  editSiteForm.auth = epsilon.auth;
  editSiteForm.allowed_instances = [];
  editSiteForm.blocked_instances = ["lemmy-alpha"];
  await epsilon.client.editSite(editSiteForm);

  // Create the main alpha/beta communities
  // Ignore thrown errors of duplicates
  try {
    await createCommunity(alpha, "main");
    await createCommunity(beta, "main");
    // wait for > INSTANCES_RECHECK_DELAY to ensure federation is initialized
    // otherwise the first few federated events may be missed
    // (because last_successful_id is set to current id when federation to an instance is first started)
    // only needed the first time so do in this try
    await delay(10_000);
  } catch (_) {
    console.log("Communities already exist");
  }
}

export async function createPost(
  api: API,
  community_id: number,
): Promise<PostResponse> {
  let name = randomString(5);
  let body = randomString(10);
  // switch from google.com to example.com for consistent title (embed_title and embed_description)
  // google switches description when a google doodle appears
  let url = "https://example.com/";
  let form: CreatePost = {
    name,
    url,
    body,
    auth: api.auth,
    community_id,
  };
  return api.client.createPost(form);
}

export async function editPost(api: API, post: Post): Promise<PostResponse> {
  let name = "A jest test federated post, updated";
  let form: EditPost = {
    name,
    post_id: post.id,
    auth: api.auth,
  };
  return api.client.editPost(form);
}

export async function deletePost(
  api: API,
  deleted: boolean,
  post: Post,
): Promise<PostResponse> {
  let form: DeletePost = {
    post_id: post.id,
    deleted: deleted,
    auth: api.auth,
  };
  return api.client.deletePost(form);
}

export async function removePost(
  api: API,
  removed: boolean,
  post: Post,
): Promise<PostResponse> {
  let form: RemovePost = {
    post_id: post.id,
    removed,
    auth: api.auth,
  };
  return api.client.removePost(form);
}

export async function featurePost(
  api: API,
  featured: boolean,
  post: Post,
): Promise<PostResponse> {
  let form: FeaturePost = {
    post_id: post.id,
    featured,
    feature_type: "Community",
    auth: api.auth,
  };
  return api.client.featurePost(form);
}

export async function lockPost(
  api: API,
  locked: boolean,
  post: Post,
): Promise<PostResponse> {
  let form: LockPost = {
    post_id: post.id,
    locked,
    auth: api.auth,
  };
  return api.client.lockPost(form);
}

export async function resolvePost(
  api: API,
  post: Post,
  localOnly = true,
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q: post.ap_id,
    auth: localOnly ? null : api.auth,
  };
  return api.client.resolveObject(form);
}

export async function searchPostLocal(
  api: API,
  post: Post,
): Promise<SearchResponse> {
  let form: Search = {
    q: post.name,
    type_: "Posts",
    sort: "TopAll",
    auth: api.auth,
  };
  return api.client.search(form);
}

export async function getPost(
  api: API,
  post_id: number,
): Promise<GetPostResponse> {
  let form: GetPost = {
    id: post_id,
    auth: api.auth,
  };
  return api.client.getPost(form);
}

export async function getComments(
  api: API,
  post_id?: number,
  listingType: ListingType = "All",
): Promise<GetCommentsResponse> {
  let form: GetComments = {
    post_id: post_id,
    type_: listingType,
    sort: "New",
    auth: api.auth,
  };
  return api.client.getComments(form);
}

export async function getUnreadCount(
  api: API,
): Promise<GetUnreadCountResponse> {
  let form: GetUnreadCount = {
    auth: api.auth,
  };
  return api.client.getUnreadCount(form);
}

export async function getReplies(api: API): Promise<GetRepliesResponse> {
  let form: GetReplies = {
    sort: "New",
    unread_only: false,
    auth: api.auth,
  };
  return api.client.getReplies(form);
}

export async function resolveComment(
  api: API,
  comment: Comment,
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q: comment.ap_id,
    auth: api.auth,
  };
  return api.client.resolveObject(form);
}

export async function resolveBetaCommunity(
  api: API,
): Promise<ResolveObjectResponse> {
  // Use short-hand search url
  let form: ResolveObject = {
    q: "!main@lemmy-beta:8551",
    auth: api.auth,
  };
  return api.client.resolveObject(form);
}

export async function resolveCommunity(
  api: API,
  q: string,
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q,
    auth: api.auth,
  };
  return api.client.resolveObject(form);
}

export async function resolvePerson(
  api: API,
  apShortname: string,
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q: apShortname,
    auth: api.auth,
  };
  return api.client.resolveObject(form);
}

export async function banPersonFromSite(
  api: API,
  person_id: number,
  ban: boolean,
  remove_data: boolean,
): Promise<BanPersonResponse> {
  // Make sure lemmy-beta/c/main is cached on lemmy_alpha
  let form: BanPerson = {
    person_id,
    ban,
    remove_data: remove_data,
    auth: api.auth,
  };
  return api.client.banPerson(form);
}

export async function banPersonFromCommunity(
  api: API,
  person_id: number,
  community_id: number,
  remove_data: boolean,
  ban: boolean,
): Promise<BanFromCommunityResponse> {
  let form: BanFromCommunity = {
    person_id,
    community_id,
    remove_data: remove_data,
    ban,
    auth: api.auth,
  };
  return api.client.banFromCommunity(form);
}

export async function followCommunity(
  api: API,
  follow: boolean,
  community_id: number,
): Promise<CommunityResponse> {
  let form: FollowCommunity = {
    community_id,
    follow,
    auth: api.auth,
  };
  const res = await api.client.followCommunity(form);
  await waitUntil(
    () => resolveCommunity(api, res.community_view.community.actor_id),
    g => g.community?.subscribed === (follow ? "Subscribed" : "NotSubscribed"),
  );
  // wait FOLLOW_ADDITIONS_RECHECK_DELAY (there's no API to wait for this currently)
  await delay(2000);
  return res;
}

export async function likePost(
  api: API,
  score: number,
  post: Post,
): Promise<PostResponse> {
  let form: CreatePostLike = {
    post_id: post.id,
    score: score,
    auth: api.auth,
  };

  return api.client.likePost(form);
}

export async function createComment(
  api: API,
  post_id: number,
  parent_id?: number,
  content = "a jest test comment",
): Promise<CommentResponse> {
  let form: CreateComment = {
    content,
    post_id,
    parent_id,
    auth: api.auth,
  };
  return api.client.createComment(form);
}

export async function editComment(
  api: API,
  comment_id: number,
  content = "A jest test federated comment update",
): Promise<CommentResponse> {
  let form: EditComment = {
    content,
    comment_id,
    auth: api.auth,
  };
  return api.client.editComment(form);
}

export async function deleteComment(
  api: API,
  deleted: boolean,
  comment_id: number,
): Promise<CommentResponse> {
  let form: DeleteComment = {
    comment_id,
    deleted,
    auth: api.auth,
  };
  return api.client.deleteComment(form);
}

export async function removeComment(
  api: API,
  removed: boolean,
  comment_id: number,
): Promise<CommentResponse> {
  let form: RemoveComment = {
    comment_id,
    removed,
    auth: api.auth,
  };
  return api.client.removeComment(form);
}

export async function getMentions(
  api: API,
): Promise<GetPersonMentionsResponse> {
  let form: GetPersonMentions = {
    sort: "New",
    unread_only: false,
    auth: api.auth,
  };
  return api.client.getPersonMentions(form);
}

export async function likeComment(
  api: API,
  score: number,
  comment: Comment,
): Promise<CommentResponse> {
  let form: CreateCommentLike = {
    comment_id: comment.id,
    score,
    auth: api.auth,
  };
  return api.client.likeComment(form);
}

export async function createCommunity(
  api: API,
  name_: string = randomString(5),
): Promise<CommunityResponse> {
  let description = "a sample description";
  let form: CreateCommunity = {
    name: name_,
    title: name_,
    description,
    auth: api.auth,
  };
  return api.client.createCommunity(form);
}

export async function getCommunity(
  api: API,
  id: number,
): Promise<CommunityResponse> {
  let form: GetCommunity = {
    id,
    auth: api.auth,
  };
  return api.client.getCommunity(form);
}

export async function getCommunityByName(
  api: API,
  name: string,
): Promise<CommunityResponse> {
  let form: GetCommunity = {
    name,
    auth: api.auth,
  };
  return api.client.getCommunity(form);
}

export async function deleteCommunity(
  api: API,
  deleted: boolean,
  community_id: number,
): Promise<CommunityResponse> {
  let form: DeleteCommunity = {
    community_id,
    deleted,
    auth: api.auth,
  };
  return api.client.deleteCommunity(form);
}

export async function removeCommunity(
  api: API,
  removed: boolean,
  community_id: number,
): Promise<CommunityResponse> {
  let form: RemoveCommunity = {
    community_id,
    removed,
    auth: api.auth,
  };
  return api.client.removeCommunity(form);
}

export async function createPrivateMessage(
  api: API,
  recipient_id: number,
): Promise<PrivateMessageResponse> {
  let content = "A jest test federated private message";
  let form: CreatePrivateMessage = {
    content,
    recipient_id,
    auth: api.auth,
  };
  return api.client.createPrivateMessage(form);
}

export async function editPrivateMessage(
  api: API,
  private_message_id: number,
): Promise<PrivateMessageResponse> {
  let updatedContent = "A jest test federated private message edited";
  let form: EditPrivateMessage = {
    content: updatedContent,
    private_message_id,
    auth: api.auth,
  };
  return api.client.editPrivateMessage(form);
}

export async function deletePrivateMessage(
  api: API,
  deleted: boolean,
  private_message_id: number,
): Promise<PrivateMessageResponse> {
  let form: DeletePrivateMessage = {
    deleted,
    private_message_id,
    auth: api.auth,
  };
  return api.client.deletePrivateMessage(form);
}

export async function registerUser(
  api: API,
  username: string = randomString(5),
): Promise<LoginResponse> {
  let form: Register = {
    username,
    password,
    password_verify: password,
    show_nsfw: true,
  };
  return api.client.register(form);
}

export async function saveUserSettingsBio(api: API): Promise<LoginResponse> {
  let form: SaveUserSettings = {
    show_nsfw: true,
    blur_nsfw: false,
    auto_expand: true,
    theme: "darkly",
    default_sort_type: "Active",
    default_listing_type: "All",
    interface_language: "en",
    show_avatars: true,
    send_notifications_to_email: false,
    bio: "a changed bio",
    auth: api.auth,
  };
  return saveUserSettings(api, form);
}

export async function saveUserSettingsFederated(
  api: API,
): Promise<LoginResponse> {
  let avatar = "https://image.flaticon.com/icons/png/512/35/35896.png";
  let banner = "https://image.flaticon.com/icons/png/512/36/35896.png";
  let bio = "a changed bio";
  let form: SaveUserSettings = {
    show_nsfw: false,
    blur_nsfw: true,
    auto_expand: false,
    default_sort_type: "Hot",
    default_listing_type: "All",
    interface_language: "",
    avatar,
    banner,
    display_name: "user321",
    show_avatars: false,
    send_notifications_to_email: false,
    bio,
    auth: api.auth,
  };
  return await saveUserSettings(alpha, form);
}

export async function saveUserSettings(
  api: API,
  form: SaveUserSettings,
): Promise<LoginResponse> {
  return api.client.saveUserSettings(form);
}
export async function getPersonDetails(
  api: API,
  person_id: number,
): Promise<GetPersonDetailsResponse> {
  let form: GetPersonDetails = {
    auth: api.auth,
    person_id: person_id,
  };
  return api.client.getPersonDetails(form);
}

export async function deleteUser(api: API): Promise<DeleteAccountResponse> {
  let form: DeleteAccount = {
    auth: api.auth,
    delete_content: true,
    password,
  };
  return api.client.deleteAccount(form);
}

export async function getSite(api: API): Promise<GetSiteResponse> {
  let form: GetSite = {
    auth: api.auth,
  };
  return api.client.getSite(form);
}

export async function listPrivateMessages(
  api: API,
): Promise<PrivateMessagesResponse> {
  let form: GetPrivateMessages = {
    auth: api.auth,
    unread_only: false,
  };
  return api.client.getPrivateMessages(form);
}

export async function unfollowRemotes(api: API): Promise<GetSiteResponse> {
  // Unfollow all remote communities
  let site = await getSite(api);
  let remoteFollowed =
    site.my_user?.follows.filter(c => c.community.local == false) ?? [];
  await Promise.all(
    remoteFollowed.map(cu => followCommunity(api, false, cu.community.id)),
  );
  let siteRes = await getSite(api);
  return siteRes;
}

export async function followBeta(api: API): Promise<CommunityResponse> {
  let betaCommunity = (await resolveBetaCommunity(api)).community;
  if (betaCommunity) {
    let follow = await followCommunity(api, true, betaCommunity.community.id);
    return follow;
  } else {
    return Promise.reject("no community worked");
  }
}

export async function reportPost(
  api: API,
  post_id: number,
  reason: string,
): Promise<PostReportResponse> {
  let form: CreatePostReport = {
    post_id,
    reason,
    auth: api.auth,
  };
  return api.client.createPostReport(form);
}

export async function listPostReports(
  api: API,
): Promise<ListPostReportsResponse> {
  let form: ListPostReports = {
    auth: api.auth,
  };
  return api.client.listPostReports(form);
}

export async function reportComment(
  api: API,
  comment_id: number,
  reason: string,
): Promise<CommentReportResponse> {
  let form: CreateCommentReport = {
    comment_id,
    reason,
    auth: api.auth,
  };
  return api.client.createCommentReport(form);
}

export async function listCommentReports(
  api: API,
): Promise<ListCommentReportsResponse> {
  let form: ListCommentReports = {
    auth: api.auth,
  };
  return api.client.listCommentReports(form);
}

export function getPosts(
  api: API,
  listingType?: ListingType,
): Promise<GetPostsResponse> {
  let form: GetPosts = {
    auth: api.auth,
    type_: listingType,
  };
  return api.client.getPosts(form);
}

export function delay(millis = 500) {
  return new Promise(resolve => setTimeout(resolve, millis));
}

export function longDelay() {
  return delay(10000);
}

export function wrapper(form: any): string {
  return JSON.stringify(form);
}

export function randomString(length: number): string {
  var result = "";
  var characters =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
  var charactersLength = characters.length;
  for (var i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}

export async function unfollows() {
  await Promise.all([
    unfollowRemotes(alpha),
    unfollowRemotes(gamma),
    unfollowRemotes(delta),
    unfollowRemotes(epsilon),
  ]);
}

export function getCommentParentId(comment: Comment): number | undefined {
  let split = comment.path.split(".");
  // remove the 0
  split.shift();

  if (split.length > 1) {
    return Number(split[split.length - 2]);
  } else {
    return undefined;
  }
}

export async function waitUntil<T>(
  fetcher: () => Promise<T>,
  checker: (t: T) => boolean,
  retries = 10,
  delaySeconds = [0.2, 0.5, 1, 2, 3],
) {
  let retry = 0;
  let result;
  while (retry++ < retries) {
    result = await fetcher();
    if (checker(result)) return result;
    await delay(
      delaySeconds[Math.min(retry - 1, delaySeconds.length - 1)] * 1000,
    );
  }
  console.error("result", result);
  throw Error(
    `Failed "${fetcher}": "${checker}" did not return true after ${retries} retries (delayed ${delaySeconds}s each)`,
  );
}
