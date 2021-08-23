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
} from 'lemmy-js-client';

export interface API {
  client: LemmyHttp;
  auth?: string;
}

export let alpha: API = {
  client: new LemmyHttp('http://localhost:8541'),
};

export let beta: API = {
  client: new LemmyHttp('http://localhost:8551'),
};

export let gamma: API = {
  client: new LemmyHttp('http://localhost:8561'),
};

export let delta: API = {
  client: new LemmyHttp('http://localhost:8571'),
};

export let epsilon: API = {
  client: new LemmyHttp('http://localhost:8581'),
};

export async function setupLogins() {
  let formAlpha: Login = {
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
  let name = randomString(5);
  let body = randomString(10);
  let url = 'https://google.com/';
  let form: CreatePost = {
    name,
    url,
    body,
    auth: api.auth,
    community_id,
    nsfw: false,
  };
  return api.client.createPost(form);
}

export async function editPost(api: API, post: Post): Promise<PostResponse> {
  let name = 'A jest test federated post, updated';
  let form: EditPost = {
    name,
    post_id: post.id,
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
  post: Post
): Promise<PostResponse> {
  let form: RemovePost = {
    post_id: post.id,
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
  let form: StickyPost = {
    post_id: post.id,
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
  let form: LockPost = {
    post_id: post.id,
    locked,
    auth: api.auth,
  };
  return api.client.lockPost(form);
}

export async function resolvePost(
  api: API,
  post: Post
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q: post.ap_id,
  };
  return api.client.resolveObject(form);
}

export async function searchPostLocal(
  api: API,
  post: Post
): Promise<SearchResponse> {
  let form: Search = {
    q: post.name,
    type_: SearchType.Posts,
    sort: SortType.TopAll,
  };
  return api.client.search(form);
}

export async function getPost(
  api: API,
  post_id: number
): Promise<GetPostResponse> {
  let form: GetPost = {
    id: post_id,
  };
  return api.client.getPost(form);
}

export async function resolveComment(
  api: API,
  comment: Comment
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q: comment.ap_id,
  };
  return api.client.resolveObject(form);
}

export async function resolveBetaCommunity(
  api: API
): Promise<ResolveObjectResponse> {
  // Use short-hand search url
  let form: ResolveObject = {
    q: '!main@lemmy-beta:8551',
  };
  return api.client.resolveObject(form);
}

export async function resolveCommunity(
  api: API,
  q: string
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q,
  };
  return api.client.resolveObject(form);
}

export async function resolvePerson(
  api: API,
  apShortname: string
): Promise<ResolveObjectResponse> {
  let form: ResolveObject = {
    q: apShortname,
  };
  return api.client.resolveObject(form);
}

export async function banPersonFromSite(
  api: API,
  person_id: number,
  ban: boolean
): Promise<BanPersonResponse> {
  // Make sure lemmy-beta/c/main is cached on lemmy_alpha
  let form: BanPerson = {
    person_id,
    ban,
    remove_data: false,
    auth: api.auth,
  };
  return api.client.banPerson(form);
}

export async function banPersonFromCommunity(
  api: API,
  person_id: number,
  community_id: number,
  ban: boolean
): Promise<BanFromCommunityResponse> {
  // Make sure lemmy-beta/c/main is cached on lemmy_alpha
  let form: BanFromCommunity = {
    person_id,
    community_id,
    remove_data: false,
    ban,
    auth: api.auth,
  };
  return api.client.banFromCommunity(form);
}

export async function followCommunity(
  api: API,
  follow: boolean,
  community_id: number
): Promise<CommunityResponse> {
  let form: FollowCommunity = {
    community_id,
    follow,
    auth: api.auth,
  };
  return api.client.followCommunity(form);
}

export async function likePost(
  api: API,
  score: number,
  post: Post
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
  content = 'a jest test comment'
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
  content = 'A jest test federated comment update'
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
  comment_id: number
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
  comment_id: number
): Promise<CommentResponse> {
  let form: RemoveComment = {
    comment_id,
    removed,
    auth: api.auth,
  };
  return api.client.removeComment(form);
}

export async function getMentions(api: API): Promise<GetPersonMentionsResponse> {
  let form: GetPersonMentions = {
    sort: SortType.New,
    unread_only: false,
    auth: api.auth,
  };
  return api.client.getPersonMentions(form);
}

export async function likeComment(
  api: API,
  score: number,
  comment: Comment
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
  name_: string = randomString(5)
): Promise<CommunityResponse> {
  let description = 'a sample description';
  let icon = 'https://image.flaticon.com/icons/png/512/35/35896.png';
  let banner = 'https://image.flaticon.com/icons/png/512/35/35896.png';
  let form: CreateCommunity = {
    name: name_,
    title: name_,
    description,
    icon,
    banner,
    nsfw: false,
    auth: api.auth,
  };
  return api.client.createCommunity(form);
}

export async function getCommunity(
  api: API,
  id: number
): Promise<CommunityResponse> {
  let form: GetCommunity = {
    id,
  };
  return api.client.getCommunity(form);
}

export async function deleteCommunity(
  api: API,
  deleted: boolean,
  community_id: number
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
  community_id: number
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
  recipient_id: number
): Promise<PrivateMessageResponse> {
  let content = 'A jest test federated private message';
  let form: CreatePrivateMessage = {
    content,
    recipient_id,
    auth: api.auth,
  };
  return api.client.createPrivateMessage(form);
}

export async function editPrivateMessage(
  api: API,
  private_message_id: number
): Promise<PrivateMessageResponse> {
  let updatedContent = 'A jest test federated private message edited';
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
  private_message_id: number
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
  username: string = randomString(5)
): Promise<LoginResponse> {
  let form: Register = {
    username,
    password: 'test',
    password_verify: 'test',
    show_nsfw: true,
  };
  return api.client.register(form);
}

export async function saveUserSettingsBio(
  api: API
): Promise<LoginResponse> {
  let form: SaveUserSettings = {
    show_nsfw: true,
    theme: 'darkly',
    default_sort_type: Object.keys(SortType).indexOf(SortType.Active),
    default_listing_type: Object.keys(ListingType).indexOf(ListingType.All),
    lang: 'en',
    show_avatars: true,
    send_notifications_to_email: false,
    bio: 'a changed bio',
    auth: api.auth,
  };
  return saveUserSettings(api, form);
}

export async function saveUserSettings(
  api: API,
  form: SaveUserSettings
): Promise<LoginResponse> {
  return api.client.saveUserSettings(form);
}

export async function getSite(
  api: API
): Promise<GetSiteResponse> {
  let form: GetSite = {
    auth: api.auth,
  };
  return api.client.getSite(form);
}

export async function listPrivateMessages(
  api: API
): Promise<PrivateMessagesResponse> {
  let form: GetPrivateMessages = {
    auth: api.auth,
    unread_only: false,
    limit: 999,
  };
  return api.client.getPrivateMessages(form);
}

export async function unfollowRemotes(
  api: API
): Promise<GetSiteResponse> {
  // Unfollow all remote communities
  let site = await getSite(api);
  let remoteFollowed = site.my_user.follows.filter(
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
  if (betaCommunity) {
    let follow = await followCommunity(api, true, betaCommunity.community.id);
    return follow;
  }
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

function randomString(length: number): string {
  var result = '';
  var characters = 'abcdefghijklmnopqrstuvwxyz0123456789_';
  var charactersLength = characters.length;
  for (var i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}
