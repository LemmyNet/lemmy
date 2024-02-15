ALTER TABLE local_site
    ADD COLUMN default_sort_type sort_type_enum DEFAULT 'Active' NOT NULL;

