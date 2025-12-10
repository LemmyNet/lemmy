ALTER TABLE sent_activity
    ADD COLUMN send_person_followers_of int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE sent_activity
    ADD COLUMN send_multi_comm_followers_of int REFERENCES multi_community (id) ON UPDATE CASCADE ON DELETE CASCADE;

