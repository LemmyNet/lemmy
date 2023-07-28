/*
    remote-home-remote Lemmy Instance behavior testing
    Tests are intended to emphasize:
       1. end-user, non-admin, accounts
       2. lemmy-beta instance being the "home instance" of a community
       3. lemmy-alpha and lemmy-gamma being "remote instances" of a community.
       4. lemmy-beta Rust code paths that replicate data between lemmy-beta & lemmy-gamma instances
*/
jest.setTimeout(90 * 1000);

import {
  Community,
  GetCommentsResponse,
  GetCommunity,
  GetCommunityResponse,
  PostView,
} from "lemmy-js-client";
import {
  registerUser,
  alpha,
  gamma,
  API,
  beta,
  createCommunity,
  followCommunity,
  resolveCommunity,
  setupLogins,
  getSite,
  resolvePerson,
  createPost,
  createComment,
  getComments,
  getCommentParentId,
  featurePost,
  lockPost,
  removePost,
  removeComment,
  getPosts,
  banPersonFromCommunity,
} from "./shared";
import { writeHeapSnapshot } from "v8";

beforeAll(async () => {
  await setupLogins();
});

afterAll(async () => {});

let alpha_user_mod: API;
let alpha_user_non_mod: API;
let alpha_user_observer: API;
let beta_user_creator_mod: API;
let gamma_user_mod: API;
let gamma_user_non_mod: API;
let gamma_user_non_subscriber: API;
let betaCommunityHome: Community;
let alphaCommunityRemote: Community;
let gammaCommunityRemote: Community;

// ToDo: is getCommunity in shared library returning wrong object?
export async function getCommunityResponse(
  api: API,
  id: number,
): Promise<GetCommunityResponse> {
  let form: GetCommunity = {
    id,
    auth: api.auth,
  };
  return api.client.getCommunity(form);
}

async function registerUserClient(withapi: API, username: string) {
  let registerUserRes = await registerUser(withapi, username);
  // this client being coppied from the other client, is that odd?
  let newUser: API = {
    client: withapi.client,
    auth: registerUserRes.jwt ?? "",
  };
  return newUser;
}

test("Establish 5 non-admin end-users for all remaining tests", async () => {
  alpha_user_mod = await registerUserClient(alpha, "alpha_powermod0");
  alpha_user_non_mod = await registerUserClient(alpha, "alpha_user0");
  beta_user_creator_mod = await registerUserClient(beta, "beta_creatormod0");
  gamma_user_mod = await registerUserClient(gamma, "gamma_powermod0");
  gamma_user_non_mod = await registerUserClient(gamma, "gamma_user0");
});

test("beta creates new community, becoming moderator", async () => {
  let betaCommunityRes = await createCommunity(
    beta_user_creator_mod,
    "enter_shikari",
  );
  betaCommunityHome = betaCommunityRes.community_view.community;

  let communityFresh = await getCommunityResponse(
    beta_user_creator_mod,
    betaCommunityHome.id,
  );
  expect(communityFresh.moderators.length).toBe(1);
  // Person object handy for self, confirm self is non-admin user
  expect(communityFresh.moderators[0].moderator.admin).toBe(false);
});

test("alpha and gamma instances discover newly created community", async () => {
  let searchShort = `!enter_shikari@lemmy-beta:8551`;

  let alphaCommunityTemp = (
    await resolveCommunity(alpha_user_non_mod, searchShort)
  ).community;
  if (!alphaCommunityTemp) {
    throw "Missing community on alpha";
  }
  alphaCommunityRemote = alphaCommunityTemp.community;

  let gammaCommunityTemp = (
    await resolveCommunity(gamma_user_non_mod, searchShort)
  ).community;
  if (!gammaCommunityTemp) {
    throw "Missing community on gamma";
  }
  gammaCommunityRemote = gammaCommunityTemp.community;
});

