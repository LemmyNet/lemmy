import fetch from 'node-fetch';

test('adds 1 + 2 to equal 3', () => {
  let sum = (a: number, b: number) => a + b;
  expect(sum(1, 2)).toBe(3);
});

test('Get communism.lemmy.ml nodeinfo href', async () => {
  let url = 'https://communism.lemmy.ml/.well-known/nodeinfo';
  let href = 'https://communism.lemmy.ml/nodeinfo/2.0.json';
  let res = await fetch(url).then(d => d.json());
  expect(res.links.href).toBe(href);
});
