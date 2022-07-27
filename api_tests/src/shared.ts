import {None, Some, Option} from '@sniptt/monads';
import {
  Login,
  LoginResponse,
  CreatePost,
  EditPost,
  CreateComment,
  DeletePost,
  RemovePost,
  StickyPost,
  LockPost,
  PostResponse,
  SearchResponse,
  FollowCommunity,
  CommunityResponse,
  GetPostResponse,
  Register,
  Comment,
  EditComment,
  DeleteComment,
  RemoveComment,
  Search,
  CommentResponse,
  GetCommunity,
  CreateCommunity,
  DeleteCommunity,
  RemoveCommunity,
  GetPersonMentions,
  CreateCommentLike,
  CreatePostLike,
  EditPrivateMessage,
  DeletePrivateMessage,
  GetPrivateMessages,
  GetSite,
  GetPost,
  PrivateMessageResponse,
  PrivateMessagesResponse,
  GetPersonMentionsResponse,
  SaveUserSettings,
  SortType,
  ListingType,
  GetSiteResponse,
  SearchType,
  LemmyHttp,
  BanPersonResponse,
  BanPerson,
  BanFromCommunity,
  BanFromCommunityResponse,
  Post,
  CreatePrivateMessage,
  ResolveObjectResponse,
  ResolveObject,
  CreatePostReport,
  ListPostReports,
  PostReportResponse,
  ListPostReportsResponse,
  CreateCommentReport,
  CommentReportResponse,
  ListCommentReports,
  ListCommentReportsResponse,
  DeleteAccount,
  DeleteAccountResponse,
  EditSite,
  CommentSortType,
  GetComments,
  GetCommentsResponse
} from 'lemmy-js-client';

export interface API {
  client: LemmyHttp;
  auth: Option<string>;
}

export let alpha: API = {
  client: new LemmyHttp('http://127.0.0.1:8541'),
  auth: None,
};

export let beta: API = {
  client: new LemmyHttp('http://127.0.0.1:8551'),
  auth: None,
};

export let gamma: API = {
  client: new LemmyHttp('http://127.0.0.1:8561'),
  auth: None,
};

export let delta: API = {
  client: new LemmyHttp('http://127.0.0.1:8571'),
  auth: None,
};

export let epsilon: API = {
  client: new LemmyHttp('http://127.0.0.1:8581'),
  auth: None,
};

const password = 'lemmylemmy'

export async function setupLogins() {
  let formAlpha = new Login({
    username_or_email: 'lemmy_alpha',
    password,
  });
  let resAlpha = alpha.client.login(formAlpha);

  let formBeta = new Login({
    username_or_email: 'lemmy_beta',
    password,
  });
  let resBeta = beta.client.login(formBeta);

  let formGamma = new Login({
    username_or_email: 'lemmy_gamma',
    password,
  });
  let resGamma = gamma.client.login(formGamma);

  let formDelta = new Login({
    username_or_email: 'lemmy_delta',
    password,
  });
  let resDelta = delta.client.login(formDelta);

  let formEpsilon = new Login({
    username_or_email: 'lemmy_epsilon',
    password,
  });
  let resEpsilon = epsilon.client.login(formEpsilon);

  let res = await Promise.all([
    resAlpha,
    resBeta,
    resGamma,
    resDelta,
    resEpsilon,
  ]);

  alpha.auth = res[0].jwt;
  beta.auth = res[1].jwt;
  gamma.auth = res[2].jwt;
  delta.auth = res[3].jwt;
  epsilon.auth = res[4].jwt;

  // Registration applications are now enabled by default, need to disable them
  let editSiteForm = new EditSite({
    name: None,
    sidebar: None,
    description: None,
    icon: None,
    banner: None,
    enable_downvotes: None,
    open_registration: None,
    enable_nsfw: None,
    community_creation_admin_only: None,
    require_email_verification: None,
    require_application: Some(false),
    application_question: None,
    private_instance: None,
    default_theme: None,
    legal_information: None,
    default_post_listing_type: None,
    auth: "",
  });
  editSiteForm.auth = alpha.auth.unwrap();
  await alpha.client.editSite(editSiteForm);
  editSiteForm.auth = beta.auth.unwrap();
  await beta.client.editSite(editSiteForm);
  editSiteForm.auth = gamma.auth.unwrap();
  await gamma.client.editSite(editSiteForm);
  editSiteForm.auth = delta.auth.unwrap();
  await delta.client.editSite(editSiteForm);
  editSiteForm.auth = epsilon.auth.unwrap();
  await epsilon.client.editSite(editSiteForm);

  // Create the main beta community, follow it
  await createCommunity(beta, "main");
  await followBeta(beta);
}

