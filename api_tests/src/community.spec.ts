jest.setTimeout(120000);

import { AddModToCommunity } from "lemmy-js-client/dist/types/AddModToCommunity";
import { CommunityView } from "lemmy-js-client/dist/types/CommunityView";
import {
  alpha,
  beta,
  gamma,
  setupLogins,
  resolveCommunity,
  createCommunity,
  deleteCommunity,
  delay,
  removeCommunity,
  getCommunity,
  followCommunity,
  banPersonFromCommunity,
  resolvePerson,
  createPost,
  getPost,
  resolvePost,
  registerUser,
  getPosts,
  getComments,
  createComment,
  getCommunityByName,
  waitUntil,
  alphaUrl,
  delta,
  longDelay,
  editCommunity,
  unfollows,
  getMyUser,
  userBlockInstance,
  resolveBetaCommunity,
  reportCommunity,
  randomString,
  listReports,
} from "./shared";
import { AdminAllowInstanceParams } from "lemmy-js-client/dist/types/AdminAllowInstanceParams";
import {
  CommunityReport,
  CommunityReportView,
  EditCommunity,
  FollowMultiCommunity,
  GetPosts,
  LemmyError,
  MultiCommunity,
  MultiCommunityView,
  ReportCombinedView,
  ResolveCommunityReport,
  Search,
} from "lemmy-js-client";

beforeAll(setupLogins);
afterAll(unfollows);

function assertCommunityFederation(
  communityOne?: CommunityView,
  communityTwo?: CommunityView,
) {
  expect(communityOne?.community.ap_id).toBe(communityTwo?.community.ap_id);
  expect(communityOne?.community.name).toBe(communityTwo?.community.name);
  expect(communityOne?.community.title).toBe(communityTwo?.community.title);
  expect(communityOne?.community.description).toBe(
    communityTwo?.community.description,
  );
  expect(communityOne?.community.icon).toBe(communityTwo?.community.icon);
  expect(communityOne?.community.banner).toBe(communityTwo?.community.banner);
  expect(communityOne?.community.published_at).toBe(
    communityTwo?.community.published_at,
  );
  expect(communityOne?.community.nsfw).toBe(communityTwo?.community.nsfw);
  expect(communityOne?.community.removed).toBe(communityTwo?.community.removed);
  expect(communityOne?.community.deleted).toBe(communityTwo?.community.deleted);
}

test("Create community", async () => {
  let communityRes = await createCommunity(alpha);
  expect(communityRes.community_view.community.name).toBeDefined();

  // A dupe check
  let prevName = communityRes.community_view.community.name;
  await expect(createCommunity(alpha, prevName)).rejects.toStrictEqual(
    new LemmyError("community_already_exists"),
  );

  // Cache the community on beta, make sure it has the other fields
  let searchShort = `!${prevName}@lemmy-alpha:8541`;
  let betaCommunity = await resolveCommunity(beta, searchShort);
  assertCommunityFederation(betaCommunity, communityRes.community_view);
});

test("Delete community", async () => {
  let communityRes = await createCommunity(beta);

  // Cache the community on Alpha
  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = await resolveCommunity(alpha, searchShort);
  if (!alphaCommunity) {
    throw "Missing alpha community";
  }
  assertCommunityFederation(alphaCommunity, communityRes.community_view);

  // Follow the community from alpha
  let follow = await followCommunity(alpha, true, alphaCommunity.community.id);

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(false);

  let deleteCommunityRes = await deleteCommunity(
    beta,
    true,
    communityRes.community_view.community.id,
  );
  expect(deleteCommunityRes.community_view.community.deleted).toBe(true);
  expect(deleteCommunityRes.community_view.community.title).toBe(
    communityRes.community_view.community.title,
  );

  // Make sure it got deleted on A
  let communityOnAlphaDeleted = await waitUntil(
    () => getCommunity(alpha, alphaCommunity!.community.id),
    g => g.community_view.community.deleted,
  );
  expect(communityOnAlphaDeleted.community_view.community.deleted).toBe(true);

  // Undelete
  let undeleteCommunityRes = await deleteCommunity(
    beta,
    false,
    communityRes.community_view.community.id,
  );
  expect(undeleteCommunityRes.community_view.community.deleted).toBe(false);

  // Make sure it got undeleted on A
  let communityOnAlphaUnDeleted = await waitUntil(
    () => getCommunity(alpha, alphaCommunity!.community.id),
    g => !g.community_view.community.deleted,
  );
  expect(communityOnAlphaUnDeleted.community_view.community.deleted).toBe(
    false,
  );
});

