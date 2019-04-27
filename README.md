<h1><img src="https://image.flaticon.com/icons/svg/194/194242.svg" width="50px" height="50px" /> Lemmy</h1>

[![Build Status](https://travis-ci.org/dessalines/lemmy.svg?branch=master)](https://travis-ci.org/dessalines/lemmy)
[![star this repo](http://githubbadges.com/star.svg?user=dessalines&repo=lemmy&style=flat)](https://github.com/dessalines/lemmy)
[![fork this repo](http://githubbadges.com/fork.svg?user=dessalines&repo=lemmy&style=flat)](https://github.com/dessalines/lemmy/fork)
[![GitHub issues](https://img.shields.io/github/issues-raw/dessalines/lemmy.svg)](https://github.com/dessalines/lemmy/issues)
![GitHub repo size](https://img.shields.io/github/repo-size/dessalines/lemmy.svg)
[![Matrix](https://img.shields.io/matrix/rust-reddit-fediverse:matrix.org.svg?label=matrix-chat)](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)
[![Patreon](https://img.shields.io/badge/-Support%20on%20Patreon-blueviolet.svg)](https://www.patreon.com/dessalines)
[![License](https://img.shields.io/github/license/dessalines/lemmy.svg)](LICENSE)

A link aggregator / reddit clone for the fediverse.

[Lemmy Dev instance](https://dev.lemmy.ml) *for testing purposes only*

This is a **very early beta version**, and a lot of features are currently broken or in active development, such as federation.

|Front Page|Post|
|-----------------------------------------------|----------------------------------------------- |
|![main screen](https://i.imgur.com/y64BtXC.png)|![chat screen](https://i.imgur.com/vsOr87q.png) |

## Features

- Open source, [AGPL License](/LICENSE).
- Self hostable, easy to deploy.
  - Comes with [docker](#docker).
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
- The [furry rodents](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/).

Made with [Rust](https://www.rust-lang.org), [Actix](https://actix.rs/), [Inferno](https://www.infernojs.org), [Typescript](https://www.typescriptlang.org/) and [Diesel](http://diesel.rs/).

## Usage

### Production

#### Docker

Make sure you have both docker and docker-compose installed.

```
git clone https://github.com/dessalines/lemmy
cd lemmy
./docker_update.sh # This pulls the newest version, builds and runs it
```

and goto http://localhost:8536

<!-- #### Kubernetes (WIP)

> TODO: Add production version with proper proxy setup and Ingress for WebSockets

```bash
skaffold run -p lemmy--prod
# Now go to http://${IP}:30002
``` -->

### Development

#### Kubernetes

##### Requirements

- Local or remote Kubernetes cluster, i.e. [`minikube`](https://kubernetes.io/docs/tasks/tools/install-minikube/)
- [`kubectl`](https://kubernetes.io/docs/tasks/tools/install-kubectl/)
- [`skaffold`](https://skaffold.dev/)

##### Running

```bash
skaffold dev -p lemmy--dev
```

And goto http://localhost:4444 (automatically proxies to localhost, both if the cluster is local or remote).

It hot-reloads the UI and automatically recompiles the server.

#### Non-Kubernetes

##### Requirements

- [Rust](https://www.rust-lang.org/)
- [Yarn](https://yarnpkg.com/en/)
- [Postgres](https://www.sqlite.org/index.html)

##### Set up Postgres DB

```
 psql -c "create user rrr with password 'rrr' superuser;" -U postgres
 psql -c 'create database rrr with owner rrr;' -U postgres
```

##### Running

```
git clone https://github.com/dessalines/lemmy
cd lemmy
./install.sh
# For live coding, where both the front and back end, automagically reload on any save, do:
# cd ui && yarn start
# cd server && cargo watch -x run
```

and goto http://localhost:8536

## Documentation

- [ActivityPub API.md](docs/API.md)
- [Goals](docs/goals.md)
- [Ranking Algorithm](docs/ranking.md)

## Support

Lemmy is free, open-source software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project.

- [Support on Patreon](https://www.patreon.com/dessalines).
- [Sponsor List](https://dev.lemmy.ml/#/sponsors).
- bitcoin: `bc1queu73nwuheqtsp65nyh5hf4jr533r8rr5nsj75`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`

## Credits

Icons made by [Freepik](https://www.freepik.com/) licensed by [CC 3.0](http://creativecommons.org/licenses/by/3.0/)
