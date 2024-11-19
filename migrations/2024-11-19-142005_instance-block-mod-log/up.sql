alter table federation_blocklist add column admin_person_id int REFERENCES person(id) ON UPDATE CASCADE ON DELETE CASCADE;
alter table federation_blocklist add column reason text;
alter table federation_blocklist add column expires timestamptz;