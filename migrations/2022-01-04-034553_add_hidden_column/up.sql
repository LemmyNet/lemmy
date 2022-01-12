alter table community add column "hidden" BOOLEAN DEFAULT FALSE;


CREATE TABLE IF NOT EXISTS mod_hide_community
(
   id serial primary key,
    community_id integer NOT NULL,
    person_id integer NOT NULL,
    when_ timestamp without time zone NOT NULL DEFAULT now(),
    reason text,
    CONSTRAINT mod_hide_community_id_fkey FOREIGN KEY (community_id)
        REFERENCES community (id) MATCH SIMPLE
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    CONSTRAINT mod_hide_community_person_id_fkey FOREIGN KEY (person_id)
        REFERENCES person (id) MATCH SIMPLE
        ON UPDATE CASCADE
        ON DELETE CASCADE
)