test("not joining community, gamma non-mod user creates post and comment, before any subscribers", async () => {
  gamma_user_non_subscriber = await registerUserClient(
    gamma,
    "gamma_nonsubscriber0",
  );
  // without any subscribers on alpha, it won't go to alpha
  let gammaPost0Res = await createPost(
    gamma_user_non_subscriber,
    gammaCommunityRemote.id,
  );
  let gammaComment0Res = await createComment(
    gamma_user_non_subscriber,
    gammaPost0Res.post_view.post.id,
  );

  // at this point, alpha has no subscribers to the community homed on beta
  // replication from beta intance to alpha instance should not happen
  // even sending the outgoing post and comment to beta instance
  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";
  let alphaBeforeSubscribersPosts = await getPosts(
    alpha_user_non_mod,
    communityNameFull,
  );
  expect(alphaBeforeSubscribersPosts.posts.length).toBe(0);
});

test("2 alpha and 2 gamma users subscribe to community", async () => {
  await followCommunity(alpha_user_mod, true, alphaCommunityRemote.id);
  await followCommunity(alpha_user_non_mod, true, alphaCommunityRemote.id);
  await followCommunity(gamma_user_mod, true, gammaCommunityRemote.id);
  await followCommunity(gamma_user_non_mod, true, gammaCommunityRemote.id);
  let freshCommunityHome = await getCommunityResponse(
    beta_user_creator_mod,
    betaCommunityHome.id,
  );
  expect(freshCommunityHome.community_view.counts.subscribers).toBe(5);
});

test("not joining community, gamma non-mod user creates another post and comment, after other subscribers", async () => {
  // other subscribers should cover replication requirements for community to beta, alpha
  let gammaPost0Res = await createPost(
    gamma_user_non_subscriber,
    gammaCommunityRemote.id,
  );
  let gammaComment0Res = await createComment(
    gamma_user_non_subscriber,
    gammaPost0Res.post_view.post.id,
  );

  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";
  let alphaAfterSubscribersPosts = await getPosts(
    alpha_user_non_mod,
    communityNameFull,
  );
  expect(alphaAfterSubscribersPosts.posts.length).toBe(1);
});

test("beta makes 1 gamma and 1 alpha user moderators of community", async () => {
  // the API object doesn't really know who they are, identity crisis
  let site = await getSite(alpha_user_mod);
  if (!site.my_user) {
    throw "Missing site user on alpha";
  }
  let apShortname = `@${site.my_user.local_user_view.person.name}@lemmy-alpha:8541`;
  let alphaPersonOnBeta = (
    await resolvePerson(beta_user_creator_mod, apShortname)
  ).person;
  if (!alphaPersonOnBeta) {
    throw "Missing alpha person on beta instance";
  }
  await beta_user_creator_mod.client.addModToCommunity({
    community_id: betaCommunityHome.id,
    person_id: alphaPersonOnBeta.person.id,
    added: true,
    auth: beta_user_creator_mod.auth,
  });

  // the API object doesn't really know who they are, identity crisis
  let siteGamma = await getSite(gamma_user_mod);
  if (!siteGamma.my_user) {
    throw "Missing site user on gamma";
  }
  let apShortnameGamma = `@${siteGamma.my_user.local_user_view.person.name}@lemmy-gamma:8561`;
  let gammaPersonOnBeta = (
    await resolvePerson(beta_user_creator_mod, apShortnameGamma)
  ).person;
  if (!gammaPersonOnBeta) {
    throw "Missing gamma person on beta instance";
  }
  await beta_user_creator_mod.client.addModToCommunity({
    community_id: betaCommunityHome.id,
    person_id: gammaPersonOnBeta.person.id,
    added: true,
    auth: beta_user_creator_mod.auth,
  });

  // check from alpha remote instance to see moderator additions replicated
  let communityFresh = await getCommunityResponse(
    alpha_user_mod,
    alphaCommunityRemote.id,
  );
  expect(communityFresh.moderators.length).toBe(3);

  // check from gamma remote instance to see moderator additions replicated
  let gammaCommunityFresh = await getCommunityResponse(
    gamma_user_mod,
    gammaCommunityRemote.id,
  );
  expect(gammaCommunityFresh.moderators.length).toBe(3);

  // check moderator usernames
  expect(gammaCommunityFresh.moderators[1].moderator.name).toBe(
    "alpha_powermod0",
  );
  expect(gammaCommunityFresh.moderators[2].moderator.name).toBe(
    "gamma_powermod0",
  );
});

