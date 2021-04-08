-- Renaming description to sidebar
alter table site rename column description to sidebar;

-- Adding a short description column
alter table site add column description varchar(150);
