jest.setTimeout(120000);
import {
  LemmyError,
  PrivateMessageReportView,
  PrivateMessageView,
} from "lemmy-js-client";
import {
  alpha,
  beta,
  setupLogins,
  createPrivateMessage,
  editPrivateMessage,
  deletePrivateMessage,
  reportPrivateMessage,
  unfollows,
  listNotifications,
  resolvePerson,
  statusBadRequest,
  jestLemmyError,
  expectSuccess,
  waitUntilSuccess,
  expectFailure,
  listReports,
} from "./shared";

let recipient_id: number;

beforeAll(async () => {
  await setupLogins();
  const betaUser = await beta.getMyUser().then(expectSuccess);
  const betaUserOnAlpha = await resolvePerson(
    alpha,
    betaUser.local_user_view.person.ap_id,
  );
  recipient_id = betaUserOnAlpha!.person.id;
});

afterAll(unfollows);

test("Create a private message", async () => {
  const pmRes = await createPrivateMessage(alpha, recipient_id).then(
    expectSuccess,
  );
  expect(pmRes.private_message_view.private_message.content).toBeDefined();
  expect(pmRes.private_message_view.private_message.local).toBe(true);
  expect(pmRes.private_message_view.creator.local).toBe(true);
  expect(pmRes.private_message_view.recipient.local).toBe(false);

  const betaPms = await waitUntilSuccess(
    () => listNotifications(beta, "private_message"),
    e => !!e.items[0],
  );
  const firstPm = betaPms.items[0].data as PrivateMessageView;
  expect(firstPm.private_message.content).toBeDefined();
  expect(firstPm.private_message.local).toBe(false);
  expect(firstPm.creator.local).toBe(false);
  expect(firstPm.recipient.local).toBe(true);
});

test("Update a private message", async () => {
  const updatedContent = "A jest test federated private message edited";

  const pmRes = await createPrivateMessage(alpha, recipient_id).then(
    expectSuccess,
  );
  const pmUpdated = await editPrivateMessage(
    alpha,
    pmRes.private_message_view.private_message.id,
  ).then(expectSuccess);
  expect(pmUpdated.private_message_view.private_message.content).toBe(
    updatedContent,
  );

  const betaPms = await waitUntilSuccess(
    () => listNotifications(beta, "private_message"),
    p =>
      p.items[0].data.type_ == "private_message" &&
      p.items[0].data.private_message.content === updatedContent,
  );
  const pm = betaPms.items[0].data as PrivateMessageView;
  expect(pm.private_message.content).toBe(updatedContent);
});

test("Delete a private message", async () => {
  const pmRes = await createPrivateMessage(alpha, recipient_id).then(
    expectSuccess,
  );
  const betaPms1 = await waitUntilSuccess(
    () => listNotifications(beta, "private_message"),
    m =>
      !!m.items.find(
        e =>
          e.data.type_ == "private_message" &&
          e.data.private_message.ap_id ===
            pmRes.private_message_view.private_message.ap_id,
      ),
  );
  const deletedPmRes = await deletePrivateMessage(
    alpha,
    true,
    pmRes.private_message_view.private_message.id,
  ).then(expectSuccess);
  expect(deletedPmRes.private_message_view.private_message.deleted).toBe(true);

  // The GetPrivateMessages filters out deleted,
  // even though they are in the actual database.
  // no reason to show them
  const betaPms2 = await waitUntilSuccess(
    () => listNotifications(beta, "private_message"),
    p => p.items.length === betaPms1.items.length - 1,
  );
  expect(betaPms2.items.length).toBe(betaPms1.items.length - 1);

  // Undelete
  const undeletedPmRes = await deletePrivateMessage(
    alpha,
    false,
    pmRes.private_message_view.private_message.id,
  ).then(expectSuccess);
  expect(undeletedPmRes.private_message_view.private_message.deleted).toBe(
    false,
  );

  const betaPms3 = await waitUntilSuccess(
    () => listNotifications(beta, "private_message"),
    p => p.items.length === betaPms1.items.length,
  );
  expect(betaPms3.items.length).toBe(betaPms1.items.length);
});

test("Create a private message report", async () => {
  const pmRes = await createPrivateMessage(alpha, recipient_id).then(
    expectSuccess,
  );
  const betaPms1 = await waitUntilSuccess(
    () => listNotifications(beta, "private_message"),
    m =>
      !!m.items.find(
        e =>
          e.data.type_ == "private_message" &&
          e.data.private_message.ap_id ===
            pmRes.private_message_view.private_message.ap_id,
      ),
  );
  const betaPm = betaPms1.items[0].data as PrivateMessageView;
  expect(betaPm).toBeDefined();

  // Make sure that only the recipient can report it, so this should fail
  await jestLemmyError(
    () =>
      reportPrivateMessage(
        alpha,
        pmRes.private_message_view.private_message.id,
        "a reason",
      ).then(expectFailure),
    new LemmyError("couldnt_create", statusBadRequest),
  );

  // This one should pass
  const reason = "another reason";
  const report = await reportPrivateMessage(
    beta,
    betaPm.private_message.id,
    reason,
  ).then(expectSuccess);

  expect(report.private_message_report_view.private_message.id).toBe(
    betaPm.private_message.id,
  );
  expect(report.private_message_report_view.private_message_report.reason).toBe(
    reason,
  );

  const list_reports = (
    await waitUntilSuccess(
      () => listReports(alpha),
      r => r.items.some(r => r.type_ === "private_message"),
    )
  ).items.filter(r => r.type_ === "private_message");

  const r = list_reports[0] as PrivateMessageReportView;
  expect(r.private_message.ap_id).toBe(betaPm.private_message.ap_id);
  expect(r.private_message.content).toBe(betaPm.private_message.content);
  expect(r.private_message_creator.ap_id).toBe(betaPm.creator.ap_id);
  expect(r.private_message_report.reason).toBe(reason);
});
