jest.setTimeout(120000);
import {
  alpha,
  beta,
  setupLogins,
  followBeta,
  createPrivateMessage,
  updatePrivateMessage,
  listPrivateMessages,
  deletePrivateMessage,
  unfollowRemotes,
  delay,
} from './shared';

let recipient_id: number;

beforeAll(async () => {
  await setupLogins();
  let follow = await followBeta(alpha);
  await delay(10000);
  recipient_id = follow.community.creator_id;
});

afterAll(async () => {
  await unfollowRemotes(alpha);
});

test('Create a private message', async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  expect(pmRes.message.content).toBeDefined();
  expect(pmRes.message.local).toBe(true);
  expect(pmRes.message.creator_local).toBe(true);
  expect(pmRes.message.recipient_local).toBe(false);
  await delay();

  let betaPms = await listPrivateMessages(beta);
  expect(betaPms.messages[0].content).toBeDefined();
  expect(betaPms.messages[0].local).toBe(false);
  expect(betaPms.messages[0].creator_local).toBe(false);
  expect(betaPms.messages[0].recipient_local).toBe(true);
});

test('Update a private message', async () => {
  let updatedContent = 'A jest test federated private message edited';

  let pmRes = await createPrivateMessage(alpha, recipient_id);
  let pmUpdated = await updatePrivateMessage(alpha, pmRes.message.id);
  expect(pmUpdated.message.content).toBe(updatedContent);
  await delay();

  let betaPms = await listPrivateMessages(beta);
  expect(betaPms.messages[0].content).toBe(updatedContent);
});

test('Delete a private message', async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  await delay();
  let betaPms1 = await listPrivateMessages(beta);
  let deletedPmRes = await deletePrivateMessage(alpha, true, pmRes.message.id);
  expect(deletedPmRes.message.deleted).toBe(true);
  await delay();

  // The GetPrivateMessages filters out deleted,
  // even though they are in the actual database.
  // no reason to show them
  let betaPms2 = await listPrivateMessages(beta);
  expect(betaPms2.messages.length).toBe(betaPms1.messages.length - 1);
  await delay();

  // Undelete
  let undeletedPmRes = await deletePrivateMessage(
    alpha,
    false,
    pmRes.message.id
  );
  expect(undeletedPmRes.message.deleted).toBe(false);
  await delay();

  let betaPms3 = await listPrivateMessages(beta);
  expect(betaPms3.messages.length).toBe(betaPms1.messages.length);
});
