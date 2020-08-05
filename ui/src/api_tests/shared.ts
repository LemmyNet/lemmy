import fetch from 'node-fetch';

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
  CommentResponse,
  CommunityForm,
  DeleteCommunityForm,
  RemoveCommunityForm,
  CommentLikeForm,
  CreatePostLikeForm,
  PrivateMessageForm,
  EditPrivateMessageForm,
  DeletePrivateMessageForm,
  PrivateMessageResponse,
  PrivateMessagesResponse,
  GetUserMentionsResponse,
  UserSettingsForm,
  SortType,
  ListingType,
  GetSiteResponse,
} from '../interfaces';

export interface API {
  url: string;
  auth?: string;
}

function apiUrl(api: API) {
  return `${api.url}/api/v1`;
}

export let alpha: API = {
  url: 'http://localhost:8540',
};

export let beta: API = {
  url: 'http://localhost:8550',
};

export let gamma: API = {
  url: 'http://localhost:8560',
};

export async function setupLogins() {
  let form: LoginForm = {
    username_or_email: 'lemmy_alpha',
    password: 'lemmy',
  };

  let resA: Promise<LoginResponse> = fetch(`${apiUrl(alpha)}/user/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(form),
  }).then(d => d.json());

  let formB = {
    username_or_email: 'lemmy_beta',
    password: 'lemmy',
  };

  let resB: Promise<LoginResponse> = fetch(`${apiUrl(beta)}/user/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(formB),
  }).then(d => d.json());

  let formC = {
    username_or_email: 'lemmy_gamma',
    password: 'lemmy',
  };

  let resG: Promise<LoginResponse> = fetch(`${apiUrl(gamma)}/user/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(formC),
  }).then(d => d.json());

  let res = await Promise.all([resA, resB, resG]);
  alpha.auth = res[0].jwt;
  beta.auth = res[1].jwt;
  gamma.auth = res[2].jwt;
}

export async function createPost(
  api: API,
  community_id: number
): Promise<PostResponse> {
  let name = 'A jest test post';
  let postForm: PostForm = {
    name,
    auth: api.auth,
    community_id,
    nsfw: false,
  };

  let createPostRes: PostResponse = await fetch(`${apiUrl(api)}/post`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(postForm),
  }).then(d => d.json());
  return createPostRes;
}

export async function updatePost(api: API, post: Post): Promise<PostResponse> {
  let name = 'A jest test federated post, updated';
  let postForm: PostForm = {
    name,
    edit_id: post.id,
    auth: api.auth,
    nsfw: false,
  };

  let updateResponse: PostResponse = await fetch(`${apiUrl(api)}/post`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(postForm),
  }).then(d => d.json());
  return updateResponse;
}

export async function deletePost(
  api: API,
  deleted: boolean,
  post: Post
): Promise<PostResponse> {
  let deletePostForm: DeletePostForm = {
    edit_id: post.id,
    deleted: deleted,
    auth: api.auth,
  };

  let deletePostRes: PostResponse = await fetch(`${apiUrl(api)}/post/delete`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(deletePostForm),
  }).then(d => d.json());
  return deletePostRes;
}

export async function removePost(
  api: API,
  removed: boolean,
  post: Post
): Promise<PostResponse> {
  let removePostForm: RemovePostForm = {
    edit_id: post.id,
    removed,
    auth: api.auth,
  };

  let removePostRes: PostResponse = await fetch(`${apiUrl(api)}/post/remove`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(removePostForm),
  }).then(d => d.json());
  return removePostRes;
}

export async function stickyPost(
  api: API,
  stickied: boolean,
  post: Post
): Promise<PostResponse> {
  let stickyPostForm: StickyPostForm = {
    edit_id: post.id,
    stickied,
    auth: api.auth,
  };

  let stickyRes: PostResponse = await fetch(`${apiUrl(api)}/post/sticky`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(stickyPostForm),
  }).then(d => d.json());

  return stickyRes;
}

export async function lockPost(
  api: API,
  locked: boolean,
  post: Post
): Promise<PostResponse> {
  let lockPostForm: LockPostForm = {
    edit_id: post.id,
    locked,
    auth: api.auth,
  };

  let lockRes: PostResponse = await fetch(`${apiUrl(api)}/post/lock`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(lockPostForm),
  }).then(d => d.json());

  return lockRes;
}

export async function searchPost(
  api: API,
  post: Post
): Promise<SearchResponse> {
  let searchUrl = `${apiUrl(api)}/search?q=${post.ap_id}&type_=All&sort=TopAll`;
  let searchResponse: SearchResponse = await fetch(searchUrl, {
    method: 'GET',
  }).then(d => d.json());
  return searchResponse;
}

export async function getPost(
  api: API,
  post_id: number
): Promise<GetPostResponse> {
  let getPostUrl = `${apiUrl(api)}/post?id=${post_id}`;
  let getPostRes: GetPostResponse = await fetch(getPostUrl, {
    method: 'GET',
  }).then(d => d.json());

  return getPostRes;
}

export async function searchComment(
  api: API,
  comment: Comment
): Promise<SearchResponse> {
  let searchUrl = `${apiUrl(api)}/search?q=${
    comment.ap_id
  }&type_=All&sort=TopAll`;
  let searchResponse: SearchResponse = await fetch(searchUrl, {
    method: 'GET',
  }).then(d => d.json());
  return searchResponse;
}

export async function searchForBetaCommunity(
  api: API
): Promise<SearchResponse> {
  // Make sure lemmy-beta/c/main is cached on lemmy_alpha
  // Use short-hand search url
  let searchUrl = `${apiUrl(
    api
  )}/search?q=!main@lemmy-beta:8550&type_=All&sort=TopAll`;

  let searchResponse: SearchResponse = await fetch(searchUrl, {
    method: 'GET',
  }).then(d => d.json());
  return searchResponse;
}

export async function searchForUser(
  api: API,
  apShortname: string
): Promise<SearchResponse> {
  // Make sure lemmy-beta/c/main is cached on lemmy_alpha
  // Use short-hand search url
  let searchUrl = `${apiUrl(
    api
  )}/search?q=${apShortname}&type_=All&sort=TopAll`;

  let searchResponse: SearchResponse = await fetch(searchUrl, {
    method: 'GET',
  }).then(d => d.json());
  return searchResponse;
}

export async function followCommunity(
  api: API,
  follow: boolean,
  community_id: number
): Promise<CommunityResponse> {
  let followForm: FollowCommunityForm = {
    community_id,
    follow,
    auth: api.auth,
  };

  let followRes: CommunityResponse = await fetch(
    `${apiUrl(api)}/community/follow`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(followForm),
    }
  )
    .then(d => d.json())
    .catch(_e => {});

  return followRes;
}

export async function checkFollowedCommunities(
  api: API
): Promise<GetFollowedCommunitiesResponse> {
  let followedCommunitiesUrl = `${apiUrl(
    api
  )}/user/followed_communities?&auth=${api.auth}`;
  let followedCommunitiesRes: GetFollowedCommunitiesResponse = await fetch(
    followedCommunitiesUrl,
    {
      method: 'GET',
    }
  ).then(d => d.json());
  return followedCommunitiesRes;
}

export async function likePost(
  api: API,
  score: number,
  post: Post
): Promise<PostResponse> {
  let likePostForm: CreatePostLikeForm = {
    post_id: post.id,
    score: score,
    auth: api.auth,
  };

  let likePostRes: PostResponse = await fetch(`${apiUrl(api)}/post/like`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(likePostForm),
  }).then(d => d.json());

  return likePostRes;
}

export async function createComment(
  api: API,
  post_id: number,
  parent_id?: number,
  content = 'a jest test comment'
): Promise<CommentResponse> {
  let commentForm: CommentForm = {
    content,
    post_id,
    parent_id,
    auth: api.auth,
  };

  let createResponse: CommentResponse = await fetch(`${apiUrl(api)}/comment`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(commentForm),
  }).then(d => d.json());
  return createResponse;
}

export async function updateComment(
  api: API,
  edit_id: number,
  content = 'A jest test federated comment update'
): Promise<CommentResponse> {
  let commentForm: CommentForm = {
    content,
    edit_id,
    auth: api.auth,
  };

  let updateResponse: CommentResponse = await fetch(`${apiUrl(api)}/comment`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(commentForm),
  }).then(d => d.json());
  return updateResponse;
}

export async function deleteComment(
  api: API,
  deleted: boolean,
  edit_id: number
): Promise<CommentResponse> {
  let deleteCommentForm: DeleteCommentForm = {
    edit_id,
    deleted,
    auth: api.auth,
  };

  let deleteCommentRes: CommentResponse = await fetch(
    `${apiUrl(api)}/comment/delete`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(deleteCommentForm),
    }
  ).then(d => d.json());
  return deleteCommentRes;
}

export async function removeComment(
  api: API,
  removed: boolean,
  edit_id: number
): Promise<CommentResponse> {
  let removeCommentForm: RemoveCommentForm = {
    edit_id,
    removed,
    auth: api.auth,
  };

  let removeCommentRes: CommentResponse = await fetch(
    `${apiUrl(api)}/comment/remove`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(removeCommentForm),
    }
  ).then(d => d.json());
  return removeCommentRes;
}

export async function getMentions(api: API): Promise<GetUserMentionsResponse> {
  let getMentionUrl = `${apiUrl(
    api
  )}/user/mention?sort=New&unread_only=false&auth=${api.auth}`;
  let getMentionsRes: GetUserMentionsResponse = await fetch(getMentionUrl, {
    method: 'GET',
  }).then(d => d.json());
  return getMentionsRes;
}

export async function likeComment(
  api: API,
  score: number,
  comment: Comment
): Promise<CommentResponse> {
  let likeCommentForm: CommentLikeForm = {
    comment_id: comment.id,
    score,
    auth: api.auth,
  };

  let likeCommentRes: CommentResponse = await fetch(
    `${apiUrl(api)}/comment/like`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(likeCommentForm),
    }
  ).then(d => d.json());
  return likeCommentRes;
}

export async function createCommunity(
  api: API,
  name_: string = randomString(5)
): Promise<CommunityResponse> {
  let communityForm: CommunityForm = {
    name: name_,
    title: name_,
    category_id: 1,
    nsfw: false,
    auth: api.auth,
  };

  let createCommunityRes: CommunityResponse = await fetch(
    `${apiUrl(api)}/community`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(communityForm),
    }
  ).then(d => d.json());
  return createCommunityRes;
}

export async function deleteCommunity(
  api: API,
  deleted: boolean,
  edit_id: number
): Promise<CommunityResponse> {
  let deleteCommunityForm: DeleteCommunityForm = {
    edit_id,
    deleted,
    auth: api.auth,
  };

  let deleteResponse: CommunityResponse = await fetch(
    `${apiUrl(api)}/community/delete`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(deleteCommunityForm),
    }
  ).then(d => d.json());
  return deleteResponse;
}

export async function removeCommunity(
  api: API,
  removed: boolean,
  edit_id: number
): Promise<CommunityResponse> {
  let removeCommunityForm: RemoveCommunityForm = {
    edit_id,
    removed,
    auth: api.auth,
  };

  let removeResponse: CommunityResponse = await fetch(
    `${apiUrl(api)}/community/remove`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(removeCommunityForm),
    }
  ).then(d => d.json());
  return removeResponse;
}

export async function createPrivateMessage(
  api: API,
  recipient_id: number
): Promise<PrivateMessageResponse> {
  let content = 'A jest test federated private message';
  let privateMessageForm: PrivateMessageForm = {
    content,
    recipient_id,
    auth: api.auth,
  };

  let createRes: PrivateMessageResponse = await fetch(
    `${apiUrl(api)}/private_message`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(privateMessageForm),
    }
  ).then(d => d.json());
  return createRes;
}

export async function updatePrivateMessage(
  api: API,
  edit_id: number
): Promise<PrivateMessageResponse> {
  let updatedContent = 'A jest test federated private message edited';
  let updatePrivateMessageForm: EditPrivateMessageForm = {
    content: updatedContent,
    edit_id,
    auth: api.auth,
  };

  let updateRes: PrivateMessageResponse = await fetch(
    `${apiUrl(api)}/private_message`,
    {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(updatePrivateMessageForm),
    }
  ).then(d => d.json());
  return updateRes;
}

export async function deletePrivateMessage(
  api: API,
  deleted: boolean,
  edit_id: number
): Promise<PrivateMessageResponse> {
  let deletePrivateMessageForm: DeletePrivateMessageForm = {
    deleted,
    edit_id,
    auth: api.auth,
  };

  let deleteRes: PrivateMessageResponse = await fetch(
    `${apiUrl(api)}/private_message/delete`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(deletePrivateMessageForm),
    }
  ).then(d => d.json());

  return deleteRes;
}

export async function registerUser(
  api: API,
  username: string = randomString(5)
): Promise<LoginResponse> {
  let registerForm: RegisterForm = {
    username,
    password: 'test',
    password_verify: 'test',
    admin: false,
    show_nsfw: true,
  };

  let registerRes: Promise<LoginResponse> = fetch(
    `${apiUrl(api)}/user/register`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(registerForm),
    }
  ).then(d => d.json());

  return registerRes;
}

export async function saveUserSettingsBio(
  api: API,
  auth: string
): Promise<LoginResponse> {
  let form: UserSettingsForm = {
    show_nsfw: true,
    theme: 'darkly',
    default_sort_type: SortType.Active,
    default_listing_type: ListingType.All,
    lang: 'en',
    show_avatars: true,
    send_notifications_to_email: false,
    bio: 'a changed bio',
    auth,
  };

  let res: Promise<LoginResponse> = fetch(
    `${apiUrl(api)}/user/save_user_settings`,
    {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(form),
    }
  ).then(d => d.json());
  return res;
}

export async function getSite(
  api: API,
  auth: string
): Promise<GetSiteResponse> {
  let siteUrl = `${apiUrl(api)}/site?auth=${auth}`;

  let res: GetSiteResponse = await fetch(siteUrl, {
    method: 'GET',
  }).then(d => d.json());
  return res;
}

export async function listPrivateMessages(
  api: API
): Promise<PrivateMessagesResponse> {
  let getPrivateMessagesUrl = `${apiUrl(api)}/private_message/list?auth=${
    api.auth
  }&unread_only=false&limit=999`;

  let getPrivateMessagesRes: PrivateMessagesResponse = await fetch(
    getPrivateMessagesUrl,
    {
      method: 'GET',
    }
  ).then(d => d.json());
  return getPrivateMessagesRes;
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