export async function findPostFromListNewGetComments(
  api: API,
  community_name: string,
  post_ap_id: string,
): Promise<GetCommentsResponse> {
  // searach new on the specified community
  let postsResponse = await api.client.getPosts({
    community_name: community_name,
    type_: "All",
    sort: "New",
    limit: 25,
  });

  if (postsResponse.posts) {
    let posts = postsResponse.posts;
    let targetPost;
    for (let i = 0; i < posts.length; i++) {
      if (posts[i].post.ap_id === post_ap_id) {
        targetPost = posts[i];
        break;
      }
    }
    if (targetPost) {
      return await getComments(api, targetPost.post.id);
    }
  } else {
    console.warn(
      "findPostGetComments found no post response on community_name %s searching for post %s",
      community_name,
      post_ap_id,
    );
  }

  throw "getPosts problem";
}

export async function findPostFromListNew(
  api: API,
  community_name: string,
  post_ap_id: string,
): Promise<PostView> {
  // searach new on the specified community
  let postsResponse = await api.client.getPosts({
    community_name: community_name,
    type_: "All",
    sort: "New",
    limit: 25,
  });

  if (postsResponse.posts) {
    let posts = postsResponse.posts;
    let targetPost;
    for (let i = 0; i < posts.length; i++) {
      if (posts[i].post.ap_id === post_ap_id) {
        targetPost = posts[i];
        break;
      }
    }
    if (targetPost) {
      return targetPost;
    }
  } else {
    console.warn(
      "findPostGetComments found no post response on community_name %s searching for post %s",
      community_name,
      post_ap_id,
    );
  }

  throw "getPosts problem";
}

export async function checkForPostFromListNew(
  api: API,
  community_name: string,
  post_ap_id: string,
): Promise<boolean> {
  // searach new on the specified community
  let postsResponse = await api.client.getPosts({
    community_name: community_name,
    type_: "All",
    sort: "New",
    limit: 25,
  });

  if (postsResponse.posts) {
    let posts = postsResponse.posts;
    let targetPost;
    for (let i = 0; i < posts.length; i++) {
      if (posts[i].post.ap_id === post_ap_id) {
        targetPost = posts[i];
        break;
      }
    }
    if (targetPost) {
      return true;
    } else {
      return false;
    }
  } else {
    return false;
  }

  throw "getPosts problem";
}

test("non-moderator user of alpha creates post + comment, non-moderator of gamma validates", async () => {
  // this must traverse from remote-home-remote of community.
  let alphaPost0Res = await createPost(
    alpha_user_non_mod,
    alphaCommunityRemote.id,
  );
  let alphaComment0Res = await createComment(
    alpha_user_non_mod,
    alphaPost0Res.post_view.post.id,
  );

  // Lemmy's API for resolve object is a quick way to get post and comment id, but
  //   it tends to invoke pulling of content.
  //   These tests want to validate routine replicaiton from remote to home to remote...
  //   The only way the content should show up on gamma is if it replicated.
  //   In the interest of avoiding conflicts with other ongoing testing changes
  //      this code uses a local function instead of adding it to shared.ts

  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";
  // ap_id of a post is stable across all instances
  let gammaPostComments = await findPostFromListNewGetComments(
    gamma_user_non_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );
  expect(gammaPostComments.comments.length).toBe(1);

  // check the comment ap_id equality out of band
  expect(alphaComment0Res.comment_view.comment.ap_id).toBe(
    gammaPostComments.comments[0].comment.ap_id,
  );

  // Reply from gamma, extending the branch
  let replyRes = await createComment(
    gamma_user_non_mod,
    gammaPostComments.comments[0].post.id,
    gammaPostComments.comments[0].comment.id,
  );
  expect(replyRes.comment_view.comment.content).toBeDefined();
  expect(getCommentParentId(replyRes.comment_view.comment)).toBe(
    gammaPostComments.comments[0].comment.id,
  );
});

