import {
  BlockInstance,
  BlockInstanceResponse,
  GetReplies,
  GetRepliesResponse,
  GetUnreadCountResponse,
  InstanceId,
  LemmyHttp,
  PostView,
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

export let alphaUrl = "http://127.0.0.1:8541";
export let betaUrl = "http://127.0.0.1:8551";
export let gammaUrl = "http://127.0.0.1:8561";
export let deltaUrl = "http://127.0.0.1:8571";
export let epsilonUrl = "http://127.0.0.1:8581";

export let alpha = new LemmyHttp(alphaUrl);
export let beta = new LemmyHttp(betaUrl);
export let gamma = new LemmyHttp(gammaUrl);
export let delta = new LemmyHttp(deltaUrl);
export let epsilon = new LemmyHttp(epsilonUrl);

const password = "lemmylemmy";

export async function setupLogins() {
  let formAlpha: Login = {
    username_or_email: "lemmy_alpha",
    password,
  };
  let resAlpha = alpha.login(formAlpha);

  let formBeta: Login = {
    username_or_email: "lemmy_beta",
    password,
  };
  let resBeta = beta.login(formBeta);

  let formGamma: Login = {
    username_or_email: "lemmy_gamma",
    password,
  };
  let resGamma = gamma.login(formGamma);

  let formDelta: Login = {
    username_or_email: "lemmy_delta",
    password,
  };
  let resDelta = delta.login(formDelta);

  let formEpsilon: Login = {
    username_or_email: "lemmy_epsilon",
    password,
  };
  let resEpsilon = epsilon.login(formEpsilon);

  let res = await Promise.all([
    resAlpha,
    resBeta,
    resGamma,
    resDelta,
    resEpsilon,
  ]);
  alpha.setHeaders({ Authorization: `Bearer ${res[0].jwt ?? ""}` });
  beta.setHeaders({ Authorization: `Bearer ${res[1].jwt ?? ""}` });
  gamma.setHeaders({ Authorization: `Bearer ${res[2].jwt ?? ""}` });
  delta.setHeaders({ Authorization: `Bearer ${res[3].jwt ?? ""}` });
  epsilon.setHeaders({ Authorization: `Bearer ${res[4].jwt ?? ""}` });

  // Registration applications are now enabled by default, need to disable them
  let editSiteForm: EditSite = {
    registration_mode: "Open",
    rate_limit_message: 999,
    rate_limit_post: 999,
    rate_limit_register: 999,
    rate_limit_image: 999,
    rate_limit_comment: 999,
    rate_limit_search: 999,
  };

  // Set the blocks and auths for each
  editSiteForm.allowed_instances = [
    "lemmy-beta",
    "lemmy-gamma",
    "lemmy-delta",
    "lemmy-epsilon",
  ];
  await alpha.editSite(editSiteForm);

  editSiteForm.allowed_instances = [
    "lemmy-alpha",
    "lemmy-gamma",
    "lemmy-delta",
    "lemmy-epsilon",
  ];
  await beta.editSite(editSiteForm);

  editSiteForm.allowed_instances = [
    "lemmy-alpha",
    "lemmy-beta",
    "lemmy-delta",
    "lemmy-epsilon",
  ];
  await gamma.editSite(editSiteForm);

  editSiteForm.allowed_instances = ["lemmy-beta"];
  await delta.editSite(editSiteForm);

  editSiteForm.allowed_instances = [];
  editSiteForm.blocked_instances = ["lemmy-alpha"];
  await epsilon.editSite(editSiteForm);

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
  api: LemmyHttp,
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
    community_id,
  };
  return api.createPost(form);
}

export async function editPost(
  api: LemmyHttp,
  post: Post,
): Promise<PostResponse> {
  let name = "A jest test federated post, updated";
  let form: EditPost = {
    name,
    post_id: post.id,
  };
  return api.editPost(form);
}

export async function deletePost(
  api: LemmyHttp,
  deleted: boolean,
  post: Post,
): Promise<PostResponse> {
  let form: DeletePost = {
    post_id: post.id,
    deleted: deleted,
  };
  return api.deletePost(form);
}

export async function removePost(
  api: LemmyHttp,
  removed: boolean,
  post: Post,
): Promise<PostResponse> {
  let form: RemovePost = {
    post_id: post.id,
    removed,
  };
  return api.removePost(form);
}

export async function featurePost(
  api: LemmyHttp,
  featured: boolean,
  post: Post,
): Promise<PostResponse> {
  let form: FeaturePost = {
    post_id: post.id,
    featured,
    feature_type: "Community",
  };
  return api.featurePost(form);
}

export async function lockPost(
  api: LemmyHttp,
  locked: boolean,
  post: Post,
): Promise<PostResponse> {
  let form: LockPost = {
    post_id: post.id,
    locked,
  };
  return api.lockPost(form);
}

export async function resolvePost(
  api: LemmyHttp,
  post: Post,
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q: post.ap_id,
  };
  return api.resolveObject(form);
}

export async function searchPostLocal(
  api: LemmyHttp,
  post: Post,
): Promise<SearchResponse> {
  let form: Search = {
    q: post.name,
    type_: "Posts",
    sort: "TopAll",
  };
  return api.search(form);
}

