<h1><img src="https://image.flaticon.com/icons/svg/194/194242.svg" width="50px" height="50px" /> Lemmy</h1>

![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/dessalines/lemmy.svg)
[![Build Status](https://travis-ci.org/dessalines/lemmy.svg?branch=master)](https://travis-ci.org/dessalines/lemmy)
![Docker Cloud Build Status](https://img.shields.io/docker/cloud/build/dessalines/lemmy.svg)
[![star this repo](http://githubbadges.com/star.svg?user=dessalines&repo=lemmy&style=flat)](https://github.com/dessalines/lemmy)
[![fork this repo](http://githubbadges.com/fork.svg?user=dessalines&repo=lemmy&style=flat)](https://github.com/dessalines/lemmy/fork)
![Docker Pulls](https://img.shields.io/docker/pulls/dessalines/lemmy.svg)
[![GitHub issues](https://img.shields.io/github/issues-raw/dessalines/lemmy.svg)](https://github.com/dessalines/lemmy/issues)
![GitHub repo size](https://img.shields.io/github/repo-size/dessalines/lemmy.svg)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/dessalines/lemmy.svg)
[![License](https://img.shields.io/github/license/dessalines/lemmy.svg)](LICENSE)
[![Mastodon](https://img.shields.io/badge/Mastodon-follow-lightgrey.svg)](https://mastodon.social/@LemmyDev)
[![Matrix](https://img.shields.io/matrix/rust-reddit-fediverse:matrix.org.svg?label=matrix-chat)](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)
[![Patreon](https://img.shields.io/badge/-Support%20on%20Patreon-blueviolet.svg)](https://www.patreon.com/dessalines)

A link aggregator / reddit clone for the fediverse.

[Lemmy Dev instance](https://dev.lemmy.ml) *for testing purposes only*

This is a **very early beta version**, and a lot of features are currently broken or in active development, such as federation.

Front Page|Post
---|---
![main screen](https://i.imgur.com/y64BtXC.png)|![chat screen](https://i.imgur.com/vsOr87q.png)
## Features
- Open source, [AGPL License](/LICENSE).
- Self hostable, easy to deploy.
  - Comes with [Docker](#docker).
- Live-updating Comment threads.
- Full vote scores `(+/-)` like old reddit.
- Moderation abilities.
  - Public Moderation Logs.
  - Both site admins, and community moderators, who can appoint other moderators.
  - Can lock, remove, and restore posts and comments.
  - Can ban and unban users from communities and the site.
- Clean, mobile-friendly interface.
- High performance.
  - Server is written in rust.
  - Front end is `~80kB` gzipped.
## About
[Lemmy](https://github.com/dessalines/lemmy) is similar to sites like [Reddit](https://reddit.com), [Lobste.rs](https://lobste.rs), [Raddle](https://raddle.me), or [Hacker News](https://news.ycombinator.com/): you subscribe to forums you're interested in, post links and discussions, then vote, and comment on them. Behind the scenes, it is very different; anyone can easily run a server, and all these servers are federated (think email), and connected to the same universe, called the [Fediverse](https://en.wikipedia.org/wiki/Fediverse).

For a link aggregator, this means a user registered on one server can subscribe to forums on any other server, and can have discussions with users registered elsewhere.

The overall goal is to create an easily self-hostable, decentralized alternative to reddit and other link aggregators, outside of their corporate control and meddling.

Each lemmy server can set its own moderation policy; appointing site-wide admins, and community moderators to keep out the trolls, and foster a healthy, non-toxic environment where all can feel comfortable contributing.

## Why's it called Lemmy?
- Lead singer from [motorhead](https://invidio.us/watch?v=pWB5JZRGl0U).
- The old school [video game](<https://en.wikipedia.org/wiki/Lemmings_(video_game)>).
- The [Koopa from Super Mario](https://www.mariowiki.com/Lemmy_Koopa).
- The [furry rodents](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/).

Made with [Rust](https://www.rust-lang.org), [Actix](https://actix.rs/), [Inferno](https://www.infernojs.org), [Typescript](https://www.typescriptlang.org/) and [Diesel](http://diesel.rs/).
## Install
### Docker
Make sure you have both docker and docker-compose(>=`1.24.0`) installed.

```
git clone https://github.com/dessalines/lemmy
cd lemmy/docker
docker-compose up -d
```

and goto http://localhost:8536

### Nginx Config
```
location / {
  rewrite (\/(user|u|inbox|post|community|c|login|search|sponsors|communities|modlog)+) /static/index.html break;
  proxy_pass http://0.0.0.0:8536;
  proxy_set_header X-Real-IP $remote_addr;
  proxy_set_header Host $host;
  proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

  # WebSocket support
  proxy_http_version 1.1;
  proxy_set_header Upgrade $http_upgrade;
  proxy_set_header Connection "upgrade";
}
```

## Develop
### Docker Development
```
git clone https://github.com/dessalines/lemmy
cd lemmy
./docker_update.sh # This pulls the newest version, builds and runs it
```

and goto http://localhost:8536
### Local Development
#### Requirements
- [Rust](https://www.rust-lang.org/)
- [Yarn](https://yarnpkg.com/en/)
- [Postgres](https://www.postgresql.org/)
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
# For live coding, where both the front and back end, automagically reload on any save, do:
# cd ui && yarn start
# cd server && cargo watch -x run
```

## Documentation
- [Websocket API for App developers](docs/api.md)
- [ActivityPub API.md](docs/apub_api_outline.md)
- [Goals](docs/goals.md)
- [Ranking Algorithm](docs/ranking.md)
## Support
Lemmy is free, open-source software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project.
- [Support on Patreon](https://www.patreon.com/dessalines).
- [Sponsor List](https://dev.lemmy.ml/sponsors).
- bitcoin: `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
## Credits
Icons made by [Freepik](https://www.freepik.com/) licensed by [CC 3.0](http://creativecommons.org/licenses/by/3.0/)