test("Remove community", async () => {
  let communityRes = await createCommunity(beta);

  // Cache the community on Alpha
  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = await resolveCommunity(alpha, searchShort);
  if (!alphaCommunity) {
    throw "Missing alpha community";
  }
  assertCommunityFederation(alphaCommunity, communityRes.community_view);

  // Follow the community from alpha
  let follow = await followCommunity(alpha, true, alphaCommunity.community.id);

  // Make sure the follow response went through
  expect(follow.community_view.community.local).toBe(false);

  let removeCommunityRes = await removeCommunity(
    beta,
    true,
    communityRes.community_view.community.id,
  );
  expect(removeCommunityRes.community_view.community.removed).toBe(true);
  expect(removeCommunityRes.community_view.community.title).toBe(
    communityRes.community_view.community.title,
  );

  // Make sure it got Removed on A
  let communityOnAlphaRemoved = await waitUntil(
    () => getCommunity(alpha, alphaCommunity!.community.id),
    g => g.community_view.community.removed,
  );
  expect(communityOnAlphaRemoved.community_view.community.removed).toBe(true);

  // unremove
  let unremoveCommunityRes = await removeCommunity(
    beta,
    false,
    communityRes.community_view.community.id,
  );
  expect(unremoveCommunityRes.community_view.community.removed).toBe(false);

  // Make sure it got undeleted on A
  let communityOnAlphaUnRemoved = await waitUntil(
    () => getCommunity(alpha, alphaCommunity!.community.id),
    g => !g.community_view.community.removed,
  );
  expect(communityOnAlphaUnRemoved.community_view.community.removed).toBe(
    false,
  );
});

test("Report a community", async () => {
  // Create community on alpha
  let alphaCommunity = await createCommunity(alpha);
  expect(alphaCommunity.community_view.community).toBeDefined();

  // Send report from beta
  let betaCommunity = await resolveCommunity(
    beta,
    alphaCommunity.community_view.community.ap_id,
  );
  let betaReport = (
    await reportCommunity(beta, betaCommunity!.community.id, randomString(10))
  ).community_report_view.community_report;
  expect(betaReport).toBeDefined();

  // Report was federated to alpha
  let alphaReport = (
    (await waitUntil(
      () =>
        listReports(alpha).then(p =>
          p.reports.find(r => {
            return checkCommunityReportName(r, betaReport);
          }),
        ),
      res => !!res,
    ))! as CommunityReportView
  ).community_report;
  expect(alphaReport).toBeDefined();
  expect(alphaReport.resolved).toBe(false);
  expect(alphaReport.original_community_name).toBe(
    betaReport.original_community_name,
  );
  expect(alphaReport.original_community_title).toBe(
    betaReport.original_community_title,
  );
  expect(alphaReport.original_community_banner).toBe(
    betaReport.original_community_banner,
  );
  expect(alphaReport.original_community_description).toBe(
    betaReport.original_community_description,
  );
  expect(alphaReport.original_community_icon).toBe(
    betaReport.original_community_icon,
  );
  expect(alphaReport.original_community_sidebar).toBe(
    betaReport.original_community_sidebar,
  );
  expect(alphaReport.reason).toBe(betaReport.reason);

  // Resolve report as admin of the community's instance
  let resolveParams: ResolveCommunityReport = {
    report_id: alphaReport.id,
    resolved: true,
  };
  let resolve = await alpha.resolveCommunityReport(resolveParams);
  expect(resolve.community_report_view.community_report.resolved).toBeTruthy();

  // Report should be marked resolved on reporter's instance
  let resolvedReport = (
    (await waitUntil(
      () =>
        listReports(beta).then(p =>
          p.reports.find(r => {
            return (
              checkCommunityReportName(r, alphaReport) && r.resolver != null
            );
          }),
        ),
      res => !!res,
    ))! as CommunityReportView
  ).community_report;
  expect(resolvedReport).toBeDefined();
  expect(resolvedReport.resolved).toBe(true);
});

test("Search for beta community", async () => {
  let communityRes = await createCommunity(beta);
  expect(communityRes.community_view.community.name).toBeDefined();

  let searchShort = `!${communityRes.community_view.community.name}@lemmy-beta:8551`;
  let alphaCommunity = await resolveCommunity(alpha, searchShort);
  assertCommunityFederation(alphaCommunity, communityRes.community_view);
});