/// wait for a post to appear locally without pulling it
export async function waitForPost(
  api: LemmyHttp,
  post: Post,
  checker: (t: PostView | undefined) => boolean = p => !!p,
) {
  return waitUntil<PostView>(
    () => searchPostLocal(api, post).then(p => p.posts[0]),
    checker,
  );
}

export async function getPost(
  api: LemmyHttp,
  post_id: number,
): Promise<GetPostResponse> {
  let form: GetPost = {
    id: post_id,
  };
  return api.getPost(form);
}

export async function getComments(
  api: LemmyHttp,
  post_id?: number,
  listingType: ListingType = "All",
): Promise<GetCommentsResponse> {
  let form: GetComments = {
    post_id: post_id,
    type_: listingType,
    sort: "New",
  };
  return api.getComments(form);
}

export async function getUnreadCount(
  api: LemmyHttp,
): Promise<GetUnreadCountResponse> {
  return api.getUnreadCount();
}

export async function getReplies(api: LemmyHttp): Promise<GetRepliesResponse> {
  let form: GetReplies = {
    sort: "New",
    unread_only: false,
  };
  return api.getReplies(form);
}

export async function resolveComment(
  api: LemmyHttp,
  comment: Comment,
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q: comment.ap_id,
  };
  return api.resolveObject(form);
}

export async function resolveBetaCommunity(
  api: LemmyHttp,
): Promise<ResolveObjectResponse> {
  // Use short-hand search url
  let form: ResolveObject = {
    q: "!main@lemmy-beta:8551",
  };
  return api.resolveObject(form);
}

export async function resolveCommunity(
  api: LemmyHttp,
  q: string,
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q,
  };
  return api.resolveObject(form);
}

export async function resolvePerson(
  api: LemmyHttp,
  apShortname: string,
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q: apShortname,
  };
  return api.resolveObject(form);
}

export async function banPersonFromSite(
  api: LemmyHttp,
  person_id: number,
  ban: boolean,
  remove_data: boolean,
): Promise<BanPersonResponse> {
  // Make sure lemmy-beta/c/main is cached on lemmy_alpha
  let form: BanPerson = {
    person_id,
    ban,
    remove_data: remove_data,
  };
  return api.banPerson(form);
}

export async function banPersonFromCommunity(
  api: LemmyHttp,
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
  };
  return api.banFromCommunity(form);
}

export async function followCommunity(
  api: LemmyHttp,
  follow: boolean,
  community_id: number,
): Promise<CommunityResponse> {
  let form: FollowCommunity = {
    community_id,
    follow,
  };
  const res = await api.followCommunity(form);
  await waitUntil(
    () => resolveCommunity(api, res.community_view.community.actor_id),
    g => g.community?.subscribed === (follow ? "Subscribed" : "NotSubscribed"),
  );
  // wait FOLLOW_ADDITIONS_RECHECK_DELAY (there's no API to wait for this currently)
  await delay(2000);
  return res;
}

export async function likePost(
  api: LemmyHttp,
  score: number,
  post: Post,
): Promise<PostResponse> {
  let form: CreatePostLike = {
    post_id: post.id,
    score: score,
  };

  return api.likePost(form);
}

export async function createComment(
  api: LemmyHttp,
  post_id: number,
  parent_id?: number,
  content = "a jest test comment",
): Promise<CommentResponse> {
  let form: CreateComment = {
    content,
    post_id,
    parent_id,
  };
  return api.createComment(form);
}

export async function editComment(
  api: LemmyHttp,
  comment_id: number,
  content = "A jest test federated comment update",
): Promise<CommentResponse> {
  let form: EditComment = {
    content,
    comment_id,
  };
  return api.editComment(form);
}

export async function deleteComment(
  api: LemmyHttp,
  deleted: boolean,
  comment_id: number,
): Promise<CommentResponse> {
  let form: DeleteComment = {
    comment_id,
    deleted,
  };
  return api.deleteComment(form);
}

export async function removeComment(
  api: LemmyHttp,
  removed: boolean,
  comment_id: number,
): Promise<CommentResponse> {
  let form: RemoveComment = {
    comment_id,
    removed,
  };
  return api.removeComment(form);
}

export async function getMentions(
  api: LemmyHttp,
): Promise<GetPersonMentionsResponse> {
  let form: GetPersonMentions = {
    sort: "New",
    unread_only: false,
  };
  return api.getPersonMentions(form);
}

export async function likeComment(
  api: LemmyHttp,
  score: number,
  comment: Comment,
): Promise<CommentResponse> {
  let form: CreateCommentLike = {
    comment_id: comment.id,
    score,
  };
  return api.likeComment(form);
}

export async function createCommunity(
  api: LemmyHttp,
  name_: string = randomString(5),
): Promise<CommunityResponse> {
  let description = "a sample description";
  let form: CreateCommunity = {
    name: name_,
    title: name_,
    description,
  };
  return api.createCommunity(form);
}

export async function getCommunity(
  api: LemmyHttp,
  id: number,
): Promise<CommunityResponse> {
  let form: GetCommunity = {
    id,
  };
  return api.getCommunity(form);
}

