alter table person add column admin boolean default false not null;

update person set admin = true from local_user where local_user.person_id = person.id and local_user.admin;

alter table local_user drop column admin;
