ALTER TABLE local_user
    ADD COLUMN enable_private_messages boolean DEFAULT TRUE NOT NULL;