test("Admin actions in remote community are not federated to origin", async () => {
  // create a community on alpha
  let communityRes = (await createCommunity(alpha)).community_view;
  expect(communityRes.community.name).toBeDefined();

  // gamma follows community and posts in it
  let gammaCommunity = await resolveCommunity(
    gamma,
    communityRes.community.ap_id,
  );
  if (!gammaCommunity) {
    throw "Missing gamma community";
  }
  await followCommunity(gamma, true, gammaCommunity.community.id);
  gammaCommunity = await waitUntil(
    () => resolveCommunity(gamma, communityRes.community.ap_id),
    g => g?.community_actions?.follow_state == "Accepted",
  );
  if (!gammaCommunity) {
    throw "Missing gamma community";
  }
  expect(gammaCommunity.community_actions?.follow_state).toBe("Accepted");
  let gammaPost = (await createPost(gamma, gammaCommunity.community.id))
    .post_view;
  expect(gammaPost.post.id).toBeDefined();
  expect(gammaPost.creator_community_actions?.received_ban_at).toBeUndefined();

  // admin of beta decides to ban gamma from community
  let betaCommunity = await resolveCommunity(
    beta,
    communityRes.community.ap_id,
  );
  if (!betaCommunity) {
    throw "Missing beta community";
  }
  let bannedUserInfo1 = (await getMyUser(gamma)).local_user_view.person;
  if (!bannedUserInfo1) {
    throw "Missing banned user 1";
  }
  let bannedUserInfo2 = await resolvePerson(beta, bannedUserInfo1.ap_id);

  if (!bannedUserInfo2) {
    throw "Missing banned user 2";
  }
  let banRes = await banPersonFromCommunity(
    beta,
    bannedUserInfo2.person.id,
    betaCommunity.community.id,
    true,
    true,
  );
  expect(banRes.banned).toBe(true);

  // ban doesn't federate to community's origin instance alpha
  let alphaPost = await resolvePost(alpha, gammaPost.post);
  expect(alphaPost?.creator_community_actions?.received_ban_at).toBeUndefined();

  // and neither to gamma
  let gammaPost2 = await getPost(gamma, gammaPost.post.id);
  expect(
    gammaPost2.post_view.creator_community_actions?.received_ban_at,
  ).toBeUndefined();
});

test("moderator view", async () => {
  // register a new user with their own community on alpha and post to it
  let otherUser = await registerUser(alpha, alphaUrl);

  let otherCommunity = (await createCommunity(otherUser)).community_view;
  expect(otherCommunity.community.name).toBeDefined();
  let otherPost = (await createPost(otherUser, otherCommunity.community.id))
    .post_view;
  expect(otherPost.post.id).toBeDefined();

  let otherComment = (await createComment(otherUser, otherPost.post.id))
    .comment_view;
  expect(otherComment.comment.id).toBeDefined();

  // create a community and post on alpha
  let alphaCommunity = (await createCommunity(alpha)).community_view;
  expect(alphaCommunity.community.name).toBeDefined();
  let alphaPost = (await createPost(alpha, alphaCommunity.community.id))
    .post_view;
  expect(alphaPost.post.id).toBeDefined();

  let alphaComment = (await createComment(otherUser, alphaPost.post.id))
    .comment_view;
  expect(alphaComment.comment.id).toBeDefined();

  // other user also posts on alpha's community
  let otherAlphaPost = (
    await createPost(otherUser, alphaCommunity.community.id)
  ).post_view;
  expect(otherAlphaPost.post.id).toBeDefined();

  let otherAlphaComment = (
    await createComment(otherUser, otherAlphaPost.post.id)
  ).comment_view;
  expect(otherAlphaComment.comment.id).toBeDefined();

  // alpha lists posts and comments on home page, should contain all posts that were made
  let posts = (await getPosts(alpha, "All")).posts;
  expect(posts).toBeDefined();
  let postIds = posts.map(post => post.post.id);

  let comments = (await getComments(alpha, undefined, "All")).comments;
  expect(comments).toBeDefined();
  let commentIds = comments.map(comment => comment.comment.id);

  expect(postIds).toContain(otherPost.post.id);
  expect(commentIds).toContain(otherComment.comment.id);

  expect(postIds).toContain(alphaPost.post.id);
  expect(commentIds).toContain(alphaComment.comment.id);

  expect(postIds).toContain(otherAlphaPost.post.id);
  expect(commentIds).toContain(otherAlphaComment.comment.id);

  // in moderator view, alpha should not see otherPost, wich was posted on a community alpha doesn't moderate
  posts = (await getPosts(alpha, "ModeratorView")).posts;
  expect(posts).toBeDefined();
  postIds = posts.map(post => post.post.id);

  comments = (await getComments(alpha, undefined, "ModeratorView")).comments;
  expect(comments).toBeDefined();
  commentIds = comments.map(comment => comment.comment.id);

  expect(postIds).not.toContain(otherPost.post.id);
  expect(commentIds).not.toContain(otherComment.comment.id);

  expect(postIds).toContain(alphaPost.post.id);
  expect(commentIds).toContain(alphaComment.comment.id);

  expect(postIds).toContain(otherAlphaPost.post.id);
  expect(commentIds).toContain(otherAlphaComment.comment.id);
});

