# Backup and Restore Guide

## Docker and Ansible

When using docker or ansible, there should be a `volumes` folder, which contains both the database, and all the pictures. Copy this folder to the new instance to restore your data.

### Incremental Database backup

To incrementally backup the DB to an `.sql` file, you can run: 

```bash
docker exec -t FOLDERNAME_postgres_1 pg_dumpall -c -U lemmy >  lemmy_dump_`date +%Y-%m-%d"_"%H_%M_%S`.sql
```
### A Sample backup script

```bash
#!/bin/sh
# DB Backup
ssh MY_USER@MY_IP "docker exec -t FOLDERNAME_postgres_1 pg_dumpall -c -U lemmy" >  ~/BACKUP_LOCATION/INSTANCE_NAME_dump_`date +%Y-%m-%d"_"%H_%M_%S`.sql

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

## More resources

- https://stackoverflow.com/questions/24718706/backup-restore-a-dockerized-postgresql-database


