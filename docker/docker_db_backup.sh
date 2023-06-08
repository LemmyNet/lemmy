#!/usr/bin/env bash
docker-compose exec postgres pg_dumpall -c -U lemmy > dump_`date +%Y-%m-%d"_"%H_%M_%S`.sql
