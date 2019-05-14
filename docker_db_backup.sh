docker exec -it lemmy_db_1 pg_dumpall -c -U rrr > dump_`date +%d-%m-%Y"_"%H_%M_%S`.sql
