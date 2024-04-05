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
  waitUntil,
  reportPrivateMessage,
  unfollows,
} from "./shared";

let recipient_id: number;

beforeAll(async () => {
  await setupLogins();
  await followBeta(alpha);
  recipient_id = 3;
});

afterAll(async () => {
  await unfollows();
});

test("Create a private message", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  expect(pmRes.private_message_view.private_message.content).toBeDefined();
  expect(pmRes.private_message_view.private_message.local).toBe(true);
  expect(pmRes.private_message_view.creator.local).toBe(true);
  expect(pmRes.private_message_view.recipient.local).toBe(false);

  let betaPms = await waitUntil(
    () => listPrivateMessages(beta),
    e => !!e.private_messages[0],
  );
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

  let betaPms = await waitUntil(
    () => listPrivateMessages(beta),
    p => p.private_messages[0].private_message.content === updatedContent,
  );
  expect(betaPms.private_messages[0].private_message.content).toBe(
    updatedContent,
  );
});

test("Delete a private message", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  let betaPms1 = await waitUntil(
    () => listPrivateMessages(beta),
    m =>
      !!m.private_messages.find(
        e =>
          e.private_message.ap_id ===
          pmRes.private_message_view.private_message.ap_id,
      ),
  );
  let deletedPmRes = await deletePrivateMessage(
    alpha,
    true,
    pmRes.private_message_view.private_message.id,
  );
  expect(deletedPmRes.private_message_view.private_message.deleted).toBe(true);

  // The GetPrivateMessages filters out deleted,
  // even though they are in the actual database.
  // no reason to show them
  let betaPms2 = await waitUntil(
    () => listPrivateMessages(beta),
    p => p.private_messages.length === betaPms1.private_messages.length - 1,
  );
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

  let betaPms3 = await waitUntil(
    () => listPrivateMessages(beta),
    p => p.private_messages.length === betaPms1.private_messages.length,
  );
  expect(betaPms3.private_messages.length).toBe(
    betaPms1.private_messages.length,
  );
});

test("Create a private message report", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  let betaPms1 = await waitUntil(
    () => listPrivateMessages(beta),
    m =>
      !!m.private_messages.find(
        e =>
          e.private_message.ap_id ===
          pmRes.private_message_view.private_message.ap_id,
      ),
  );
  let betaPm = betaPms1.private_messages[0];
  expect(betaPm).toBeDefined();

  // Make sure that only the recipient can report it, so this should fail
  await expect(
    reportPrivateMessage(
      alpha,
      pmRes.private_message_view.private_message.id,
      "a reason",
    ),
  ).rejects.toStrictEqual(Error("couldnt_create_report"));

  // This one should pass
  let reason = "another reason";
  let report = await reportPrivateMessage(
    beta,
    betaPm.private_message.id,
    reason,
  );

  expect(report.private_message_report_view.private_message.id).toBe(
    betaPm.private_message.id,
  );
  expect(report.private_message_report_view.private_message_report.reason).toBe(
    reason,
  );
});
