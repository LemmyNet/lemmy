<h1><img src="https://image.flaticon.com/icons/svg/194/194242.svg" width="50px" height="50px" /> Lemmy</h1>

[![Build Status](https://travis-ci.org/dessalines/lemmy.svg?branch=master)](https://travis-ci.org/dessalines/lemmy)
[![star this repo](http://githubbadges.com/star.svg?user=dessalines&repo=lemmy&style=flat)](https://github.com/dessalines/lemmy)
[![fork this repo](http://githubbadges.com/fork.svg?user=dessalines&repo=lemmy&style=flat)](https://github.com/dessalines/lemmy/fork)
[![GitHub issues](https://img.shields.io/github/issues-raw/dessalines/lemmy.svg)](https://github.com/dessalines/lemmy/issues)
![GitHub repo size](https://img.shields.io/github/repo-size/dessalines/lemmy.svg)
[![Matrix](https://img.shields.io/matrix/rust-reddit-fediverse:matrix.org.svg?label=matrix-chat)](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)
[![License](https://img.shields.io/github/license/dessalines/lemmy.svg)](LICENSE)

A link aggregator / reddit clone for the fediverse.

[Lemmy Dev instance](https://dev.lemmy.ml) *for testing purposes only*

This is a **very early beta version**, and a lot of features are currently broken or missing.

## Features
- Self hostable, easy to deploy.
  - Comes with docker.
- Open source.
- Live-updating Comment threads.
- Clean, minimal interface.
  - Mobile-friendly.
- Full vote scores `(+/-)` like old reddit.
- Full moderation.
  - Both site admins, and community moderators.
  - Can lock, remove, and restore posts.
  - Can remove and restore comments.
- High performance.
  - Server is written in rust.
  - Front end is `~80kB` gzipped.

## Why's it called Lemmy?
- Lead singer from [motorhead](https://invidio.us/watch?v=pWB5JZRGl0U).
- The old school [video game](https://en.wikipedia.org/wiki/Lemmings_(video_game)).
- The [furry rodents](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/).

Made with [Rust](https://www.rust-lang.org), [Actix](https://actix.rs/), [Inferno](https://www.infernojs.org), [Typescript](https://www.typescriptlang.org/) and [Diesel](http://diesel.rs/)

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

## Support
Support the development, and help cover hosting costs.
- Patreon
- bitcoin: `bc1queu73nwuheqtsp65nyh5hf4jr533r8rr5nsj75`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`

## Credits

Icons made by [Freepik](https://www.freepik.com/) licensed by [CC 3.0](http://creativecommons.org/licenses/by/3.0/)