test("Get community for different casing on domain", async () => {
  let communityRes = await createCommunity(alpha);
  expect(communityRes.community_view.community.name).toBeDefined();

  // A dupe check
  let prevName = communityRes.community_view.community.name;
  await expect(createCommunity(alpha, prevName)).rejects.toStrictEqual(
    new LemmyError("community_already_exists"),
  );

  // Cache the community on beta, make sure it has the other fields
  let communityName = `${communityRes.community_view.community.name}@LEMMY-ALPHA:8541`;
  let betaCommunity = (await getCommunityByName(beta, communityName))
    .community_view;
  assertCommunityFederation(betaCommunity, communityRes.community_view);
});

test("User blocks instance, communities are hidden", async () => {
  // create community and post on beta
  let communityRes = await createCommunity(beta);
  expect(communityRes.community_view.community.name).toBeDefined();
  let postRes = await createPost(
    beta,
    communityRes.community_view.community.id,
  );
  expect(postRes.post_view.post.id).toBeDefined();

  // fetch post to alpha
  let alphaPost = await resolvePost(alpha, postRes.post_view.post);
  expect(alphaPost?.post).toBeDefined();

  // post should be included in listing
  let listing = await getPosts(alpha, "All");
  let listing_ids = listing.posts.map(p => p.post.ap_id);
  expect(listing_ids).toContain(postRes.post_view.post.ap_id);

  // block the beta instance
  await userBlockInstance(alpha, alphaPost!.community.instance_id, true);

  // after blocking, post should not be in listing
  let listing2 = await getPosts(alpha, "All");
  let listing_ids2 = listing2.posts.map(p => p.post.ap_id);
  expect(listing_ids2.indexOf(postRes.post_view.post.ap_id)).toBe(-1);

  // unblock instance again
  await userBlockInstance(alpha, alphaPost!.community.instance_id, false);

  // post should be included in listing
  let listing3 = await getPosts(alpha, "All");
  let listing_ids3 = listing3.posts.map(p => p.post.ap_id);
  expect(listing_ids3).toContain(postRes.post_view.post.ap_id);
});

// TODO: this test keeps failing randomly in CI
test.skip("Community follower count is federated", async () => {
  // Follow the beta community from alpha
  let community = await createCommunity(beta);
  let communityActorId = community.community_view.community.ap_id;
  let resolved = await resolveCommunity(alpha, communityActorId);
  if (!resolved?.community) {
    throw "Missing beta community";
  }

  await followCommunity(alpha, true, resolved.community.id);
  let followed = await waitUntil(
    () => resolveCommunity(alpha, communityActorId),
    c => c?.community_actions?.follow_state == "Accepted",
  );

  // Make sure there is 1 subscriber
  expect(followed?.community.subscribers).toBe(1);

  // Follow the community from gamma
  resolved = await resolveCommunity(gamma, communityActorId);
  if (!resolved?.community) {
    throw "Missing beta community";
  }

  await followCommunity(gamma, true, resolved.community.id);
  followed = await waitUntil(
    () => resolveCommunity(gamma, communityActorId),
    c => c?.community_actions?.follow_state == "Accepted",
  );

  // Make sure there are 2 subscribers
  expect(followed?.community?.subscribers).toBe(2);

  // Follow the community from delta
  resolved = await resolveCommunity(delta, communityActorId);
  if (!resolved?.community) {
    throw "Missing beta community";
  }

  await followCommunity(delta, true, resolved.community.id);
  followed = await waitUntil(
    () => resolveCommunity(delta, communityActorId),
    c => c?.community_actions?.follow_state == "Accepted",
  );
});

