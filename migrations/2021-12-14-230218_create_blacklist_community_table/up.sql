-- Your SQL goes here

CREATE TABLE blacklist_community (
	id serial PRIMARY KEY,
	reason text,
	published timestamp not null default now(),
	creator_id int,
	community_id int UNIQUE,
    CONSTRAINT fk_creator
      FOREIGN KEY(creator_id)
	  REFERENCES person(id)
	  ON DELETE SET NULL,

	CONSTRAINT fk_community
      FOREIGN KEY(community_id)
	  REFERENCES community(id)
	  ON DELETE SET NULL
);