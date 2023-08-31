-- Renaming description to sidebar
ALTER TABLE site RENAME COLUMN description TO sidebar;

-- Adding a short description column
ALTER TABLE site
    ADD COLUMN description varchar(150);