export async function createPost(
  api: API,
  community_id: number
): Promise<PostResponse> {
  let name = randomString(5);
  let body = Some(randomString(10));
  let url = Some('https://google.com/');
  let form = new CreatePost({
    name,
    url,
    body,
    auth: api.auth.unwrap(),
    community_id,
    nsfw: None,
    honeypot: None,
  });
  return api.client.createPost(form);
}

export async function editPost(api: API, post: Post): Promise<PostResponse> {
  let name = Some('A jest test federated post, updated');
  let form = new EditPost({
    name,
    post_id: post.id,
    auth: api.auth.unwrap(),
    nsfw: None,
    url: None,
    body: None,
  });
  return api.client.editPost(form);
}

export async function deletePost(
  api: API,
  deleted: boolean,
  post: Post
): Promise<PostResponse> {
  let form = new DeletePost({
    post_id: post.id,
    deleted: deleted,
    auth: api.auth.unwrap(),
  });
  return api.client.deletePost(form);
}

export async function removePost(
  api: API,
  removed: boolean,
  post: Post
): Promise<PostResponse> {
  let form = new RemovePost({
    post_id: post.id,
    removed,
    auth: api.auth.unwrap(),
    reason: None,
  });
  return api.client.removePost(form);
}

export async function stickyPost(
  api: API,
  stickied: boolean,
  post: Post
): Promise<PostResponse> {
  let form = new StickyPost({
    post_id: post.id,
    stickied,
    auth: api.auth.unwrap(),
  });
  return api.client.stickyPost(form);
}

export async function lockPost(
  api: API,
  locked: boolean,
  post: Post
): Promise<PostResponse> {
  let form = new LockPost({
    post_id: post.id,
    locked,
    auth: api.auth.unwrap(),
  });
  return api.client.lockPost(form);
}

export async function resolvePost(
  api: API,
  post: Post
): Promise<ResolveObjectResponse> {
  let form = new ResolveObject({
    q: post.ap_id,
    auth: api.auth,
  });
  return api.client.resolveObject(form);
}

export async function searchPostLocal(
  api: API,
  post: Post
): Promise<SearchResponse> {
  let form = new Search({
    q: post.name,
    type_: Some(SearchType.Posts),
    sort: Some(SortType.TopAll),
    community_id: None,
    community_name: None,
    creator_id: None,
    listing_type: None,
    page: None,
    limit: None,
    auth: api.auth,
  });
  return api.client.search(form);
}

export async function getPost(
  api: API,
  post_id: number
): Promise<GetPostResponse> {
  let form = new GetPost({
    id: Some(post_id),
    comment_id: None,
    auth: api.auth,
  });
  return api.client.getPost(form);
}

export async function getComments(
  api: API,
  post_id: number
): Promise<GetCommentsResponse> {
  let form = new GetComments({
    post_id: Some(post_id),
    type_: Some(ListingType.All),
    sort: Some(CommentSortType.New), // TODO this sort might be wrong
    max_depth: None,
    page: None,
    limit: None,
    community_id: None,
    community_name: None,
    saved_only: None,
    parent_id: None,
    auth: api.auth,
  });
  return api.client.getComments(form);
}

export async function resolveComment(
  api: API,
  comment: Comment
): Promise<ResolveObjectResponse> {
  let form = new ResolveObject({
    q: comment.ap_id,
    auth: api.auth,
  });
  return api.client.resolveObject(form);
}

