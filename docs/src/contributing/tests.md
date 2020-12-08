### Tests

#### Rust

After installing [local development dependencies](local_development.md), run the
following commands:

```bash
psql -U lemmy -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
./test.sh
```

### Federation

Install the [Docker development dependencies](docker_development.md), and execute:

```
cd docker/federation
./run-tests.bash
```