test("Dont receive community activities after unsubscribe", async () => {
  let communityRes = await createCommunity(alpha);
  expect(communityRes.community_view.community.name).toBeDefined();
  expect(communityRes.community_view.community.subscribers).toBe(1);

  let betaCommunity = await resolveCommunity(
    beta,
    communityRes.community_view.community.ap_id,
  );
  assertCommunityFederation(betaCommunity, communityRes.community_view);

  // follow alpha community from beta
  await followCommunity(beta, true, betaCommunity!.community.id);

  // ensure that follower count was updated
  let communityRes1 = await getCommunity(
    alpha,
    communityRes.community_view.community.id,
  );
  expect(communityRes1.community_view.community.subscribers).toBe(2);

  // temporarily block alpha, so that it doesn't know about unfollow
  var allow_instance_params: AdminAllowInstanceParams = {
    instance: "lemmy-alpha",
    allow: false,
    reason: undefined,
  };
  await beta.adminAllowInstance(allow_instance_params);
  await longDelay();

  // unfollow
  await followCommunity(beta, false, betaCommunity!.community.id);

  // ensure that alpha still sees beta as follower
  let communityRes2 = await getCommunity(
    alpha,
    communityRes.community_view.community.id,
  );
  expect(communityRes2.community_view.community.subscribers).toBe(2);

  // unblock alpha
  allow_instance_params.allow = true;
  await beta.adminAllowInstance(allow_instance_params);
  await longDelay();

  // create a post, it shouldnt reach beta
  let postRes = await createPost(
    alpha,
    communityRes.community_view.community.id,
  );
  expect(postRes.post_view.post.id).toBeDefined();
  // await longDelay();

  let form: Search = {
    q: postRes.post_view.post.name,
    type_: "Posts",
    listing_type: "All",
  };

  let res = await beta.search(form);
  expect(res.results.length).toBe(0);
});

test("Fetch community, includes posts", async () => {
  let communityRes = await createCommunity(alpha);
  expect(communityRes.community_view.community.name).toBeDefined();
  expect(communityRes.community_view.community.subscribers).toBe(1);

  let postRes = await createPost(
    alpha,
    communityRes.community_view.community.id,
  );
  expect(postRes.post_view.post).toBeDefined();

  let resolvedCommunity = await waitUntil(
    () => resolveCommunity(beta, communityRes.community_view.community.ap_id),
    c => c?.community.id != undefined,
  );
  let betaCommunity = resolvedCommunity;
  expect(betaCommunity?.community.ap_id).toBe(
    communityRes.community_view.community.ap_id,
  );

  await longDelay();

  let post_listing = await getPosts(beta, "All", betaCommunity?.community.id);
  expect(post_listing.posts.length).toBe(1);
  expect(post_listing.posts[0].post.ap_id).toBe(postRes.post_view.post.ap_id);
});

test("Content in local-only community doesn't federate", async () => {
  // create a community and set it local-only
  let communityRes = (await createCommunity(alpha)).community_view.community;
  let form: EditCommunity = {
    community_id: communityRes.id,
    visibility: "LocalOnlyPublic",
  };
  await editCommunity(alpha, form);

  // cant resolve the community from another instance
  await expect(
    resolveCommunity(beta, communityRes.ap_id),
  ).rejects.toStrictEqual(new LemmyError("not_found"));

  // create a post, also cant resolve it
  let postRes = await createPost(alpha, communityRes.id);
  await expect(resolvePost(beta, postRes.post_view.post)).rejects.toStrictEqual(
    new LemmyError("not_found"),
  );
});

test("Remote mods can edit communities", async () => {
  let communityRes = await createCommunity(alpha);

  let betaCommunity = await resolveCommunity(
    beta,
    communityRes.community_view.community.ap_id,
  );
  if (!betaCommunity?.community) {
    throw "Missing beta community";
  }
  let betaOnAlpha = await resolvePerson(alpha, "lemmy_beta@lemmy-beta:8551");

  let form: AddModToCommunity = {
    community_id: communityRes.community_view.community.id,
    person_id: betaOnAlpha?.person.id as number,
    added: true,
  };
  alpha.addModToCommunity(form);

  let form2: EditCommunity = {
    community_id: betaCommunity.community.id as number,
    description: "Example description",
  };

  await editCommunity(beta, form2);
  // give alpha time to get and process the edit
  await delay(1000);

  let alphaCommunity = await getCommunity(
    alpha,
    communityRes.community_view.community.id,
  );

  expect(alphaCommunity.community_view.community.description).toBe(
    "Example description",
  );
});

