-- Some view that act as aliases 
-- unfortunately necessary, since diesel doesn't have self joins
-- or alias support yet
create view user_alias_1 as select * from user_;
create view user_alias_2 as select * from user_;
create view comment_alias_1 as select * from comment;

