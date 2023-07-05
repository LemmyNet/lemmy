drop index if exists idx_person_admin;
create index idx_person_admin on person(admin) where admin; -- allow quickly finding all admins (PersonView::admins)