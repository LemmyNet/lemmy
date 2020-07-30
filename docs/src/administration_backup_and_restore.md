# Backup and Restore Guide

## Docker and Ansible

When using docker or ansible, there should be a `volumes` folder, which contains both the database, and all the pictures. Copy this folder to the new instance to restore your data.

### Incremental Database backup

To incrementally backup the DB to an `.sql` file, you can run: 

```bash
docker-compose exec postgres pg_dumpall -c -U lemmy >  lemmy_dump_`date +%Y-%m-%d"_"%H_%M_%S`.sql
```
### A Sample backup script

```bash
#!/bin/sh
# DB Backup
ssh MY_USER@MY_IP "docker-compose exec postgres pg_dumpall -c -U lemmy" >  ~/BACKUP_LOCATION/INSTANCE_NAME_dump_`date +%Y-%m-%d"_"%H_%M_%S`.sql

# Volumes folder Backup
rsync -avP -zz --rsync-path="sudo rsync" MY_USER@MY_IP:/LEMMY_LOCATION/volumes ~/BACKUP_LOCATION/FOLDERNAME
```

### Restoring the DB

If you need to restore from a `pg_dumpall` file, you need to first clear out your existing database

```bash
# Drop the existing DB
docker exec -i FOLDERNAME_postgres_1 psql -U lemmy -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

# Restore from the .sql backup
cat db_dump.sql  |  docker exec -i FOLDERNAME_postgres_1 psql -U lemmy # restores the db

# This also might be necessary when doing a db import with a different password.
docker exec -i FOLDERNAME_postgres_1 psql -U lemmy -c "alter user lemmy with password 'bleh'"
```

### Changing your domain name

If you haven't federated yet, you can change your domain name in the DB. **Warning: do not do this after you've federated, or it will break federation.**

Get into `psql` for your docker: 

`docker-compose exec postgres psql -U lemmy`

```
-- Post
update post set ap_id = replace (ap_id, 'old_domain', 'new_domain');
update post set url = replace (url, 'old_domain', 'new_domain');
update post set body = replace (body, 'old_domain', 'new_domain');
update post set thumbnail_url = replace (thumbnail_url, 'old_domain', 'new_domain');

delete from post_aggregates_fast;
insert into post_aggregates_fast select * from post_aggregates_view;

-- Comments
update comment set ap_id = replace (ap_id, 'old_domain', 'new_domain');
update comment set content = replace (content, 'old_domain', 'new_domain');

delete from comment_aggregates_fast;
insert into comment_aggregates_fast select * from comment_aggregates_view;

-- User
update user_ set actor_id = replace (actor_id, 'old_domain', 'new_domain');
update user_ set avatar = replace (avatar, 'old_domain', 'new_domain');

delete from user_fast;
insert into user_fast select * from user_view;

-- Community
update community set actor_id = replace (actor_id, 'old_domain', 'new_domain');

delete from community_aggregates_fast;
insert into community_aggregates_fast select * from community_aggregates_view;
```

## More resources

- https://stackoverflow.com/questions/24718706/backup-restore-a-dockerized-postgresql-database


