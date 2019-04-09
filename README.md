<h1><img src="https://image.flaticon.com/icons/svg/194/194242.svg" width="50px" height="50px" /> Lemmy</h1>

[![Build Status](https://travis-ci.org/dessalines/lemmy.svg?branch=master)](https://travis-ci.org/dessalines/lemmy)
[![star this repo](http://githubbadges.com/star.svg?user=dessalines&repo=lemmy&style=flat)](https://github.com/dessalines/lemmy)
[![fork this repo](http://githubbadges.com/fork.svg?user=dessalines&repo=lemmy&style=flat)](https://github.com/dessalines/lemmy/fork)
[![GitHub issues](https://img.shields.io/github/issues-raw/dessalines/lemmy.svg)](https://github.com/dessalines/lemmy/issues)
![GitHub repo size](https://img.shields.io/github/repo-size/dessalines/lemmy.svg)
[![Matrix](https://img.shields.io/matrix/rust-reddit-fediverse:matrix.org.svg?label=matrix-chat)](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)
[![License](https://img.shields.io/github/license/dessalines/lemmy.svg)](LICENSE)

A link aggregator / reddit clone for the fediverse.

Made with [Rust](https://www.rust-lang.org), [Actix](https://actix.rs/), [Inferno](https://www.infernojs.org), [Typescript](https://www.typescriptlang.org/).

## Navigation
- [Matrix Chatroom](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)
- [Issues / Feature Requests](https://github.com/dessalines/lemmy/issues)
- Support the development via Patreon

## Features
- TBD
-
the name

Lead singer from motorhead.
The old school video game.
The furry rodents.

Goals r/ censorship

## Install
### Docker
```
git clone https://github.com/dessalines/lemmy
cd lemmy
docker-compose up
```
and goto http://localhost:8536
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
and goto http://localhost:8536

## Documentation
- [ActivityPub API.md](docs/API.md)
- [Goals](docs/goals.md)
- [Ranking Algorithm](docs/ranking.md)

## Credits

Icons made by [Freepik](https://www.freepik.com/) licensed by [CC 3.0](http://creativecommons.org/licenses/by/3.0/)
