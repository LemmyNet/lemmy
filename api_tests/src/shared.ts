import {
  LoginForm,
  LoginResponse,
  Post,
  PostForm,
  Comment,
  DeletePostForm,
  RemovePostForm,
  StickyPostForm,
  LockPostForm,
  PostResponse,
  SearchResponse,
  FollowCommunityForm,
  CommunityResponse,
  GetFollowedCommunitiesResponse,
  GetPostResponse,
  RegisterForm,
  CommentForm,
  DeleteCommentForm,
  RemoveCommentForm,
  SearchForm,
  CommentResponse,
  GetCommunityForm,
  CommunityForm,
  DeleteCommunityForm,
  RemoveCommunityForm,
  GetUserMentionsForm,
  CommentLikeForm,
  CreatePostLikeForm,
  PrivateMessageForm,
  EditPrivateMessageForm,
  DeletePrivateMessageForm,
  GetFollowedCommunitiesForm,
  GetPrivateMessagesForm,
  GetSiteForm,
  GetPostForm,
  PrivateMessageResponse,
  PrivateMessagesResponse,
  GetUserMentionsResponse,
  UserSettingsForm,
  SortType,
  ListingType,
  GetSiteResponse,
  SearchType,
  LemmyHttp,
} from 'lemmy-js-client';

export interface API {
  client: LemmyHttp;
  auth?: string;
}

export let alpha: API = {
  client: new LemmyHttp('http://localhost:8541/api/v1'),
};

export let beta: API = {
  client: new LemmyHttp('http://localhost:8551/api/v1'),
};

export let gamma: API = {
  client: new LemmyHttp('http://localhost:8561/api/v1'),
};

export let delta: API = {
  client: new LemmyHttp('http://localhost:8571/api/v1'),
};

export let epsilon: API = {
  client: new LemmyHttp('http://localhost:8581/api/v1'),
};

export async function setupLogins() {
  let formAlpha: LoginForm = {
    username_or_email: 'lemmy_alpha',
    password: 'lemmy',
  };
  let resAlpha = alpha.client.login(formAlpha);

  let formBeta = {
    username_or_email: 'lemmy_beta',
    password: 'lemmy',
  };
  let resBeta = beta.client.login(formBeta);

  let formGamma = {
    username_or_email: 'lemmy_gamma',
    password: 'lemmy',
  };
  let resGamma = gamma.client.login(formGamma);

  let formDelta = {
    username_or_email: 'lemmy_delta',
    password: 'lemmy',
  };
  let resDelta = delta.client.login(formDelta);

  let formEpsilon = {
    username_or_email: 'lemmy_epsilon',
    password: 'lemmy',
  };
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
}

export async function createPost(
  api: API,
  community_id: number
): Promise<PostResponse> {
  let name = 'A jest test post';
  let body = 'Some body';
  let url = 'https://google.com/';
  let form: PostForm = {
    name,
    url,
    body,
    auth: api.auth,
    community_id,
    nsfw: false,
  };
  return api.client.createPost(form);
}

export async function updatePost(api: API, post: Post): Promise<PostResponse> {
  let name = 'A jest test federated post, updated';
  let form: PostForm = {
    name,
    edit_id: post.id,
    auth: api.auth,
    nsfw: false,
  };
  return api.client.editPost(form);
}

export async function deletePost(
  api: API,
  deleted: boolean,
  post: Post
): Promise<PostResponse> {
  let form: DeletePostForm = {
    edit_id: post.id,
    deleted: deleted,
    auth: api.auth,
  };
  return api.client.deletePost(form);
}

export async function removePost(
  api: API,
  removed: boolean,
  post: Post
): Promise<PostResponse> {
  let form: RemovePostForm = {
    edit_id: post.id,
    removed,
    auth: api.auth,
  };
  return api.client.removePost(form);
}

export async function stickyPost(
  api: API,
  stickied: boolean,
  post: Post
): Promise<PostResponse> {
  let form: StickyPostForm = {
    edit_id: post.id,
    stickied,
    auth: api.auth,
  };
  return api.client.stickyPost(form);
}

export async function lockPost(
  api: API,
  locked: boolean,
  post: Post
): Promise<PostResponse> {
  let form: LockPostForm = {
    edit_id: post.id,
    locked,
    auth: api.auth,
  };
  return api.client.lockPost(form);
}