/*
SHORT OF BUG diagnostic, try0
this test was inserted to demonstrate where Lemmy 0.18.2 bug begins and ends.
remote to home replication works, but remote to home to remote does not.
In this case, beta commmunity-features the post instead of alpha.
Think of it is contacting the Community home instance and askikng community creator to help troubleshoot.
*/
test("SHORT OF BUG diagnostic try0: non-moderator user of alpha creates post, moderator+creator of beta community-features, gamma reads it", async () => {
  // this must traverse from remote-home-remote of community.
  let alphaPost0Res = await createPost(
    alpha_user_non_mod,
    alphaCommunityRemote.id,
  );
  let alphaComment0Res = await createComment(
    alpha_user_non_mod,
    alphaPost0Res.post_view.post.id,
  );

  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";

  // SHORT OF BUG diagnostic
  // need to locate post on non-orgin instance to mod action from non-origin instance
  let betaLocatedPost = await findPostFromListNew(
    beta_user_creator_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );

  // moderator AND CREATOR of community on beta features the post in the community
  let modActionResult = await featurePost(
    beta_user_creator_mod,
    true,
    betaLocatedPost.post,
  );
  expect(modActionResult.post_view.post.featured_community).toBe(true);

  // gamma reads it
  // ap_id of a post is stable across all instances
  let gammaPostComments = await findPostFromListNewGetComments(
    gamma_user_non_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );
  expect(gammaPostComments.comments.length).toBe(1);

  // check the comment ap_id equality out of band
  expect(alphaComment0Res.comment_view.comment.ap_id).toBe(
    gammaPostComments.comments[0].comment.ap_id,
  );
  // check the post ap_id equality out of band.
  expect(alphaPost0Res.post_view.post.ap_id).toBe(
    gammaPostComments.comments[0].post.ap_id,
  );

  // FixMe: lemmy_server version 0.18.2 remote-home-remote replication bug here, next test will fail:
  // look at the post, is it community-featured on gamma?
  expect(gammaPostComments.comments[0].post.featured_community).toBe(true);

  // Reply from gamma, creating new reply branch
  let replyRes = await createComment(
    gamma_user_non_mod,
    gammaPostComments.comments[0].post.id,
    gammaPostComments.comments[0].comment.id,
  );
  expect(replyRes.comment_view.comment.content).toBeDefined();
  expect(getCommentParentId(replyRes.comment_view.comment)).toBe(
    gammaPostComments.comments[0].comment.id,
  );
});

/*
SHORT OF BUG diagnostic, try1
this test was inserted to demonstrate where Lemmy 0.18.2 bug begins and ends.
remote to home replication works, but remote to home to remote does not.
In this case, beta reads it instead of gamma.
Think of it is contacting the Community home instance and askikng community creator to help troubleshoot.
*/
test("SHORT OF BUG diagnostic try1: non-moderator user of alpha creates post, moderator of alpha community-features, beta reads it", async () => {
  // this must traverse from remote-home-remote of community.
  let alphaPost0Res = await createPost(
    alpha_user_non_mod,
    alphaCommunityRemote.id,
  );
  let alphaComment0Res = await createComment(
    alpha_user_non_mod,
    alphaPost0Res.post_view.post.id,
  );

  // moderator of community on alpha features the post in the community
  let modActionResult = await featurePost(
    alpha_user_mod,
    true,
    alphaPost0Res.post_view.post,
  );
  expect(modActionResult.post_view.post.featured_community).toBe(true);

  // beta reads it
  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";
  // ap_id of a post is stable across all instances
  let betaPostComments = await findPostFromListNewGetComments(
    beta_user_creator_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );
  expect(betaPostComments.comments.length).toBe(1);

  // check the comment ap_id equality out of band
  expect(alphaComment0Res.comment_view.comment.ap_id).toBe(
    betaPostComments.comments[0].comment.ap_id,
  );
  // check the post ap_id equality out of band.
  expect(alphaPost0Res.post_view.post.ap_id).toBe(
    betaPostComments.comments[0].post.ap_id,
  );

  // FixMe: lemmy_server version 0.18.2 remote-home replication bug here, next test will fail:
  // look at the post, is it community-featured on beta?
  expect(betaPostComments.comments[0].post.featured_community).toBe(true);
});

