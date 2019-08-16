docker exec -it dev_lemmy_db_1 pg_dumpall -c -U rrr > dump_`date +%Y-%m-%d"_"%H_%M_%S`.sql