export async function searchPost(
  api: API,
  post: Post
): Promise<SearchResponse> {
  let form: SearchForm = {
    q: post.ap_id,
    type_: SearchType.Posts,
    sort: SortType.TopAll,
  };
  return api.client.search(form);
}

export async function getPost(
  api: API,
  post_id: number
): Promise<GetPostResponse> {
  let form: GetPostForm = {
    id: post_id,
  };
  return api.client.getPost(form);
}

export async function searchComment(
  api: API,
  comment: Comment
): Promise<SearchResponse> {
  let form: SearchForm = {
    q: comment.ap_id,
    type_: SearchType.Comments,
    sort: SortType.TopAll,
  };
  return api.client.search(form);
}

export async function searchForBetaCommunity(
  api: API
): Promise<SearchResponse> {
  // Make sure lemmy-beta/c/main is cached on lemmy_alpha
  // Use short-hand search url
  let form: SearchForm = {
    q: '!main@lemmy-beta:8551',
    type_: SearchType.Communities,
    sort: SortType.TopAll,
  };
  return api.client.search(form);
}

export async function searchForCommunity(
  api: API,
  q: string,
): Promise<SearchResponse> {
  // Use short-hand search url
  let form: SearchForm = {
    q,
    type_: SearchType.Communities,
    sort: SortType.TopAll,
  };
  return api.client.search(form);
}

export async function searchForUser(
  api: API,
  apShortname: string
): Promise<SearchResponse> {
  // Make sure lemmy-beta/c/main is cached on lemmy_alpha
  // Use short-hand search url
  let form: SearchForm = {
    q: apShortname,
    type_: SearchType.Users,
    sort: SortType.TopAll,
  };
  return api.client.search(form);
}

export async function followCommunity(
  api: API,
  follow: boolean,
  community_id: number
): Promise<CommunityResponse> {
  let form: FollowCommunityForm = {
    community_id,
    follow,
    auth: api.auth,
  };
  return api.client.followCommunity(form);
}

export async function checkFollowedCommunities(
  api: API
): Promise<GetFollowedCommunitiesResponse> {
  let form: GetFollowedCommunitiesForm = {
    auth: api.auth,
  };
  return api.client.getFollowedCommunities(form);
}

