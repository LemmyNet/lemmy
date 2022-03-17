-- Delete the empty public keys
delete from community where public_key is null;
delete from person where public_key is null;

-- Make it required
alter table community alter column public_key set not null;
alter table person alter column public_key set not null;
