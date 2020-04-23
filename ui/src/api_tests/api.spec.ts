import fetch from 'node-fetch';

import {
  LoginForm,
  LoginResponse,
  GetPostsForm,
  GetPostsResponse,
  CommentForm,
  CommentResponse,
  ListingType,
  SortType,
} from '../interfaces';

let baseUrl = 'https://test.lemmy.ml';
let apiUrl = `${baseUrl}/api/v1`;
let auth: string;

beforeAll(async () => {
  console.log('Logging in as test user.');
  let form: LoginForm = {
    username_or_email: 'tester',
    password: 'tester',
  };

  let res: LoginResponse = await fetch(`${apiUrl}/user/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(form),
  }).then(d => d.json());

  auth = res.jwt;
});

test('Get test user posts', async () => {
  let form: GetPostsForm = {
    type_: ListingType[ListingType.All],
    sort: SortType[SortType.TopAll],
    auth,
  };

  let res: GetPostsResponse = await fetch(
    `${apiUrl}/post/list?type_=${form.type_}&sort=${form.sort}&auth=${auth}`
  ).then(d => d.json());

  // console.debug(res);

  expect(res.posts[0].id).toBe(2);
});

test('Create test comment', async () => {
  let content = 'A jest test comment';
  let form: CommentForm = {
    post_id: 2,
    content,
    auth,
  };

  let res: CommentResponse = await fetch(`${apiUrl}/comment`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: wrapper(form),
  }).then(d => d.json());

  expect(res.comment.content).toBe(content);
});

test('adds 1 + 2 to equal 3', () => {
  let sum = (a: number, b: number) => a + b;
  expect(sum(1, 2)).toBe(3);
});

test(`Get ${baseUrl} nodeinfo href`, async () => {
  let url = `${baseUrl}/.well-known/nodeinfo`;
  let href = `${baseUrl}/nodeinfo/2.0.json`;
  let res = await fetch(url).then(d => d.json());
  expect(res.links.href).toBe(href);
});

function wrapper(form: any): string {
  return JSON.stringify(form);
}
