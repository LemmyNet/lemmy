import fetch from 'node-fetch';

import {
  LoginForm,
  LoginResponse,
  PostForm,
  PostResponse,
  SearchResponse,
} from '../interfaces';

let lemmyAlphaUrl = 'http://localhost:8540';
let lemmyBetaUrl = 'http://localhost:8550';
let lemmyAlphaApiUrl = `${lemmyAlphaUrl}/api/v1`;
let lemmyBetaApiUrl = `${lemmyBetaUrl}/api/v1`;
let lemmyAlphaAuth: string;

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
  });

  test('Create test post on alpha and fetch it on beta', async () => {
    let name = 'A jest test post';
    let postForm: PostForm = {
      name,
      auth: lemmyAlphaAuth,
      community_id: 2,
      creator_id: 2,
      nsfw: false,
    };

    let createResponse: PostResponse = await fetch(`${lemmyAlphaApiUrl}/post`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: wrapper(postForm),
    }).then(d => d.json());
    expect(createResponse.post.name).toBe(name);

    let searchUrl = `${lemmyBetaApiUrl}/search?q=${createResponse.post.ap_id}&type_=All&sort=TopAll`;
    let searchResponse: SearchResponse = await fetch(searchUrl, {
      method: 'GET',
    }).then(d => d.json());

    // TODO: check more fields
    expect(searchResponse.posts[0].name).toBe(name);
  });

  function wrapper(form: any): string {
    return JSON.stringify(form);
  }
});