export async function likePost(
  api: API,
  score: number,
  post: Post
): Promise<PostResponse> {
  let form: CreatePostLikeForm = {
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
  content = 'a jest test comment'
): Promise<CommentResponse> {
  let form: CommentForm = {
    content,
    post_id,
    parent_id,
    auth: api.auth,
  };
  return api.client.createComment(form);
}

export async function updateComment(
  api: API,
  edit_id: number,
  content = 'A jest test federated comment update'
): Promise<CommentResponse> {
  let form: CommentForm = {
    content,
    edit_id,
    auth: api.auth,
  };
  return api.client.editComment(form);
}

export async function deleteComment(
  api: API,
  deleted: boolean,
  edit_id: number
): Promise<CommentResponse> {
  let form: DeleteCommentForm = {
    edit_id,
    deleted,
    auth: api.auth,
  };
  return api.client.deleteComment(form);
}

export async function removeComment(
  api: API,
  removed: boolean,
  edit_id: number
): Promise<CommentResponse> {
  let form: RemoveCommentForm = {
    edit_id,
    removed,
    auth: api.auth,
  };
  return api.client.removeComment(form);
}

export async function getMentions(api: API): Promise<GetUserMentionsResponse> {
  let form: GetUserMentionsForm = {
    sort: SortType.New,
    unread_only: false,
    auth: api.auth,
  };
  return api.client.getUserMentions(form);
}

export async function likeComment(
  api: API,
  score: number,
  comment: Comment
): Promise<CommentResponse> {
  let form: CommentLikeForm = {
    comment_id: comment.id,
    score,
    auth: api.auth,
  };
  return api.client.likeComment(form);
}

export async function createCommunity(
  api: API,
  name_: string = randomString(5)
): Promise<CommunityResponse> {
  let description = 'a sample description';
  let icon = 'https://image.flaticon.com/icons/png/512/35/35896.png';
  let banner = 'https://image.flaticon.com/icons/png/512/35/35896.png';
  let form: CommunityForm = {
    name: name_,
    title: name_,
    description,
    icon,
    banner,
    category_id: 1,
    nsfw: false,
    auth: api.auth,
  };
  return api.client.createCommunity(form);
}

export async function getCommunity(
  api: API,
  id: number,
): Promise<CommunityResponse> {
  let form: GetCommunityForm = {
    id,
  };
  return api.client.getCommunity(form);
}

export async function deleteCommunity(
  api: API,
  deleted: boolean,
  edit_id: number
): Promise<CommunityResponse> {
  let form: DeleteCommunityForm = {
    edit_id,
    deleted,
    auth: api.auth,
  };
  return api.client.deleteCommunity(form);
}

export async function removeCommunity(
  api: API,
  removed: boolean,
  edit_id: number
): Promise<CommunityResponse> {
  let form: RemoveCommunityForm = {
    edit_id,
    removed,
    auth: api.auth,
  };
  return api.client.removeCommunity(form);
}

export async function createPrivateMessage(
  api: API,
  recipient_id: number
): Promise<PrivateMessageResponse> {
  let content = 'A jest test federated private message';
  let form: PrivateMessageForm = {
    content,
    recipient_id,
    auth: api.auth,
  };
  return api.client.createPrivateMessage(form);
}

export async function updatePrivateMessage(
  api: API,
  edit_id: number
): Promise<PrivateMessageResponse> {
  let updatedContent = 'A jest test federated private message edited';
  let form: EditPrivateMessageForm = {
    content: updatedContent,
    edit_id,
    auth: api.auth,
  };
  return api.client.editPrivateMessage(form);
}

export async function deletePrivateMessage(
  api: API,
  deleted: boolean,
  edit_id: number
): Promise<PrivateMessageResponse> {
  let form: DeletePrivateMessageForm = {
    deleted,
    edit_id,
    auth: api.auth,
  };
  return api.client.deletePrivateMessage(form);
}

export async function registerUser(
  api: API,
  username: string = randomString(5)
): Promise<LoginResponse> {
  let form: RegisterForm = {
    username,
    password: 'test',
    password_verify: 'test',
    admin: false,
    show_nsfw: true,
  };
  return api.client.register(form);
}

export async function saveUserSettingsBio(
  api: API,
  auth: string
): Promise<LoginResponse> {
  let form: UserSettingsForm = {
    show_nsfw: true,
    theme: 'darkly',
    default_sort_type: Object.keys(SortType).indexOf(SortType.Active),
    default_listing_type: Object.keys(ListingType).indexOf(ListingType.All),
    lang: 'en',
    show_avatars: true,
    send_notifications_to_email: false,
    bio: 'a changed bio',
    auth,
  };
  return saveUserSettings(api, form);
}

export async function saveUserSettings(
  api: API,
  form: UserSettingsForm
): Promise<LoginResponse> {
  return api.client.saveUserSettings(form);
}

export async function getSite(
  api: API,
  auth: string
): Promise<GetSiteResponse> {
  let form: GetSiteForm = {
    auth,
  };
  return api.client.getSite(form);
}

export async function listPrivateMessages(
  api: API
): Promise<PrivateMessagesResponse> {
  let form: GetPrivateMessagesForm = {
    auth: api.auth,
    unread_only: false,
    limit: 999,
  };
  return api.client.getPrivateMessages(form);
}

export async function unfollowRemotes(
  api: API
): Promise<GetFollowedCommunitiesResponse> {
  // Unfollow all remote communities
  let followed = await checkFollowedCommunities(api);
  let remoteFollowed = followed.communities.filter(
    c => c.community_local == false
  );
  for (let cu of remoteFollowed) {
    await followCommunity(api, false, cu.community_id);
  }
  let followed2 = await checkFollowedCommunities(api);
  return followed2;
}

export async function followBeta(api: API): Promise<CommunityResponse> {
  await unfollowRemotes(api);

  // Cache it
  let search = await searchForBetaCommunity(api);
  let com = search.communities.filter(c => c.local == false);
  if (com[0]) {
    let follow = await followCommunity(api, true, com[0].id);
    return follow;
  }
}

export function delay(millis: number = 500) {
  return new Promise((resolve, _reject) => {
    setTimeout(_ => resolve(), millis);
  });
}

export function longDelay() {
  return delay(10000);
}

export function wrapper(form: any): string {
  return JSON.stringify(form);
}

function randomString(length: number): string {
  var result = '';
  var characters = 'abcdefghijklmnopqrstuvwxyz0123456789_';
  var charactersLength = characters.length;
  for (var i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}