test("Community name with non-ascii chars", async () => {
  const name = "това_ме_ядосва" + Math.random().toString().slice(2, 6);
  let communityRes = await createCommunity(alpha, name);

  let betaCommunity1 = await resolveCommunity(
    beta,
    communityRes.community_view.community.ap_id,
  );
  expect(betaCommunity1?.community.name).toBe(name);

  let alphaCommunity2 = await getCommunityByName(alpha, name);
  expect(alphaCommunity2.community_view.community.name).toBe(name);

  let fediName = `${communityRes.community_view.community.name}@LEMMY-ALPHA:8541`;
  let betaCommunity2 = await getCommunityByName(beta, fediName);
  expect(betaCommunity2.community_view.community.name).toBe(name);

  let postRes = await createPost(beta, betaCommunity1!.community.id);

  let form: GetPosts = {
    community_name: fediName,
  };
  let posts = await beta.getPosts(form);
  expect(posts.posts.length).toBe(1);
  expect(posts.posts[0].post.name).toBe(postRes.post_view.post.name);
});

test("Multi-community", async () => {
  // create multi
  let res = await alpha.createMultiCommunity({ name: "multi-comm" });
  let myUser = await getMyUser(alpha);
  expect(res.multi_community_view.multi.name).toBe("multi-comm");
  expect(res.multi_community_view.multi.ap_id).toBe(
    "http://lemmy-alpha:8541/m/multi-comm",
  );
  expect(res.multi_community_view.owner.id).toBe(
    myUser.local_user_view.person.id,
  );

  // add initial community
  let community1 = (await createCommunity(alpha)).community_view.community;
  let success1 = await alpha.createMultiCommunityEntry({
    id: res.multi_community_view.multi.id,
    community_id: community1.id,
  });
  expect(success1.success).toBeTruthy();

  // resolve over federation
  let betaMulti = (
    await beta.resolveObject({ q: res.multi_community_view.multi.ap_id })
  ).results[0] as MultiCommunityView;
  expect(betaMulti.multi.ap_id).toBe(res.multi_community_view.multi.ap_id);

  var betaRes = await waitUntil(
    () => beta.getMultiCommunity({ id: betaMulti.multi.id }),
    m => m.communities.length == 1,
  );
  expect(betaRes.communities[0].community.ap_id).toBe(community1.ap_id);

  // follow multi over federation
  let form: FollowMultiCommunity = {
    multi_community_id: betaMulti.multi.id,
    follow: true,
  };
  await beta.followMultiCommunity(form);

  let followed = await waitUntil(
    () => beta.listMultiCommunities({ followed_only: true }),
    m => m.multi_communities.length == 1,
  );
  expect(followed.multi_communities[0].multi.ap_id).toBe(betaMulti.multi.ap_id);
  await delay();

  // add community to multi
  let community2 = await resolveBetaCommunity(alpha);
  let success2 = await alpha.createMultiCommunityEntry({
    id: res.multi_community_view.multi.id,
    community_id: community2!.community.id,
  });
  expect(success2.success).toBeTruthy();

  // federated to beta
  betaRes = await waitUntil(
    () => beta.getMultiCommunity({ id: betaMulti.multi.id }),
    m => m.communities.length == 2,
  );
  let ap_ids = betaRes.communities.map(c => c.community.ap_id);
  expect(ap_ids.includes(community2!.community.ap_id)).toBeTruthy();

  let post = await createPost(alpha, community2!.community.id);

  let multi_post_listing = await waitUntil(
    () =>
      beta.getPosts({
        multi_community_id: betaRes.multi_community_view.multi.id,
      }),
    p => p.posts.length == 1,
  );
  expect(multi_post_listing.posts[0].post.ap_id).toBe(
    post.post_view.post.ap_id,
  );
});

function checkCommunityReportName(
  rcv: ReportCombinedView,
  report: CommunityReport,
) {
  switch (rcv.type_) {
    case "Community":
      return (
        rcv.community_report.original_community_name ===
        report.original_community_name
      );
    default:
      return false;
  }
}
