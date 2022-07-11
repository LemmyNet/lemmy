-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


alter table site alter column require_application set default true;
alter table site alter column application_question set default 'To verify that you are human, please explain why you want to create an account on this site';