export async function getCommunityByName(
  api: LemmyHttp,
  name: string,
): Promise<CommunityResponse> {
  let form: GetCommunity = {
    name,
  };
  return api.getCommunity(form);
}

export async function deleteCommunity(
  api: LemmyHttp,
  deleted: boolean,
  community_id: number,
): Promise<CommunityResponse> {
  let form: DeleteCommunity = {
    community_id,
    deleted,
  };
  return api.deleteCommunity(form);
}

export async function removeCommunity(
  api: LemmyHttp,
  removed: boolean,
  community_id: number,
): Promise<CommunityResponse> {
  let form: RemoveCommunity = {
    community_id,
    removed,
  };
  return api.removeCommunity(form);
}

export async function createPrivateMessage(
  api: LemmyHttp,
  recipient_id: number,
): Promise<PrivateMessageResponse> {
  let content = "A jest test federated private message";
  let form: CreatePrivateMessage = {
    content,
    recipient_id,
  };
  return api.createPrivateMessage(form);
}

export async function editPrivateMessage(
  api: LemmyHttp,
  private_message_id: number,
): Promise<PrivateMessageResponse> {
  let updatedContent = "A jest test federated private message edited";
  let form: EditPrivateMessage = {
    content: updatedContent,
    private_message_id,
  };
  return api.editPrivateMessage(form);
}

export async function deletePrivateMessage(
  api: LemmyHttp,
  deleted: boolean,
  private_message_id: number,
): Promise<PrivateMessageResponse> {
  let form: DeletePrivateMessage = {
    deleted,
    private_message_id,
  };
  return api.deletePrivateMessage(form);
}

export async function registerUser(
  api: LemmyHttp,
  username: string = randomString(5),
): Promise<LoginResponse> {
  let form: Register = {
    username,
    password,
    password_verify: password,
    show_nsfw: true,
  };
  return api.register(form);
}

export async function loginUser(
  api: LemmyHttp,
  username: string,
): Promise<LoginResponse> {
  let form: Login = {
    username_or_email: username,
    password: password,
  };
  return api.login(form);
}

export async function saveUserSettingsBio(
  api: LemmyHttp,
): Promise<LoginResponse> {
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
  };
  return saveUserSettings(api, form);
}

export async function saveUserSettingsFederated(
  api: LemmyHttp,
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
  };
  return await saveUserSettings(api, form);
}

export async function saveUserSettings(
  api: LemmyHttp,
  form: SaveUserSettings,
): Promise<LoginResponse> {
  return api.saveUserSettings(form);
}
export async function getPersonDetails(
  api: LemmyHttp,
  person_id: number,
): Promise<GetPersonDetailsResponse> {
  let form: GetPersonDetails = {
    person_id: person_id,
  };
  return api.getPersonDetails(form);
}

export async function deleteUser(
  api: LemmyHttp,
): Promise<DeleteAccountResponse> {
  let form: DeleteAccount = {
    delete_content: true,
    password,
  };
  return api.deleteAccount(form);
}

export async function getSite(api: LemmyHttp): Promise<GetSiteResponse> {
  return api.getSite();
}

export async function listPrivateMessages(
  api: LemmyHttp,
): Promise<PrivateMessagesResponse> {
  let form: GetPrivateMessages = {
    unread_only: false,
  };
  return api.getPrivateMessages(form);
}

export async function unfollowRemotes(
  api: LemmyHttp,
): Promise<GetSiteResponse> {
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

export async function followBeta(api: LemmyHttp): Promise<CommunityResponse> {
  let betaCommunity = (await resolveBetaCommunity(api)).community;
  if (betaCommunity) {
    let follow = await followCommunity(api, true, betaCommunity.community.id);
    return follow;
  } else {
    return Promise.reject("no community worked");
  }
}

export async function reportPost(
  api: LemmyHttp,
  post_id: number,
  reason: string,
): Promise<PostReportResponse> {
  let form: CreatePostReport = {
    post_id,
    reason,
  };
  return api.createPostReport(form);
}

export async function listPostReports(
  api: LemmyHttp,
): Promise<ListPostReportsResponse> {
  let form: ListPostReports = {};
  return api.listPostReports(form);
}

export async function reportComment(
  api: LemmyHttp,
  comment_id: number,
  reason: string,
): Promise<CommentReportResponse> {
  let form: CreateCommentReport = {
    comment_id,
    reason,
  };
  return api.createCommentReport(form);
}

export async function listCommentReports(
  api: LemmyHttp,
): Promise<ListCommentReportsResponse> {
  let form: ListCommentReports = {};
  return api.listCommentReports(form);
}

export function getPosts(
  api: LemmyHttp,
  listingType?: ListingType,
): Promise<GetPostsResponse> {
  let form: GetPosts = {
    type_: listingType,
  };
  return api.getPosts(form);
}

export function blockInstance(
  api: LemmyHttp,
  instance_id: InstanceId,
  block: boolean,
): Promise<BlockInstanceResponse> {
  let form: BlockInstance = {
    instance_id,
    block,
  };
  return api.blockInstance(form);
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