export async function resolveBetaCommunity(
  api: API
): Promise<ResolveObjectResponse> {
  // Use short-hand search url
  let form = new ResolveObject({
    q: '!main@lemmy-beta:8551',
    auth: api.auth,
  });
  return api.client.resolveObject(form);
}

export async function resolveCommunity(
  api: API,
  q: string
): Promise<ResolveObjectResponse> {
  let form = new ResolveObject({
    q,
    auth: api.auth,
  });
  return api.client.resolveObject(form);
}

export async function resolvePerson(
  api: API,
  apShortname: string
): Promise<ResolveObjectResponse> {
  let form = new ResolveObject({
    q: apShortname,
    auth: api.auth,
  });
  return api.client.resolveObject(form);
}

export async function banPersonFromSite(
  api: API,
  person_id: number,
  ban: boolean,
  remove_data: boolean
): Promise<BanPersonResponse> {
  // Make sure lemmy-beta/c/main is cached on lemmy_alpha
  let form = new BanPerson({
    person_id,
    ban,
    remove_data: Some(remove_data),
    auth: api.auth.unwrap(),
    reason: None,
    expires: None,
  });
  return api.client.banPerson(form);
}

export async function banPersonFromCommunity(
  api: API,
  person_id: number,
  community_id: number,
  remove_data: boolean,
  ban: boolean
): Promise<BanFromCommunityResponse> {
  let form = new BanFromCommunity({
    person_id,
    community_id,
    remove_data: Some(remove_data),
    ban,
    reason: None,
    expires: None,
    auth: api.auth.unwrap(),
  });
  return api.client.banFromCommunity(form);
}

export async function followCommunity(
  api: API,
  follow: boolean,
  community_id: number
): Promise<CommunityResponse> {
  let form = new FollowCommunity({
    community_id,
    follow,
    auth: api.auth.unwrap()
  });
  return api.client.followCommunity(form);
}

export async function likePost(
  api: API,
  score: number,
  post: Post
): Promise<PostResponse> {
  let form = new CreatePostLike({
    post_id: post.id,
    score: score,
    auth: api.auth.unwrap()
  });

  return api.client.likePost(form);
}

export async function createComment(
  api: API,
  post_id: number,
  parent_id: Option<number>,
  content = 'a jest test comment'
): Promise<CommentResponse> {
  let form = new CreateComment({
    content,
    post_id,
    parent_id,
    form_id: None,
    auth: api.auth.unwrap(),
  });
  return api.client.createComment(form);
}

export async function editComment(
  api: API,
  comment_id: number,
  content = 'A jest test federated comment update'
): Promise<CommentResponse> {
  let form = new EditComment({
    content,
    comment_id,
    form_id: None,
    auth: api.auth.unwrap()
  });
  return api.client.editComment(form);
}

export async function deleteComment(
  api: API,
  deleted: boolean,
  comment_id: number
): Promise<CommentResponse> {
  let form = new DeleteComment({
    comment_id,
    deleted,
    auth: api.auth.unwrap(),
  });
  return api.client.deleteComment(form);
}

export async function removeComment(
  api: API,
  removed: boolean,
  comment_id: number
): Promise<CommentResponse> {
  let form = new RemoveComment({
    comment_id,
    removed,
    reason: None,
    auth: api.auth.unwrap(),
  });
  return api.client.removeComment(form);
}

export async function getMentions(api: API): Promise<GetPersonMentionsResponse> {
  let form = new GetPersonMentions({
    sort: Some(CommentSortType.New),
    unread_only: Some(false),
    auth: api.auth.unwrap(),
    page: None,
    limit: None,
  });
  return api.client.getPersonMentions(form);
}

export async function likeComment(
  api: API,
  score: number,
  comment: Comment
): Promise<CommentResponse> {
  let form = new CreateCommentLike({
    comment_id: comment.id,
    score,
    auth: api.auth.unwrap(),
  });
  return api.client.likeComment(form);
}