test("non-moderator user of alpha creates post, moderator of alpha community-features, gamma reads it", async () => {
  // this must traverse from remote-home-remote of community.
  let alphaPost0Res = await createPost(
    alpha_user_non_mod,
    alphaCommunityRemote.id,
  );
  let alphaComment0Res = await createComment(
    alpha_user_non_mod,
    alphaPost0Res.post_view.post.id,
  );

  // moderator of community on alpha features the post in the community
  let modActionResult = await featurePost(
    alpha_user_mod,
    true,
    alphaPost0Res.post_view.post,
  );
  expect(modActionResult.post_view.post.featured_community).toBe(true);

  // gamma reads it
  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";
  // ap_id of a post is stable across all instances
  let gammaPostComments = await findPostFromListNewGetComments(
    gamma_user_non_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );
  expect(gammaPostComments.comments.length).toBe(1);

  // check the comment ap_id equality out of band
  expect(alphaComment0Res.comment_view.comment.ap_id).toBe(
    gammaPostComments.comments[0].comment.ap_id,
  );
  // check the post ap_id equality out of band.
  expect(alphaPost0Res.post_view.post.ap_id).toBe(
    gammaPostComments.comments[0].post.ap_id,
  );

  // FixMe: lemmy_server version 0.18.2 remote-home-remote replication bug here, next test will fail:
  // look at the post, is it community-featured on gamma?
  expect(gammaPostComments.comments[0].post.featured_community).toBe(true);

  // Reply from gamma, creating new reply branch
  let replyRes = await createComment(
    gamma_user_non_mod,
    gammaPostComments.comments[0].post.id,
    gammaPostComments.comments[0].comment.id,
  );
  expect(replyRes.comment_view.comment.content).toBeDefined();
  expect(getCommentParentId(replyRes.comment_view.comment)).toBe(
    gammaPostComments.comments[0].comment.id,
  );
});

test("non-moderator user of alpha creates post, moderator of alpha locks post, gamma reads it", async () => {
  // this must traverse from remote-home-remote of community.
  let alphaPost0Res = await createPost(
    alpha_user_non_mod,
    alphaCommunityRemote.id,
  );
  let alphaComment0Res = await createComment(
    alpha_user_non_mod,
    alphaPost0Res.post_view.post.id,
  );

  // moderator of community on alpha locks the post
  let modActionResult = await lockPost(
    alpha_user_mod,
    true,
    alphaPost0Res.post_view.post,
  );
  expect(modActionResult.post_view.post.locked).toBe(true);

  // gamma reads it
  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";
  // ap_id of a post is stable across all instances
  let gammaPostComments = await findPostFromListNewGetComments(
    gamma_user_non_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );
  expect(gammaPostComments.comments.length).toBe(1);

  // check the comment ap_id equality out of band
  expect(alphaComment0Res.comment_view.comment.ap_id).toBe(
    gammaPostComments.comments[0].comment.ap_id,
  );
  // check the post ap_id equality out of band.
  expect(alphaPost0Res.post_view.post.ap_id).toBe(
    gammaPostComments.comments[0].post.ap_id,
  );

  // look at the post, is it locked on gamma?
  expect(gammaPostComments.comments[0].post.locked).toBe(true);
});

