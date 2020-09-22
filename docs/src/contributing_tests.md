### Tests

#### Rust

After installing [local development dependencies](contributing_local_development.md), run the
following commands:

```bash
psql -U lemmy -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
./test.sh
```

### Federation

Install the [Docker development dependencies](contributing_docker_development.md), and execute:

```
cd docker/federation
./run-tests.bash
```
