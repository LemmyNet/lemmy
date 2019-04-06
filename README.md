# Lemmy

[![Build Status](https://travis-ci.org/dessalines/lemmy.svg?branch=master)](https://travis-ci.org/dessalines/lemmy)

A link aggregator / reddit clone for the fediverse.

Made with [Rust](https://www.rust-lang.org), [Actix](https://actix.rs/), [Inferno](https://www.infernojs.org), [Typescript](https://www.typescriptlang.org/).

## Navigation
- [Matrix Chatroom](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)
- [Issues / Feature Requests](https://github.com/dessalines/lemmy/issues)
- Support the development via Patreon

## Features
- TBD
## Install
### Docker
```
git clone https://github.com/dessalines/lemmy
cd lemmy
docker-compose up
```
and goto http://localhost:8080
### Local Development
#### Requirements
- [Rust](https://www.rust-lang.org/)
- [Yarn](https://yarnpkg.com/en/)
- [Postgres](https://www.sqlite.org/index.html)
#### Set up Postgres DB
```
 psql -c "create user rrr with password 'rrr' superuser;" -U postgres
 psql -c 'create database rrr with owner rrr;' -U postgres
```
#### Running
```
git clone https://github.com/dessalines/lemmy
cd lemmy
./install.sh
```
and goto http://localhost:8080

## Documentation
- [ActivityPub API.md](API.md)
- [Goals](goals.md)
- [Ranking Algorithm](ranking.md)

