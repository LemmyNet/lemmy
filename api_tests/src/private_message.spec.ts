jest.setTimeout(120000);
import {
  alpha,
  beta,
  setupLogins,
  followBeta,
  createPrivateMessage,
  editPrivateMessage,
  listPrivateMessages,
  deletePrivateMessage,
  unfollowRemotes,
} from "./shared";

let recipient_id: number;

beforeAll(async () => {
  await setupLogins();
  await followBeta(alpha);
  recipient_id = 3;
});

afterAll(async () => {
  await unfollowRemotes(alpha);
});

test("Create a private message", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  expect(pmRes.private_message_view.private_message.content).toBeDefined();
  expect(pmRes.private_message_view.private_message.local).toBe(true);
  expect(pmRes.private_message_view.creator.local).toBe(true);
  expect(pmRes.private_message_view.recipient.local).toBe(false);

  let betaPms = await listPrivateMessages(beta);
  expect(betaPms.private_messages[0].private_message.content).toBeDefined();
  expect(betaPms.private_messages[0].private_message.local).toBe(false);
  expect(betaPms.private_messages[0].creator.local).toBe(false);
  expect(betaPms.private_messages[0].recipient.local).toBe(true);
});

test("Update a private message", async () => {
  let updatedContent = "A jest test federated private message edited";

  let pmRes = await createPrivateMessage(alpha, recipient_id);
  let pmUpdated = await editPrivateMessage(
    alpha,
    pmRes.private_message_view.private_message.id,
  );
  expect(pmUpdated.private_message_view.private_message.content).toBe(
    updatedContent,
  );

  let betaPms = await listPrivateMessages(beta);
  expect(betaPms.private_messages[0].private_message.content).toBe(
    updatedContent,
  );
});

test("Delete a private message", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  let betaPms1 = await listPrivateMessages(beta);
  let deletedPmRes = await deletePrivateMessage(
    alpha,
    true,
    pmRes.private_message_view.private_message.id,
  );
  expect(deletedPmRes.private_message_view.private_message.deleted).toBe(true);

  // The GetPrivateMessages filters out deleted,
  // even though they are in the actual database.
  // no reason to show them
  let betaPms2 = await listPrivateMessages(beta);
  expect(betaPms2.private_messages.length).toBe(
    betaPms1.private_messages.length - 1,
  );

  // Undelete
  let undeletedPmRes = await deletePrivateMessage(
    alpha,
    false,
    pmRes.private_message_view.private_message.id,
  );
  expect(undeletedPmRes.private_message_view.private_message.deleted).toBe(
    false,
  );

  let betaPms3 = await listPrivateMessages(beta);
  expect(betaPms3.private_messages.length).toBe(
    betaPms1.private_messages.length,
  );
});
