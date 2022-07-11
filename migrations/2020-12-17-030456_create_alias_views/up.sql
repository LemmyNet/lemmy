-- SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
--
-- SPDX-License-Identifier: AGPL-3.0-only


-- Some view that act as aliases 
-- unfortunately necessary, since diesel doesn't have self joins
-- or alias support yet
create view user_alias_1 as select * from user_;
create view user_alias_2 as select * from user_;
create view comment_alias_1 as select * from comment;

