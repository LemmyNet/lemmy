/*
With Lemmy 0.18.3 and earlier, performance has been a big concern.
Logging basic expectations of response times is the purpose of this module.
*/
jest.setTimeout(120 * 1000);

import { PostResponse } from "lemmy-js-client";
import { alpha, API, beta, createCommunity, followCommunity, resolveCommunity, setupLogins, createPost, createComment, likeComment, likePost, registerUser } from "./shared";

beforeAll(async () => {
    await setupLogins();
});

afterAll(async () => {
});


async function registerUserClient(withapi: API, username: string) {
    let registerUserRes = await registerUser(withapi, username);
    // this client being coppied from the other client, is that odd?
    let newUser: API = {
      client: withapi.client,
      auth: registerUserRes.jwt ?? "",
    };
    return newUser;
}

// reference: https://stackoverflow.com/questions/58461792/timing-function-calls-in-jest
test("benchmark baseline: community, post, comment", async () => {
    let prevPost : PostResponse | undefined;
    let prevComment;

    let alpha_user_casual0 = await registerUserClient(alpha, "alpha_casual0");

    const start = performance.now();

    for (let i = 0; i < 13; i++) {
        const name = "series_" + i;
        let communityRes = await createCommunity(alpha, name);
        expect(communityRes.community_view.community.name).toBeDefined();
    
        // Cache the community on beta, make sure it has the other fields
        let searchShort = `!${name}@lemmy-alpha:8541`;
        let betaCommunity = (await resolveCommunity(beta, searchShort)).community;

        if (!betaCommunity) {
            throw "betaCommunity resolve failure";
        }
        await followCommunity(beta, true, betaCommunity.community.id);

        let postRes = await createPost(alpha, communityRes.community_view.community.id);
        let commentRes = await createComment(alpha, postRes.post_view.post.id);

        if (prevComment) {
            if (prevPost) {
                await createComment(alpha, prevPost?.post_view.post.id, prevComment.comment_view.comment.id, "reply to previous " + i);
            }
        }

        // Other user upvotes.
        await likePost(alpha_user_casual0, 1, postRes.post_view.post);
        await likeComment(alpha_user_casual0, 1, commentRes.comment_view.comment);
        prevPost = postRes;
        prevComment = commentRes;
    }

    const end = performance.now();
    // 60 seconds is NOT good performance for 13 loops. I suggest 8 or even 1.3 seconds as a goal on empty database.
    expect(end - start).toBeLessThan(60 * 1000);   
});