export async function createCommunity(
  api: API,
  name_: string = randomString(5)
): Promise<CommunityResponse> {
  let description = Some('a sample description');
  let form = new CreateCommunity({
    name: name_,
    title: name_,
    description,
    nsfw: None,
    icon: None,
    banner: None,
    posting_restricted_to_mods: None,
    auth: api.auth.unwrap(),
  });
  return api.client.createCommunity(form);
}

export async function getCommunity(
  api: API,
  id: number
): Promise<CommunityResponse> {
  let form = new GetCommunity({
    id: Some(id),
    name: None,
    auth: api.auth,
  });
  return api.client.getCommunity(form);
}

export async function deleteCommunity(
  api: API,
  deleted: boolean,
  community_id: number
): Promise<CommunityResponse> {
  let form = new DeleteCommunity({
    community_id,
    deleted,
    auth: api.auth.unwrap(),
  });
  return api.client.deleteCommunity(form);
}

export async function removeCommunity(
  api: API,
  removed: boolean,
  community_id: number
): Promise<CommunityResponse> {
  let form = new RemoveCommunity({
    community_id,
    removed,
    reason: None,
    expires: None,
    auth: api.auth.unwrap(),
  });
  return api.client.removeCommunity(form);
}

export async function createPrivateMessage(
  api: API,
  recipient_id: number
): Promise<PrivateMessageResponse> {
  let content = 'A jest test federated private message';
  let form = new CreatePrivateMessage({
    content,
    recipient_id,
    auth: api.auth.unwrap(),
  });
  return api.client.createPrivateMessage(form);
}

export async function editPrivateMessage(
  api: API,
  private_message_id: number
): Promise<PrivateMessageResponse> {
  let updatedContent = 'A jest test federated private message edited';
  let form = new EditPrivateMessage({
    content: updatedContent,
    private_message_id,
    auth: api.auth.unwrap(),
  });
  return api.client.editPrivateMessage(form);
}

export async function deletePrivateMessage(
  api: API,
  deleted: boolean,
  private_message_id: number
): Promise<PrivateMessageResponse> {
  let form = new DeletePrivateMessage({
    deleted,
    private_message_id,
    auth: api.auth.unwrap(),
  });
  return api.client.deletePrivateMessage(form);
}

export async function registerUser(
  api: API,
  username: string = randomString(5)
): Promise<LoginResponse> {
  let form = new Register({
    username,
    password,
    password_verify: password,
    show_nsfw: true,
    email: None,
    captcha_uuid: None,
    captcha_answer: None,
    honeypot: None,
    answer: None,
  });
  return api.client.register(form);
}

export async function saveUserSettingsBio(
  api: API
): Promise<LoginResponse> {
  let form = new SaveUserSettings({
    show_nsfw: Some(true),
    theme: Some('darkly'),
    default_sort_type: Some(Object.keys(SortType).indexOf(SortType.Active)),
    default_listing_type: Some(Object.keys(ListingType).indexOf(ListingType.All)),
    lang: Some('en'),
    show_avatars: Some(true),
    send_notifications_to_email: Some(false),
    bio: Some('a changed bio'),
    avatar: None,
    banner: None,
    display_name: None,
    email: None,
    matrix_user_id: None,
    show_scores: None,
    show_read_posts: None,
    show_bot_accounts: None,
    show_new_post_notifs: None,
    bot_account: None,
    auth: api.auth.unwrap(),
  });
  return saveUserSettings(api, form);
}

export async function saveUserSettingsFederated(
  api: API
): Promise<LoginResponse> {
  let avatar = Some('https://image.flaticon.com/icons/png/512/35/35896.png');
  let banner = Some('https://image.flaticon.com/icons/png/512/36/35896.png');
  let bio = Some('a changed bio');
  let form = new SaveUserSettings({
    show_nsfw: Some(false),
    theme: Some(''),
    default_sort_type: Some(Object.keys(SortType).indexOf(SortType.Hot)),
    default_listing_type: Some(Object.keys(ListingType).indexOf(ListingType.All)),
    lang: Some(''),
    avatar,
    banner,
    display_name: Some('user321'),
    show_avatars: Some(false),
    send_notifications_to_email: Some(false),
    bio,
    email: None,
    show_scores: None,
    show_read_posts: None,
    matrix_user_id: None,
    bot_account: None,
    show_bot_accounts: None,
    show_new_post_notifs: None,
    auth: api.auth.unwrap(),
  });
  return await saveUserSettings(alpha, form);
}

