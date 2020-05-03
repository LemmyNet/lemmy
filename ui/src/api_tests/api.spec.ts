import fetch from 'node-fetch';

import {
  LoginForm,
  LoginResponse,
  PostForm,
  PostResponse,
  SearchResponse,
  FollowCommunityForm,
  CommunityResponse,
  GetFollowedCommunitiesResponse,
  GetPostForm,
  GetPostResponse,
  CommentForm,
  CommentResponse,
  CommunityForm,
  GetCommunityForm,
  GetCommunityResponse,
} from '../interfaces';

let lemmyAlphaUrl = 'http://localhost:8540';
let lemmyBetaUrl = 'http://localhost:8550';
let lemmyAlphaApiUrl = `${lemmyAlphaUrl}/api/v1`;
let lemmyBetaApiUrl = `${lemmyBetaUrl}/api/v1`;
let lemmyAlphaAuth: string;
let lemmyBetaAuth: string;

// Workaround for tests being run before beforeAll() is finished
// https://github.com/facebook/jest/issues/9527#issuecomment-592406108
describe('main', () => {
  beforeAll(async () => {
    console.log('Logging in as lemmy_alpha');
    let form: LoginForm = {
      username_or_email: 'lemmy_alpha',
      password: 'lemmy',
    };

    let res: LoginResponse = await fetch(`${lemmyAlphaApiUrl}/user/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(form),
    }).then(d => d.json());

    lemmyAlphaAuth = res.jwt;

    console.log('Logging in as lemmy_beta');
    let formB = {
      username_or_email: 'lemmy_beta',
      password: 'lemmy',
    };

    let resB: LoginResponse = await fetch(`${lemmyBetaApiUrl}/user/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(formB),
    }).then(d => d.json());

    lemmyBetaAuth = resB.jwt;
  });

  describe('beta_fetch', () => {
    test('Create test post on alpha and fetch it on beta', async () => {
      let name = 'A jest test post';
      let postForm: PostForm = {
        name,
        auth: lemmyAlphaAuth,
        community_id: 2,
        creator_id: 2,
        nsfw: false,
      };

      let createResponse: PostResponse = await fetch(
        `${lemmyAlphaApiUrl}/post`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(postForm),
        }
      ).then(d => d.json());
      expect(createResponse.post.name).toBe(name);

      let searchUrl = `${lemmyBetaApiUrl}/search?q=${createResponse.post.ap_id}&type_=All&sort=TopAll`;
      let searchResponse: SearchResponse = await fetch(searchUrl, {
        method: 'GET',
      }).then(d => d.json());

      // TODO: check more fields
      expect(searchResponse.posts[0].name).toBe(name);
    });
  });

  describe('follow_accept', () => {
    test('/u/lemmy_alpha follows and accepts lemmy_beta/c/main', async () => {
      // Make sure lemmy_beta/c/main is cached on lemmy_alpha
      // Use short-hand search url
      let searchUrl = `${lemmyAlphaApiUrl}/search?q=!main@lemmy_beta:8550&type_=All&sort=TopAll`;

      let searchResponse: SearchResponse = await fetch(searchUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(searchResponse.communities[0].name).toBe('main');

      let followForm: FollowCommunityForm = {
        community_id: searchResponse.communities[0].id,
        follow: true,
        auth: lemmyAlphaAuth,
      };

      let followRes: CommunityResponse = await fetch(
        `${lemmyAlphaApiUrl}/community/follow`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(followForm),
        }
      ).then(d => d.json());

      // Make sure the follow response went through
      expect(followRes.community.local).toBe(false);
      expect(followRes.community.name).toBe('main');

      // Check that you are subscribed to it locally
      let followedCommunitiesUrl = `${lemmyAlphaApiUrl}/user/followed_communities?&auth=${lemmyAlphaAuth}`;
      let followedCommunitiesRes: GetFollowedCommunitiesResponse = await fetch(
        followedCommunitiesUrl,
        {
          method: 'GET',
        }
      ).then(d => d.json());

      expect(followedCommunitiesRes.communities[1].community_local).toBe(false);
    });
  });

  describe('create test post', () => {
    test('/u/lemmy_alpha creates a post on /c/lemmy_beta/main, its on both instances', async () => {
      let name = 'A jest test federated post';
      let postForm: PostForm = {
        name,
        auth: lemmyAlphaAuth,
        community_id: 3,
        creator_id: 2,
        nsfw: false,
      };

      let createResponse: PostResponse = await fetch(
        `${lemmyAlphaApiUrl}/post`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(postForm),
        }
      ).then(d => d.json());

      expect(createResponse.post.name).toBe(name);
      expect(createResponse.post.community_local).toBe(false);
      expect(createResponse.post.creator_local).toBe(true);
      expect(createResponse.post.score).toBe(1);

      let getPostUrl = `${lemmyBetaApiUrl}/post?id=2`;
      let getPostRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(getPostRes.post.name).toBe(name);
      expect(getPostRes.post.community_local).toBe(true);
      expect(getPostRes.post.creator_local).toBe(false);
      expect(getPostRes.post.score).toBe(1);
    });
  });

  describe('update test post', () => {
    test('/u/lemmy_alpha updates a post on /c/lemmy_beta/main, the update is on both', async () => {
      let name = 'A jest test federated post, updated';
      let postForm: PostForm = {
        name,
        edit_id: 2,
        auth: lemmyAlphaAuth,
        community_id: 3,
        creator_id: 2,
        nsfw: false,
      };

      let updateResponse: PostResponse = await fetch(
        `${lemmyAlphaApiUrl}/post`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(postForm),
        }
      ).then(d => d.json());

      expect(updateResponse.post.name).toBe(name);
      expect(updateResponse.post.community_local).toBe(false);
      expect(updateResponse.post.creator_local).toBe(true);

      let getPostUrl = `${lemmyBetaApiUrl}/post?id=2`;
      let getPostRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(getPostRes.post.name).toBe(name);
      expect(getPostRes.post.community_local).toBe(true);
      expect(getPostRes.post.creator_local).toBe(false);
    });
  });

  describe('create test comment', () => {
    test('/u/lemmy_alpha creates a comment on /c/lemmy_beta/main, its on both instances', async () => {
      let content = 'A jest test federated comment';
      let commentForm: CommentForm = {
        content,
        post_id: 2,
        auth: lemmyAlphaAuth,
      };

      let createResponse: CommentResponse = await fetch(
        `${lemmyAlphaApiUrl}/comment`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(commentForm),
        }
      ).then(d => d.json());

      expect(createResponse.comment.content).toBe(content);
      expect(createResponse.comment.community_local).toBe(false);
      expect(createResponse.comment.creator_local).toBe(true);
      expect(createResponse.comment.score).toBe(1);

      let getPostUrl = `${lemmyBetaApiUrl}/post?id=2`;
      let getPostRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(getPostRes.comments[0].content).toBe(content);
      expect(getPostRes.comments[0].community_local).toBe(true);
      expect(getPostRes.comments[0].creator_local).toBe(false);
      expect(getPostRes.comments[0].score).toBe(1);

      // Now do beta replying to that comment, as a child comment
      let contentBeta = 'A child federated comment from beta';
      let commentFormBeta: CommentForm = {
        content: contentBeta,
        post_id: getPostRes.post.id,
        parent_id: getPostRes.comments[0].id,
        auth: lemmyBetaAuth,
      };

      let createResponseBeta: CommentResponse = await fetch(
        `${lemmyBetaApiUrl}/comment`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(commentFormBeta),
        }
      ).then(d => d.json());

      expect(createResponseBeta.comment.content).toBe(contentBeta);
      expect(createResponseBeta.comment.community_local).toBe(true);
      expect(createResponseBeta.comment.creator_local).toBe(true);
      expect(createResponseBeta.comment.parent_id).toBe(1);
      expect(createResponseBeta.comment.score).toBe(1);

      // Make sure lemmy alpha sees that new child comment from beta
      let getPostUrlAlpha = `${lemmyAlphaApiUrl}/post?id=2`;
      let getPostResAlpha: GetPostResponse = await fetch(getPostUrlAlpha, {
        method: 'GET',
      }).then(d => d.json());

      // The newest show up first
      expect(getPostResAlpha.comments[0].content).toBe(contentBeta);
      expect(getPostResAlpha.comments[0].community_local).toBe(false);
      expect(getPostResAlpha.comments[0].creator_local).toBe(false);
      expect(getPostResAlpha.comments[0].score).toBe(1);
    });
  });

  describe('update test comment', () => {
    test('/u/lemmy_alpha updates a comment on /c/lemmy_beta/main, its on both instances', async () => {
      let content = 'A jest test federated comment update';
      let commentForm: CommentForm = {
        content,
        post_id: 2,
        edit_id: 1,
        auth: lemmyAlphaAuth,
        creator_id: 2,
      };

      let updateResponse: CommentResponse = await fetch(
        `${lemmyAlphaApiUrl}/comment`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(commentForm),
        }
      ).then(d => d.json());

      expect(updateResponse.comment.content).toBe(content);
      expect(updateResponse.comment.community_local).toBe(false);
      expect(updateResponse.comment.creator_local).toBe(true);

      let getPostUrl = `${lemmyBetaApiUrl}/post?id=2`;
      let getPostRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(getPostRes.comments[1].content).toBe(content);
      expect(getPostRes.comments[1].community_local).toBe(true);
      expect(getPostRes.comments[1].creator_local).toBe(false);
    });
  });

  describe('delete things', () => {
    test('/u/lemmy_beta deletes and undeletes a federated comment, post, and community, lemmy_alpha sees its deleted.', async () => {
      // Create a test community
      let communityName = 'test_community';
      let communityForm: CommunityForm = {
        name: communityName,
        title: communityName,
        category_id: 1,
        nsfw: false,
        auth: lemmyBetaAuth,
      };

      let createCommunityRes: CommunityResponse = await fetch(
        `${lemmyBetaApiUrl}/community`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(communityForm),
        }
      ).then(d => d.json());

      expect(createCommunityRes.community.name).toBe(communityName);

      // Cache it on lemmy_alpha
      let searchUrl = `${lemmyAlphaApiUrl}/search?q=http://lemmy_beta:8550/c/${communityName}&type_=All&sort=TopAll`;
      let searchResponse: SearchResponse = await fetch(searchUrl, {
        method: 'GET',
      }).then(d => d.json());

      let communityOnAlphaId = searchResponse.communities[0].id;

      // Follow it
      let followForm: FollowCommunityForm = {
        community_id: communityOnAlphaId,
        follow: true,
        auth: lemmyAlphaAuth,
      };

      let followRes: CommunityResponse = await fetch(
        `${lemmyAlphaApiUrl}/community/follow`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(followForm),
        }
      ).then(d => d.json());

      // Make sure the follow response went through
      expect(followRes.community.local).toBe(false);
      expect(followRes.community.name).toBe(communityName);

      // Lemmy beta creates a test post
      let postName = 'A jest test post with delete';
      let createPostForm: PostForm = {
        name: postName,
        auth: lemmyBetaAuth,
        community_id: createCommunityRes.community.id,
        creator_id: 2,
        nsfw: false,
      };

      let createPostRes: PostResponse = await fetch(`${lemmyBetaApiUrl}/post`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: wrapper(createPostForm),
      }).then(d => d.json());
      expect(createPostRes.post.name).toBe(postName);

      // Lemmy beta creates a test comment
      let commentContent = 'A jest test federated comment with delete';
      let createCommentForm: CommentForm = {
        content: commentContent,
        post_id: createPostRes.post.id,
        auth: lemmyBetaAuth,
      };

      let createCommentRes: CommentResponse = await fetch(
        `${lemmyBetaApiUrl}/comment`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(createCommentForm),
        }
      ).then(d => d.json());

      expect(createCommentRes.comment.content).toBe(commentContent);

      // lemmy_beta deletes the comment
      let deleteCommentForm: CommentForm = {
        content: commentContent,
        edit_id: createCommentRes.comment.id,
        post_id: createPostRes.post.id,
        deleted: true,
        auth: lemmyBetaAuth,
        creator_id: createCommentRes.comment.creator_id,
      };

      let deleteCommentRes: CommentResponse = await fetch(
        `${lemmyBetaApiUrl}/comment`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(deleteCommentForm),
        }
      ).then(d => d.json());
      expect(deleteCommentRes.comment.deleted).toBe(true);

      // lemmy_alpha sees that the comment is deleted
      let getPostUrl = `${lemmyAlphaApiUrl}/post?id=3`;
      let getPostRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());
      expect(getPostRes.comments[0].deleted).toBe(true);

      // lemmy_beta undeletes the comment
      let undeleteCommentForm: CommentForm = {
        content: commentContent,
        edit_id: createCommentRes.comment.id,
        post_id: createPostRes.post.id,
        deleted: false,
        auth: lemmyBetaAuth,
        creator_id: createCommentRes.comment.creator_id,
      };

      let undeleteCommentRes: CommentResponse = await fetch(
        `${lemmyBetaApiUrl}/comment`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(undeleteCommentForm),
        }
      ).then(d => d.json());
      expect(undeleteCommentRes.comment.deleted).toBe(false);

      // lemmy_alpha sees that the comment is undeleted
      let getPostUndeleteRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());
      expect(getPostUndeleteRes.comments[0].deleted).toBe(false);

      // lemmy_beta deletes the post
      let deletePostForm: PostForm = {
        name: postName,
        edit_id: createPostRes.post.id,
        auth: lemmyBetaAuth,
        community_id: createPostRes.post.community_id,
        creator_id: createPostRes.post.creator_id,
        nsfw: false,
        deleted: true,
      };

      let deletePostRes: PostResponse = await fetch(`${lemmyBetaApiUrl}/post`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: wrapper(deletePostForm),
      }).then(d => d.json());
      expect(deletePostRes.post.deleted).toBe(true);

      // Make sure lemmy_alpha sees the post is deleted
      let getPostResAgain: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());
      expect(getPostResAgain.post.deleted).toBe(true);

      // lemmy_beta undeletes the post
      let undeletePostForm: PostForm = {
        name: postName,
        edit_id: createPostRes.post.id,
        auth: lemmyBetaAuth,
        community_id: createPostRes.post.community_id,
        creator_id: createPostRes.post.creator_id,
        nsfw: false,
        deleted: false,
      };

      let undeletePostRes: PostResponse = await fetch(
        `${lemmyBetaApiUrl}/post`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(undeletePostForm),
        }
      ).then(d => d.json());
      expect(undeletePostRes.post.deleted).toBe(false);

      // Make sure lemmy_alpha sees the post is undeleted
      let getPostResAgainTwo: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());
      expect(getPostResAgainTwo.post.deleted).toBe(false);

      // lemmy_beta deletes the community
      let deleteCommunityForm: CommunityForm = {
        name: communityName,
        title: communityName,
        category_id: 1,
        edit_id: createCommunityRes.community.id,
        nsfw: false,
        deleted: true,
        auth: lemmyBetaAuth,
      };

      let deleteResponse: CommunityResponse = await fetch(
        `${lemmyBetaApiUrl}/community`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(deleteCommunityForm),
        }
      ).then(d => d.json());

      // Make sure the delete went through
      expect(deleteResponse.community.deleted).toBe(true);

      // Re-get it from alpha, make sure its deleted there too
      let getCommunityUrl = `${lemmyAlphaApiUrl}/community?id=${communityOnAlphaId}&auth=${lemmyAlphaAuth}`;
      let getCommunityRes: GetCommunityResponse = await fetch(getCommunityUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(getCommunityRes.community.deleted).toBe(true);

      // lemmy_beta undeletes the community
      let undeleteCommunityForm: CommunityForm = {
        name: communityName,
        title: communityName,
        category_id: 1,
        edit_id: createCommunityRes.community.id,
        nsfw: false,
        deleted: false,
        auth: lemmyBetaAuth,
      };

      let undeleteCommunityRes: CommunityResponse = await fetch(
        `${lemmyBetaApiUrl}/community`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(undeleteCommunityForm),
        }
      ).then(d => d.json());

      // Make sure the delete went through
      expect(undeleteCommunityRes.community.deleted).toBe(false);

      // Re-get it from alpha, make sure its deleted there too
      let getCommunityResAgain: GetCommunityResponse = await fetch(
        getCommunityUrl,
        {
          method: 'GET',
        }
      ).then(d => d.json());
      expect(getCommunityResAgain.community.deleted).toBe(false);
    });
  });
});

function wrapper(form: any): string {
  return JSON.stringify(form);
}
