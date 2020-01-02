#### Requirements

- [Rust](https://www.rust-lang.org/)
- [Yarn](https://yarnpkg.com/en/)
- [Postgres](https://www.postgresql.org/)

#### Set up Postgres DB

```bash
 psql -c "create user lemmy with password 'password' superuser;" -U postgres
 psql -c 'create database lemmy with owner lemmy;' -U postgres
 export DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
```

#### Running

```bash
git clone https://github.com/dessalines/lemmy
cd lemmy
./install.sh
# For live coding, where both the front and back end, automagically reload on any save, do:
# cd ui && yarn start
# cd server && cargo watch -x run
```
