### Tests

#### Rust

After installing [local development dependencies](contributing_local_development.md), run the
following commands in the `server` subfolder:

```bash
psql -U lemmy -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
export DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
diesel migration run
RUST_TEST_THREADS=1 cargo test
```

### Federation

Install the [Docker development dependencies](contributing_docker_development.md), and execute
`docker/federation-test/run-tests.sh`
