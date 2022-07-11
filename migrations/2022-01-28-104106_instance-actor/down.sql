-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


alter table site
    drop column actor_id,
    drop column last_refreshed_at,
    drop column inbox_url,
    drop column private_key,
    drop column public_key;
