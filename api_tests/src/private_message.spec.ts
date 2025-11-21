jest.setTimeout(120000);
import { LemmyError, PrivateMessageView } from "lemmy-js-client";
import {
  alpha,
  beta,
  setupLogins,
  createPrivateMessage,
  editPrivateMessage,
  deletePrivateMessage,
  waitUntil,
  reportPrivateMessage,
  unfollows,
  listNotifications,
  resolvePerson,
  statusBadRequest,
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
    () => listNotifications(beta, "private_message"),
    e => !!e.data[0],
  );
  const firstPm = betaPms.data[0].data as PrivateMessageView;
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
    () => listNotifications(beta, "private_message"),
    p =>
      p.data[0].data.type_ == "private_message" &&
      p.data[0].data.private_message.content === updatedContent,
  );
  let pm = betaPms.data[0].data as PrivateMessageView;
  expect(pm.private_message.content).toBe(updatedContent);
});

test("Delete a private message", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  let betaPms1 = await waitUntil(
    () => listNotifications(beta, "private_message"),
    m =>
      !!m.data.find(
        e =>
          e.data.type_ == "private_message" &&
          e.data.private_message.ap_id ===
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
    () => listNotifications(beta, "private_message"),
    p => p.data.length === betaPms1.data.length - 1,
  );
  expect(betaPms2.data.length).toBe(betaPms1.data.length - 1);

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
    () => listNotifications(beta, "private_message"),
    p => p.data.length === betaPms1.data.length,
  );
  expect(betaPms3.data.length).toBe(betaPms1.data.length);
});

test("Create a private message report", async () => {
  let pmRes = await createPrivateMessage(alpha, recipient_id);
  let betaPms1 = await waitUntil(
    () => listNotifications(beta, "private_message"),
    m =>
      !!m.data.find(
        e =>
          e.data.type_ == "private_message" &&
          e.data.private_message.ap_id ===
            pmRes.private_message_view.private_message.ap_id,
      ),
  );
  let betaPm = betaPms1.data[0].data as PrivateMessageView;
  expect(betaPm).toBeDefined();

  // Make sure that only the recipient can report it, so this should fail
  await expect(
    reportPrivateMessage(
      alpha,
      pmRes.private_message_view.private_message.id,
      "a reason",
    ),
  ).rejects.toStrictEqual(
    new LemmyError("couldnt_create_report", statusBadRequest),
  );

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
