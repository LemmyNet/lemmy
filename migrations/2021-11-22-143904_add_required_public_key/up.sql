-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


-- Delete the empty public keys
delete from community where public_key is null;
delete from person where public_key is null;

-- Make it required
alter table community alter column public_key set not null;
alter table person alter column public_key set not null;
