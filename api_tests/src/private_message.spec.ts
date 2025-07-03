jest.setTimeout(120000);
import {
  LemmyError,
  PrivateMessage,
  PrivateMessageView,
} from "lemmy-js-client";
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
  resolvePerson,
} from "./shared";

let recipient_id: number;

beforeAll(async () => {
  await setupLogins();
  let betaUser = await beta.getMyUser();
  let betaUserOnAlpha = await resolvePerson(
    alpha,
    betaUser.local_user_view.person.ap_id,
  );
  recipient_id = betaUserOnAlpha!.person.id;
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
  const firstPm = betaPms.inbox[0].data as {
    type_: "PrivateMessage";
    pm: PrivateMessage;
  };
  expect(firstPm.pm.content).toBeDefined();
  expect(firstPm.pm.local).toBe(false);
  expect(betaPms.inbox[0].creator.local).toBe(false);
  expect(betaPms.inbox[0].recipient.local).toBe(true);
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
      p.inbox[0].data.type_ == "PrivateMessage" &&
      p.inbox[0].data.pm.content === updatedContent,
  );
  let pm = betaPms.inbox[0].data as {
    type_: "PrivateMessage";
    pm: PrivateMessage;
  };
  expect(pm.pm.content).toBe(updatedContent);
});

test("Delete a private message", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  let betaPms1 = await waitUntil(
    () => listInbox(beta, "PrivateMessage"),
    m =>
      !!m.inbox.find(
        e =>
          e.data.type_ == "PrivateMessage" &&
          e.data.pm.ap_id === pmRes.private_message_view.private_message.ap_id,
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
          e.data.type_ == "PrivateMessage" &&
          e.data.pm.ap_id === pmRes.private_message_view.private_message.ap_id,
      ),
  );
  let betaPm = betaPms1.inbox[0].data as {
    type_: "PrivateMessage";
    pm: PrivateMessage;
  };
  expect(betaPm).toBeDefined();

  // Make sure that only the recipient can report it, so this should fail
  await expect(
    reportPrivateMessage(
      alpha,
      pmRes.private_message_view.private_message.id,
      "a reason",
    ),
  ).rejects.toStrictEqual(new LemmyError("couldnt_create_report"));

  // This one should pass
  let reason = "another reason";
  let report = await reportPrivateMessage(beta, betaPm.pm.id, reason);

  expect(report.private_message_report_view.private_message.id).toBe(
    betaPm.pm.id,
  );
  expect(report.private_message_report_view.private_message_report.reason).toBe(
    reason,
  );
});
