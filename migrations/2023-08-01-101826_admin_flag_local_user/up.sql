alter table local_user add column admin boolean default false not null;

update local_user set admin = true from person where local_user.person_id = person.id and person.admin;

alter table person drop column admin;