test("non-moderator user of alpha creates post, moderator of alpha removes post, gamma checks for post", async () => {
  // this must traverse from remote-home-remote of community.
  let alphaPost0Res = await createPost(
    alpha_user_non_mod,
    alphaCommunityRemote.id,
  );
  let alphaComment0Res = await createComment(
    alpha_user_non_mod,
    alphaPost0Res.post_view.post.id,
  );

  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";

  // gamma refreshes the community by new to confirm it replicated.
  // ap_id of a post is stable across all instances
  let gammaPostInList0 = await checkForPostFromListNew(
    gamma_user_non_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );
  expect(gammaPostInList0).toBe(true);

  // moderator of community on alpha removes the post
  let modActionResult = await removePost(
    alpha_user_mod,
    true,
    alphaPost0Res.post_view.post,
  );
  expect(modActionResult.post_view.post.removed).toBe(true);

  // gamma refreshes the community by new to confirm remove replicated.
  // as a non-mod end-user on gamma, should not see post in list?
  // ap_id of a post is stable across all instances
  let gammaPostInList1 = await checkForPostFromListNew(
    gamma_user_non_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );
  expect(gammaPostInList1).toBe(false);
});

test("non-moderator user of alpha creates post+comment, moderator of alpha removes comment, gamma checks for comment", async () => {
  // this must traverse from remote-home-remote of community.
  let alphaPost0Res = await createPost(
    alpha_user_non_mod,
    alphaCommunityRemote.id,
  );
  let alphaComment0Res = await createComment(
    alpha_user_non_mod,
    alphaPost0Res.post_view.post.id,
  );

  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";

  // gamma refreshes the community by new to confirm it replicated.
  // ap_id of a post is stable across all instances
  let gammaPostComments0 = await findPostFromListNewGetComments(
    gamma_user_non_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );
  expect(gammaPostComments0.comments.length).toBe(1);

  // check the comment ap_id equality out of band
  expect(alphaComment0Res.comment_view.comment.ap_id).toBe(
    gammaPostComments0.comments[0].comment.ap_id,
  );
  // check the post ap_id equality out of band.
  expect(alphaPost0Res.post_view.post.ap_id).toBe(
    gammaPostComments0.comments[0].post.ap_id,
  );

  // moderator of community on alpha removes the comment
  let modActionResult = await removeComment(
    alpha_user_mod,
    true,
    alphaComment0Res.comment_view.comment.id,
  );
  expect(modActionResult.comment_view.comment.removed).toBe(true);

  // gamma reads it, normal non-mod non-admin end-user should see it?
  // ap_id of a post is stable across all instances
  let gammaPostComments = await findPostFromListNewGetComments(
    gamma_user_non_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );
  expect(gammaPostComments.comments.length).toBe(0);
});

test("alpha moderator creates comment 'speaking as moderator', gamma validates", async () => {
  // this must traverse from remote-home-remote of community.
  let alphaPost0Res = await createPost(
    alpha_user_non_mod,
    alphaCommunityRemote.id,
  );
  // moderator of alpha comments on the post that the other user just created.
  let alphaComment0Res = await createComment(
    alpha_user_mod,
    alphaPost0Res.post_view.post.id,
  );

  // mod on alpha distinguishes their own comment as 'speaking as moderator'
  let modActionResult = await alpha_user_mod.client.distinguishComment({
    distinguished: true,
    comment_id: alphaComment0Res.comment_view.comment.id,
    auth: alpha_user_mod.auth,
  });
  expect(modActionResult.comment_view.comment.distinguished).toBe(true);

  // gamma reads it
  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";
  // ap_id of a post is stable across all instances
  let gammaPostComments = await findPostFromListNewGetComments(
    gamma_user_non_mod,
    communityNameFull,
    alphaPost0Res.post_view.post.ap_id,
  );
  expect(gammaPostComments.comments.length).toBe(1);

  // FixMe: lemmy_server version 0.18.2 remote-home-remote replication bug here, next test will fail:
  // bug report: https://github.com/LemmyNet/lemmy/issues/3705
  // look at the comment, is it moderator distinguished "speaking as moderator"?
  expect(gammaPostComments.comments[0].comment.distinguished).toBe(true);
});

