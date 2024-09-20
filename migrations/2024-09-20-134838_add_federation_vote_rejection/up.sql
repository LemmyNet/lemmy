ALTER TABLE local_site
    ADD COLUMN reject_federated_upvotes boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN reject_federated_downvotes boolean DEFAULT FALSE NOT NULL;

