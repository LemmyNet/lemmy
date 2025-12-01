ALTER TABLE local_image
    ADD CONSTRAINT image_upload_local_user_id_not_null NOT NULL local_user_id;