async function doBanUnbanUser(with_remove_data: boolean) {
  let communityNameFull = betaCommunityHome.name + "@lemmy-beta";

  // The ban target user in previous test had several posts and comments in the community of focus
  // grab a list before ban
  let beforeBanPosts = await getPosts(alpha_user_observer, communityNameFull);
  // 9 posts total so far, reminder that a non-subscriber may have posted before replication
  expect(beforeBanPosts.posts.length).toBe(9);

  // target user to ban, locate that user on gamma
  // the API object doesn't really know who they are, identity crisis
  let site = await getSite(alpha_user_non_mod);
  if (!site.my_user) {
    throw "Missing site user on alpha";
  }
  let apShortname = `@${site.my_user.local_user_view.person.name}@lemmy-alpha:8541`;
  let alphaPersonOnGamma = (await resolvePerson(gamma_user_mod, apShortname))
    .person;
  if (!alphaPersonOnGamma) {
    throw "Missing alpha person on gamma instance";
  }

  // gamma non-admin moderator bans the alpha non-mod user.
  // NOTE: REMOVE DATA option seems irreversable, afterUnbanPosts on alpha do NOT return!
  let modActionResult = await banPersonFromCommunity(
    gamma_user_mod,
    alphaPersonOnGamma.person.id,
    gammaCommunityRemote.id,
    with_remove_data,
    true,
  );

  // confirm the user who was banned is unable to post in community
  await expect(
    createPost(alpha_user_non_mod, alphaCommunityRemote.id),
  ).rejects.toBe("banned_from_community");

  // this user in precvious test had several posts and comments in the community of focus
  // grab a list after ban
  let afterBanPosts = await getPosts(alpha_user_observer, communityNameFull);
  // if content was not removed, count will be the same, otherwise count is 2.
  if (with_remove_data) {
    expect(afterBanPosts.posts.length).toBe(2);
    expect(afterBanPosts.posts[0].creator.name).toBe("gamma_nonsubscriber0");
    // is there a comment on tht post? Yes, it was created after replication subscribers.
    expect(afterBanPosts.posts[0].counts.comments).toBe(1);
    // ToDo: mystery, how exactly did the oldest post replicate from that user
    //    when there were no subscribers when they posted and commented?
    expect(afterBanPosts.posts[1].creator.name).toBe("gamma_nonsubscriber0");
    // is there a comment on tht post? No, there is not! It did not replicate.
    expect(afterBanPosts.posts[1].counts.comments).toBe(0);
  } else {
    expect(afterBanPosts.posts.length).toBe(beforeBanPosts.posts.length);
  }

  // gamma non-admin moderator reverses ban.
  // REMOVE DATA option, does it matter in unban? tried both true and false, do not get posts back.
  let modActionResult1 = await banPersonFromCommunity(
    gamma_user_mod,
    alphaPersonOnGamma.person.id,
    gammaCommunityRemote.id,
    with_remove_data,
    false,
  );

  let afterUnbanPosts = await getPosts(alpha_user_observer, communityNameFull);
  if (with_remove_data) {
    // FixMe: is it documented clearly that remove_data is irreversable?
    expect(afterUnbanPosts.posts.length).toBe(2);
  } else {
    expect(afterUnbanPosts.posts.length).toBe(beforeBanPosts.posts.length);
  }
}

// open issue on GitHub: https://github.com/LemmyNet/lemmy/issues/3535
test("non-admin remote moderator on gamma bans remote non-mod user from alpha, unbans", async () => {
  // create an alpha observer account that
  // doe snot need to follow community
  alpha_user_observer = await registerUserClient(alpha, "alpha_observer0");
  await doBanUnbanUser(false);
});

test("rerun previous ban test with remove_data", async () => {
  await doBanUnbanUser(true);
  // now that the counting is all done...
  // confirm the user who was banned/unbanned is again able to post in community
  await createPost(alpha_user_non_mod, alphaCommunityRemote.id);
});

test.skip("once the replication bugs previously identified are fixed, compare post & comment lists between alpha and gamma instances", async () => {
  // ToDo; include comparing from anonymous vs. logged-in accounts
  // reminder that a non-subscriber may have posted before replication
});
