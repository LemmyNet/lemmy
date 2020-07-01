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
  CommentLikeForm,
  CreatePostLikeForm,
  PrivateMessageForm,
  EditPrivateMessageForm,
  PrivateMessageResponse,
  PrivateMessagesResponse,
  GetUserMentionsResponse,
} from '../interfaces';

let lemmyAlphaUrl = 'http://localhost:8540';
let lemmyAlphaApiUrl = `${lemmyAlphaUrl}/api/v1`;
let lemmyAlphaAuth: string;

let lemmyBetaUrl = 'http://localhost:8550';
let lemmyBetaApiUrl = `${lemmyBetaUrl}/api/v1`;
let lemmyBetaAuth: string;

let lemmyGammaUrl = 'http://localhost:8560';
let lemmyGammaApiUrl = `${lemmyGammaUrl}/api/v1`;
let lemmyGammaAuth: string;

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

    console.log('Logging in as lemmy_gamma');
    let formC = {
      username_or_email: 'lemmy_gamma',
      password: 'lemmy',
    };

    let resG: LoginResponse = await fetch(`${lemmyGammaApiUrl}/user/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(formC),
    }).then(d => d.json());

    lemmyGammaAuth = resG.jwt;
  });

  describe('post_search', () => {
    test('Create test post on alpha and fetch it on beta', async () => {
      let name = 'A jest test post';
      let postForm: PostForm = {
        name,
        auth: lemmyAlphaAuth,
        community_id: 2,
        creator_id: 2,
        nsfw: false,
      };

      let createPostRes: PostResponse = await fetch(
        `${lemmyAlphaApiUrl}/post`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(postForm),
        }
      ).then(d => d.json());
      expect(createPostRes.post.name).toBe(name);

      let searchUrl = `${lemmyBetaApiUrl}/search?q=${createPostRes.post.ap_id}&type_=All&sort=TopAll`;
      let searchResponse: SearchResponse = await fetch(searchUrl, {
        method: 'GET',
      }).then(d => d.json());

      // TODO: check more fields
      expect(searchResponse.posts[0].name).toBe(name);
    });
  });

  describe('follow_accept', () => {
    test('/u/lemmy_alpha follows and accepts lemmy-beta/c/main', async () => {
      // Make sure lemmy-beta/c/main is cached on lemmy_alpha
      // Use short-hand search url
      let searchUrl = `${lemmyAlphaApiUrl}/search?q=!main@lemmy-beta:8550&type_=All&sort=TopAll`;

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

      // Test out unfollowing
      let unfollowForm: FollowCommunityForm = {
        community_id: searchResponse.communities[0].id,
        follow: false,
        auth: lemmyAlphaAuth,
      };

      let unfollowRes: CommunityResponse = await fetch(
        `${lemmyAlphaApiUrl}/community/follow`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(unfollowForm),
        }
      ).then(d => d.json());
      expect(unfollowRes.community.local).toBe(false);

      // Check that you are unsubscribed to it locally
      let followedCommunitiesResAgain: GetFollowedCommunitiesResponse = await fetch(
        followedCommunitiesUrl,
        {
          method: 'GET',
        }
      ).then(d => d.json());

      expect(followedCommunitiesResAgain.communities.length).toBe(1);

      // Follow again, for other tests
      let followResAgain: CommunityResponse = await fetch(
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
      expect(followResAgain.community.local).toBe(false);
      expect(followResAgain.community.name).toBe('main');

      // Also make G follow B

      // Use short-hand search url
      let searchUrlG = `${lemmyGammaApiUrl}/search?q=!main@lemmy-beta:8550&type_=All&sort=TopAll`;

      let searchResponseG: SearchResponse = await fetch(searchUrlG, {
        method: 'GET',
      }).then(d => d.json());

      expect(searchResponseG.communities[0].name).toBe('main');

      let followFormG: FollowCommunityForm = {
        community_id: searchResponseG.communities[0].id,
        follow: true,
        auth: lemmyGammaAuth,
      };

      let followResG: CommunityResponse = await fetch(
        `${lemmyGammaApiUrl}/community/follow`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(followFormG),
        }
      ).then(d => d.json());

      // Make sure the follow response went through
      expect(followResG.community.local).toBe(false);
      expect(followResG.community.name).toBe('main');

      // Check that you are subscribed to it locally
      let followedCommunitiesUrlG = `${lemmyGammaApiUrl}/user/followed_communities?&auth=${lemmyGammaAuth}`;
      let followedCommunitiesResG: GetFollowedCommunitiesResponse = await fetch(
        followedCommunitiesUrlG,
        {
          method: 'GET',
        }
      ).then(d => d.json());

      expect(followedCommunitiesResG.communities[1].community_local).toBe(
        false
      );
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

      let unlikePostForm: CreatePostLikeForm = {
        post_id: createResponse.post.id,
        score: 0,
        auth: lemmyAlphaAuth,
      };
      expect(createResponse.post.name).toBe(name);
      expect(createResponse.post.community_local).toBe(false);
      expect(createResponse.post.creator_local).toBe(true);
      expect(createResponse.post.score).toBe(1);

      let unlikePostRes: PostResponse = await fetch(
        `${lemmyAlphaApiUrl}/post/like`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(unlikePostForm),
        }
      ).then(d => d.json());
      expect(unlikePostRes.post.score).toBe(0);

      let getPostUrl = `${lemmyBetaApiUrl}/post?id=2`;
      let getPostRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(getPostRes.post.name).toBe(name);
      expect(getPostRes.post.community_local).toBe(true);
      expect(getPostRes.post.creator_local).toBe(false);
      expect(getPostRes.post.score).toBe(0);
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

      // Do an unlike, to test it
      let unlikeCommentForm: CommentLikeForm = {
        comment_id: createResponse.comment.id,
        score: 0,
        post_id: 2,
        auth: lemmyAlphaAuth,
      };

      let unlikeCommentRes: CommentResponse = await fetch(
        `${lemmyAlphaApiUrl}/comment/like`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(unlikeCommentForm),
        }
      ).then(d => d.json());

      expect(unlikeCommentRes.comment.score).toBe(0);

      let getPostUrl = `${lemmyBetaApiUrl}/post?id=2`;
      let getPostRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(getPostRes.comments[0].content).toBe(content);
      expect(getPostRes.comments[0].community_local).toBe(true);
      expect(getPostRes.comments[0].creator_local).toBe(false);
      expect(getPostRes.comments[0].score).toBe(0);

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

      // Lemmy alpha responds to their own comment, but mentions lemmy beta.
      // Make sure lemmy beta gets that in their inbox.
      let mentionContent = 'A test mention of @lemmy_beta@lemmy-beta:8550';
      let mentionCommentForm: CommentForm = {
        content: mentionContent,
        post_id: 2,
        parent_id: createResponse.comment.id,
        auth: lemmyAlphaAuth,
      };

      let createMentionRes: CommentResponse = await fetch(
        `${lemmyAlphaApiUrl}/comment`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(mentionCommentForm),
        }
      ).then(d => d.json());

      expect(createMentionRes.comment.content).toBe(mentionContent);
      expect(createMentionRes.comment.community_local).toBe(false);
      expect(createMentionRes.comment.creator_local).toBe(true);
      expect(createMentionRes.comment.score).toBe(1);

      // Make sure lemmy beta sees that new mention
      let getMentionUrl = `${lemmyBetaApiUrl}/user/mention?sort=New&unread_only=false&auth=${lemmyBetaAuth}`;
      let getMentionsRes: GetUserMentionsResponse = await fetch(getMentionUrl, {
        method: 'GET',
      }).then(d => d.json());

      // The newest show up first
      expect(getMentionsRes.mentions[0].content).toBe(mentionContent);
      expect(getMentionsRes.mentions[0].community_local).toBe(true);
      expect(getMentionsRes.mentions[0].creator_local).toBe(false);
      expect(getMentionsRes.mentions[0].score).toBe(1);
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

      expect(getPostRes.comments[2].content).toBe(content);
      expect(getPostRes.comments[2].community_local).toBe(true);
      expect(getPostRes.comments[2].creator_local).toBe(false);
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
      let searchUrl = `${lemmyAlphaApiUrl}/search?q=http://lemmy-beta:8550/c/${communityName}&type_=All&sort=TopAll`;
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

  describe('remove things', () => {
    test('/u/lemmy_beta removes and unremoves a federated comment, post, and community, lemmy_alpha sees its removed.', async () => {
      // Create a test community
      let communityName = 'test_community_rem';
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
      let searchUrl = `${lemmyAlphaApiUrl}/search?q=http://lemmy-beta:8550/c/${communityName}&type_=All&sort=TopAll`;
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
      let postName = 'A jest test post with remove';
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
      let commentContent = 'A jest test federated comment with remove';
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

      // lemmy_beta removes the comment
      let removeCommentForm: CommentForm = {
        content: commentContent,
        edit_id: createCommentRes.comment.id,
        post_id: createPostRes.post.id,
        removed: true,
        auth: lemmyBetaAuth,
        creator_id: createCommentRes.comment.creator_id,
      };

      let removeCommentRes: CommentResponse = await fetch(
        `${lemmyBetaApiUrl}/comment`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(removeCommentForm),
        }
      ).then(d => d.json());
      expect(removeCommentRes.comment.removed).toBe(true);

      // lemmy_alpha sees that the comment is removed
      let getPostUrl = `${lemmyAlphaApiUrl}/post?id=4`;
      let getPostRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());
      expect(getPostRes.comments[0].removed).toBe(true);

      // lemmy_beta undeletes the comment
      let unremoveCommentForm: CommentForm = {
        content: commentContent,
        edit_id: createCommentRes.comment.id,
        post_id: createPostRes.post.id,
        removed: false,
        auth: lemmyBetaAuth,
        creator_id: createCommentRes.comment.creator_id,
      };

      let unremoveCommentRes: CommentResponse = await fetch(
        `${lemmyBetaApiUrl}/comment`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(unremoveCommentForm),
        }
      ).then(d => d.json());
      expect(unremoveCommentRes.comment.removed).toBe(false);

      // lemmy_alpha sees that the comment is undeleted
      let getPostUnremoveRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());
      expect(getPostUnremoveRes.comments[0].removed).toBe(false);

      // lemmy_beta deletes the post
      let removePostForm: PostForm = {
        name: postName,
        edit_id: createPostRes.post.id,
        auth: lemmyBetaAuth,
        community_id: createPostRes.post.community_id,
        creator_id: createPostRes.post.creator_id,
        nsfw: false,
        removed: true,
      };

      let removePostRes: PostResponse = await fetch(`${lemmyBetaApiUrl}/post`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: wrapper(removePostForm),
      }).then(d => d.json());
      expect(removePostRes.post.removed).toBe(true);

      // Make sure lemmy_alpha sees the post is deleted
      let getPostResAgain: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());
      expect(getPostResAgain.post.removed).toBe(true);

      // lemmy_beta unremoves the post
      let unremovePostForm: PostForm = {
        name: postName,
        edit_id: createPostRes.post.id,
        auth: lemmyBetaAuth,
        community_id: createPostRes.post.community_id,
        creator_id: createPostRes.post.creator_id,
        nsfw: false,
        removed: false,
      };

      let unremovePostRes: PostResponse = await fetch(
        `${lemmyBetaApiUrl}/post`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(unremovePostForm),
        }
      ).then(d => d.json());
      expect(unremovePostRes.post.removed).toBe(false);

      // Make sure lemmy_alpha sees the post is unremoved
      let getPostResAgainTwo: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());
      expect(getPostResAgainTwo.post.removed).toBe(false);

      // lemmy_beta deletes the community
      let removeCommunityForm: CommunityForm = {
        name: communityName,
        title: communityName,
        category_id: 1,
        edit_id: createCommunityRes.community.id,
        nsfw: false,
        removed: true,
        auth: lemmyBetaAuth,
      };

      let removeCommunityRes: CommunityResponse = await fetch(
        `${lemmyBetaApiUrl}/community`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(removeCommunityForm),
        }
      ).then(d => d.json());

      // Make sure the delete went through
      expect(removeCommunityRes.community.removed).toBe(true);

      // Re-get it from alpha, make sure its removed there too
      let getCommunityUrl = `${lemmyAlphaApiUrl}/community?id=${communityOnAlphaId}&auth=${lemmyAlphaAuth}`;
      let getCommunityRes: GetCommunityResponse = await fetch(getCommunityUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(getCommunityRes.community.removed).toBe(true);

      // lemmy_beta unremoves the community
      let unremoveCommunityForm: CommunityForm = {
        name: communityName,
        title: communityName,
        category_id: 1,
        edit_id: createCommunityRes.community.id,
        nsfw: false,
        removed: false,
        auth: lemmyBetaAuth,
      };

      let unremoveCommunityRes: CommunityResponse = await fetch(
        `${lemmyBetaApiUrl}/community`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(unremoveCommunityForm),
        }
      ).then(d => d.json());

      // Make sure the delete went through
      expect(unremoveCommunityRes.community.removed).toBe(false);

      // Re-get it from alpha, make sure its deleted there too
      let getCommunityResAgain: GetCommunityResponse = await fetch(
        getCommunityUrl,
        {
          method: 'GET',
        }
      ).then(d => d.json());
      expect(getCommunityResAgain.community.removed).toBe(false);
    });
  });

  describe('private message', () => {
    test('/u/lemmy_alpha creates/updates/deletes/undeletes a private_message to /u/lemmy_beta, its on both instances', async () => {
      let content = 'A jest test federated private message';
      let privateMessageForm: PrivateMessageForm = {
        content,
        recipient_id: 3,
        auth: lemmyAlphaAuth,
      };

      let createRes: PrivateMessageResponse = await fetch(
        `${lemmyAlphaApiUrl}/private_message`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(privateMessageForm),
        }
      ).then(d => d.json());
      expect(createRes.message.content).toBe(content);
      expect(createRes.message.local).toBe(true);
      expect(createRes.message.creator_local).toBe(true);
      expect(createRes.message.recipient_local).toBe(false);

      // Get it from beta
      let getPrivateMessagesUrl = `${lemmyBetaApiUrl}/private_message/list?auth=${lemmyBetaAuth}&unread_only=false`;

      let getPrivateMessagesRes: PrivateMessagesResponse = await fetch(
        getPrivateMessagesUrl,
        {
          method: 'GET',
        }
      ).then(d => d.json());

      expect(getPrivateMessagesRes.messages[0].content).toBe(content);
      expect(getPrivateMessagesRes.messages[0].local).toBe(false);
      expect(getPrivateMessagesRes.messages[0].creator_local).toBe(false);
      expect(getPrivateMessagesRes.messages[0].recipient_local).toBe(true);

      // lemmy alpha updates the private message
      let updatedContent = 'A jest test federated private message edited';
      let updatePrivateMessageForm: EditPrivateMessageForm = {
        content: updatedContent,
        edit_id: createRes.message.id,
        auth: lemmyAlphaAuth,
      };

      let updateRes: PrivateMessageResponse = await fetch(
        `${lemmyAlphaApiUrl}/private_message`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(updatePrivateMessageForm),
        }
      ).then(d => d.json());

      expect(updateRes.message.content).toBe(updatedContent);

      // Fetch from beta again
      let getPrivateMessagesUpdatedRes: PrivateMessagesResponse = await fetch(
        getPrivateMessagesUrl,
        {
          method: 'GET',
        }
      ).then(d => d.json());

      expect(getPrivateMessagesUpdatedRes.messages[0].content).toBe(
        updatedContent
      );

      // lemmy alpha deletes the private message
      let deletePrivateMessageForm: EditPrivateMessageForm = {
        deleted: true,
        edit_id: createRes.message.id,
        auth: lemmyAlphaAuth,
      };

      let deleteRes: PrivateMessageResponse = await fetch(
        `${lemmyAlphaApiUrl}/private_message`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(deletePrivateMessageForm),
        }
      ).then(d => d.json());

      expect(deleteRes.message.deleted).toBe(true);

      // Fetch from beta again
      let getPrivateMessagesDeletedRes: PrivateMessagesResponse = await fetch(
        getPrivateMessagesUrl,
        {
          method: 'GET',
        }
      ).then(d => d.json());

      // The GetPrivateMessages filters out deleted,
      // even though they are in the actual database.
      // no reason to show them
      expect(getPrivateMessagesDeletedRes.messages.length).toBe(0);

      // lemmy alpha undeletes the private message
      let undeletePrivateMessageForm: EditPrivateMessageForm = {
        deleted: false,
        edit_id: createRes.message.id,
        auth: lemmyAlphaAuth,
      };

      let undeleteRes: PrivateMessageResponse = await fetch(
        `${lemmyAlphaApiUrl}/private_message`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(undeletePrivateMessageForm),
        }
      ).then(d => d.json());

      expect(undeleteRes.message.deleted).toBe(false);

      // Fetch from beta again
      let getPrivateMessagesUnDeletedRes: PrivateMessagesResponse = await fetch(
        getPrivateMessagesUrl,
        {
          method: 'GET',
        }
      ).then(d => d.json());

      expect(getPrivateMessagesUnDeletedRes.messages[0].deleted).toBe(false);
    });
  });

  describe('comment_search', () => {
    test('Create comment on alpha and search it', async () => {
      let content = 'A jest test federated comment for search';
      let commentForm: CommentForm = {
        content,
        post_id: 1,
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

      let searchUrl = `${lemmyBetaApiUrl}/search?q=${createResponse.comment.ap_id}&type_=All&sort=TopAll`;
      let searchResponse: SearchResponse = await fetch(searchUrl, {
        method: 'GET',
      }).then(d => d.json());

      // TODO: check more fields
      expect(searchResponse.comments[0].content).toBe(content);
    });
  });

  describe('announce', () => {
    test('A and G subscribe to B (center) A does action, it gets announced to G', async () => {
      // A and G are already subscribed to B earlier.
      //
      let postName = 'A jest test post for announce';
      let createPostForm: PostForm = {
        name: postName,
        auth: lemmyAlphaAuth,
        community_id: 2,
        creator_id: 2,
        nsfw: false,
      };

      let createPostRes: PostResponse = await fetch(
        `${lemmyAlphaApiUrl}/post`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(createPostForm),
        }
      ).then(d => d.json());
      expect(createPostRes.post.name).toBe(postName);

      // Make sure that post got announced to Gamma
      let searchUrl = `${lemmyGammaApiUrl}/search?q=${createPostRes.post.ap_id}&type_=All&sort=TopAll`;
      let searchResponse: SearchResponse = await fetch(searchUrl, {
        method: 'GET',
      }).then(d => d.json());
      let postId = searchResponse.posts[0].id;
      expect(searchResponse.posts[0].name).toBe(postName);

      // Create a test comment on Gamma, make sure it gets announced to alpha
      let commentContent =
        'A jest test federated comment announce, lets mention @lemmy_beta@lemmy-beta:8550';

      let commentForm: CommentForm = {
        content: commentContent,
        post_id: postId,
        auth: lemmyGammaAuth,
      };

      let createCommentRes: CommentResponse = await fetch(
        `${lemmyGammaApiUrl}/comment`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(commentForm),
        }
      ).then(d => d.json());

      expect(createCommentRes.comment.content).toBe(commentContent);
      expect(createCommentRes.comment.community_local).toBe(false);
      expect(createCommentRes.comment.creator_local).toBe(true);
      expect(createCommentRes.comment.score).toBe(1);

      // Get the post from alpha, make sure it has gamma's comment
      let getPostUrl = `${lemmyAlphaApiUrl}/post?id=5`;
      let getPostRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(getPostRes.comments[0].content).toBe(commentContent);
      expect(getPostRes.comments[0].community_local).toBe(true);
      expect(getPostRes.comments[0].creator_local).toBe(false);
      expect(getPostRes.comments[0].score).toBe(1);
    });
  });

  describe('fetch inreplytos', () => {
    test('A is unsubbed from B, B makes a post, and some embedded comments, A subs to B, B updates the lowest level comment, A fetches both the post and all the inreplyto comments for that post.', async () => {
      // Check that A is subscribed to B
      let followedCommunitiesUrl = `${lemmyAlphaApiUrl}/user/followed_communities?&auth=${lemmyAlphaAuth}`;
      let followedCommunitiesRes: GetFollowedCommunitiesResponse = await fetch(
        followedCommunitiesUrl,
        {
          method: 'GET',
        }
      ).then(d => d.json());
      expect(followedCommunitiesRes.communities[1].community_local).toBe(false);

      // A unsubs from B (communities ids 3-5)
      for (let i = 3; i <= 5; i++) {
        let unfollowForm: FollowCommunityForm = {
          community_id: i,
          follow: false,
          auth: lemmyAlphaAuth,
        };

        let unfollowRes: CommunityResponse = await fetch(
          `${lemmyAlphaApiUrl}/community/follow`,
          {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
            },
            body: wrapper(unfollowForm),
          }
        ).then(d => d.json());
        expect(unfollowRes.community.local).toBe(false);
      }

      // Check that you are unsubscribed from all of them locally
      let followedCommunitiesResAgain: GetFollowedCommunitiesResponse = await fetch(
        followedCommunitiesUrl,
        {
          method: 'GET',
        }
      ).then(d => d.json());
      expect(followedCommunitiesResAgain.communities.length).toBe(1);

      // B creates a post, and two comments, should be invisible to A
      let betaPostName = 'Test post on B, invisible to A at first';
      let postForm: PostForm = {
        name: betaPostName,
        auth: lemmyBetaAuth,
        community_id: 2,
        creator_id: 2,
        nsfw: false,
      };

      let createPostRes: PostResponse = await fetch(`${lemmyBetaApiUrl}/post`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: wrapper(postForm),
      }).then(d => d.json());
      expect(createPostRes.post.name).toBe(betaPostName);

      // B creates a comment, then a child one of that.
      let parentCommentContent = 'An invisible top level comment from beta';
      let createParentCommentForm: CommentForm = {
        content: parentCommentContent,
        post_id: createPostRes.post.id,
        auth: lemmyBetaAuth,
      };

      let createParentCommentRes: CommentResponse = await fetch(
        `${lemmyBetaApiUrl}/comment`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(createParentCommentForm),
        }
      ).then(d => d.json());
      expect(createParentCommentRes.comment.content).toBe(parentCommentContent);

      let childCommentContent = 'An invisible child comment from beta';
      let createChildCommentForm: CommentForm = {
        content: childCommentContent,
        parent_id: createParentCommentRes.comment.id,
        post_id: createPostRes.post.id,
        auth: lemmyBetaAuth,
      };

      let createChildCommentRes: CommentResponse = await fetch(
        `${lemmyBetaApiUrl}/comment`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(createChildCommentForm),
        }
      ).then(d => d.json());
      expect(createChildCommentRes.comment.content).toBe(childCommentContent);

      // Follow again, for other tests
      let searchUrl = `${lemmyAlphaApiUrl}/search?q=!main@lemmy-beta:8550&type_=All&sort=TopAll`;

      let searchResponse: SearchResponse = await fetch(searchUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(searchResponse.communities[0].name).toBe('main');

      let followForm: FollowCommunityForm = {
        community_id: searchResponse.communities[0].id,
        follow: true,
        auth: lemmyAlphaAuth,
      };

      let followResAgain: CommunityResponse = await fetch(
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
      expect(followResAgain.community.local).toBe(false);
      expect(followResAgain.community.name).toBe('main');

      let updatedCommentContent = 'An update child comment from beta';
      let updatedCommentForm: CommentForm = {
        content: updatedCommentContent,
        post_id: createPostRes.post.id,
        edit_id: createChildCommentRes.comment.id,
        auth: lemmyBetaAuth,
        creator_id: 2,
      };

      let updateResponse: CommentResponse = await fetch(
        `${lemmyBetaApiUrl}/comment`,
        {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: wrapper(updatedCommentForm),
        }
      ).then(d => d.json());
      expect(updateResponse.comment.content).toBe(updatedCommentContent);

      // Make sure that A picked up the post, parent comment, and child comment
      let getPostUrl = `${lemmyAlphaApiUrl}/post?id=6`;
      let getPostRes: GetPostResponse = await fetch(getPostUrl, {
        method: 'GET',
      }).then(d => d.json());

      expect(getPostRes.post.name).toBe(betaPostName);
      expect(getPostRes.comments[1].content).toBe(parentCommentContent);
      expect(getPostRes.comments[0].content).toBe(updatedCommentContent);
      expect(getPostRes.post.community_local).toBe(false);
      expect(getPostRes.post.creator_local).toBe(false);
    });
  });
});

function wrapper(form: any): string {
  return JSON.stringify(form);
}
