-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


-- Add columns to site table
alter table site drop column require_application;
alter table site drop column application_question;
alter table site drop column private_instance;

-- Add pending to local_user
alter table local_user drop column accepted_application;

drop table registration_application;
