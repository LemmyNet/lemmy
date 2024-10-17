-- Renaming description to sidebar
ALTER TABLE community RENAME COLUMN description TO sidebar;

-- Adding a short description column
ALTER TABLE community
    ADD COLUMN description varchar(150);

