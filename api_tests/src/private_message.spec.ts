jest.setTimeout(120000);
import { PrivateMessageView } from "lemmy-js-client";
import {
  alpha,
  beta,
  setupLogins,
  followBeta,
  createPrivateMessage,
  editPrivateMessage,
  deletePrivateMessage,
  waitUntil,
  reportPrivateMessage,
  unfollows,
  listInbox,
} from "./shared";

let recipient_id: number;

beforeAll(async () => {
  await setupLogins();
  await followBeta(alpha);
  recipient_id = 3;
});

afterAll(unfollows);

test("Create a private message", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  expect(pmRes.private_message_view.private_message.content).toBeDefined();
  expect(pmRes.private_message_view.private_message.local).toBe(true);
  expect(pmRes.private_message_view.creator.local).toBe(true);
  expect(pmRes.private_message_view.recipient.local).toBe(false);

  let betaPms = await waitUntil(
    () => listInbox(beta, "PrivateMessage"),
    e => !!e.inbox[0],
  );
  const firstPm = betaPms.inbox[0] as PrivateMessageView;
  expect(firstPm.private_message.content).toBeDefined();
  expect(firstPm.private_message.local).toBe(false);
  expect(firstPm.creator.local).toBe(false);
  expect(firstPm.recipient.local).toBe(true);
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
    () => listInbox(beta, "PrivateMessage"),
    p =>
      p.inbox[0].type_ == "PrivateMessage" &&
      p.inbox[0].private_message.content === updatedContent,
  );
  expect((betaPms.inbox[0] as PrivateMessageView).private_message.content).toBe(
    updatedContent,
  );
});

test("Delete a private message", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  let betaPms1 = await waitUntil(
    () => listInbox(beta, "PrivateMessage"),
    m =>
      !!m.inbox.find(
        e =>
          e.type_ == "PrivateMessage" &&
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
    () => listInbox(beta, "PrivateMessage"),
    p => p.inbox.length === betaPms1.inbox.length - 1,
  );
  expect(betaPms2.inbox.length).toBe(betaPms1.inbox.length - 1);

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
    () => listInbox(beta, "PrivateMessage"),
    p => p.inbox.length === betaPms1.inbox.length,
  );
  expect(betaPms3.inbox.length).toBe(betaPms1.inbox.length);
});

test("Create a private message report", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  let betaPms1 = await waitUntil(
    () => listInbox(beta, "PrivateMessage"),
    m =>
      !!m.inbox.find(
        e =>
          e.type_ == "PrivateMessage" &&
          e.private_message.ap_id ===
            pmRes.private_message_view.private_message.ap_id,
      ),
  );
  let betaPm = betaPms1.inbox[0] as PrivateMessageView;
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