export async function saveUserSettings(
  api: API,
  form: SaveUserSettings
): Promise<LoginResponse> {
  return api.client.saveUserSettings(form);
}

export async function deleteUser(
  api: API
): Promise<DeleteAccountResponse> {
  let form = new DeleteAccount({
    auth: api.auth.unwrap(),
    password
  });
  return api.client.deleteAccount(form);
}

export async function getSite(
  api: API
): Promise<GetSiteResponse> {
  let form = new GetSite({
    auth: api.auth,
  });
  return api.client.getSite(form);
}

export async function listPrivateMessages(
  api: API
): Promise<PrivateMessagesResponse> {
  let form = new GetPrivateMessages({
    auth: api.auth.unwrap(),
    unread_only: Some(false),
    page: None,
    limit: None,
  });
  return api.client.getPrivateMessages(form);
}

export async function unfollowRemotes(
  api: API
): Promise<GetSiteResponse> {
  // Unfollow all remote communities
  let site = await getSite(api);
  let remoteFollowed = site.my_user.unwrap().follows.filter(
    c => c.community.local == false
  );
  for (let cu of remoteFollowed) {
    await followCommunity(api, false, cu.community.id);
  }
  let siteRes = await getSite(api);
  return siteRes;
}

export async function followBeta(api: API): Promise<CommunityResponse> {
  let betaCommunity = (await resolveBetaCommunity(api)).community;
  if (betaCommunity.isSome()) {
    let follow = await followCommunity(api, true, betaCommunity.unwrap().community.id);
    return follow;
  } else {
    return Promise.reject("no community worked");
  }
}

export async function reportPost(
  api: API,
  post_id: number,
  reason: string
): Promise<PostReportResponse> {
  let form = new CreatePostReport({
    post_id,
    reason,
    auth: api.auth.unwrap(),
  });
  return api.client.createPostReport(form);
}

export async function listPostReports(api: API): Promise<ListPostReportsResponse> {
  let form = new ListPostReports({
    auth: api.auth.unwrap(),
    page: None,
    limit: None,
    community_id: None,
    unresolved_only: None,
  });
  return api.client.listPostReports(form);
}

export async function reportComment(
  api: API,
  comment_id: number,
  reason: string
): Promise<CommentReportResponse> {
  let form = new CreateCommentReport({
    comment_id,
    reason,
    auth: api.auth.unwrap(),
  });
  return api.client.createCommentReport(form);
}

export async function listCommentReports(api: API): Promise<ListCommentReportsResponse> {
  let form = new ListCommentReports({
    page: None,
    limit: None,
    community_id: None,
    unresolved_only: None,
    auth: api.auth.unwrap(),
  });
  return api.client.listCommentReports(form);
}

export function delay(millis: number = 500) {
  return new Promise(resolve => setTimeout(resolve, millis));
}

export function longDelay() {
  return delay(10000);
}

export function wrapper(form: any): string {
  return JSON.stringify(form);
}

export function randomString(length: number): string {
  var result = '';
  var characters = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_';
  var charactersLength = characters.length;
  for (var i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}

export async function unfollows() {
  await unfollowRemotes(alpha);
  await unfollowRemotes(gamma);
  await unfollowRemotes(delta);
  await unfollowRemotes(epsilon);
}

export function getCommentParentId(comment: Comment): Option<number> {
  let split = comment.path.split(".");
  // remove the 0
  split.shift();

  if (split.length > 1) {
    return Some(Number(split[split.length - 2]));
  } else {
    return None;
  }
}
