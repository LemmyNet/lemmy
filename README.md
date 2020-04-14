<div align="center">

![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/LemmyNet/lemmy.svg)
[![Build Status](https://travis-ci.org/LemmyNet/lemmy.svg?branch=master)](https://travis-ci.org/LemmyNet/lemmy)
[![GitHub issues](https://img.shields.io/github/issues-raw/LemmyNet/lemmy.svg)](https://github.com/LemmyNet/lemmy/issues)
[![Docker Pulls](https://img.shields.io/docker/pulls/dessalines/lemmy.svg)](https://cloud.docker.com/repository/docker/dessalines/lemmy/)
[![Translation status](http://weblate.yerbamate.dev/widgets/lemmy/-/lemmy/svg-badge.svg)](http://weblate.yerbamate.dev/engage/lemmy/)
[![License](https://img.shields.io/github/license/LemmyNet/lemmy.svg)](LICENSE)
![GitHub stars](https://img.shields.io/github/stars/LemmyNet/lemmy?style=social)
</div>

<p align="center">
  <a href="https://dev.lemmy.ml/" rel="noopener">
 <img width=200px height=200px src="ui/assets/favicon.svg"></a>

 <h3 align="center"><a href="https://dev.lemmy.ml">Lemmy</a></h3>
  <p align="center">
    A link aggregator / reddit clone for the fediverse.
    <br />
    <br />
    <a href="https://dev.lemmy.ml">View Site</a>
    ·
    <a href="https://dev.lemmy.ml/docs/index.html">Documentation</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/issues">Report Bug</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/issues">Request Feature</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/blob/master/RELEASES.md">Releases</a>
  </p>
</p>

## About The Project

Front Page|Post
---|---
![main screen](https://i.imgur.com/kZSRcRu.png)|![chat screen](https://i.imgur.com/4XghNh6.png)

[Lemmy](https://github.com/LemmyNet/lemmy) is similar to sites like [Reddit](https://reddit.com), [Lobste.rs](https://lobste.rs), [Raddle](https://raddle.me), or [Hacker News](https://news.ycombinator.com/): you subscribe to forums you're interested in, post links and discussions, then vote, and comment on them. Behind the scenes, it is very different; anyone can easily run a server, and all these servers are federated (think email), and connected to the same universe, called the [Fediverse](https://en.wikipedia.org/wiki/Fediverse).

For a link aggregator, this means a user registered on one server can subscribe to forums on any other server, and can have discussions with users registered elsewhere.

The overall goal is to create an easily self-hostable, decentralized alternative to reddit and other link aggregators, outside of their corporate control and meddling.

Each lemmy server can set its own moderation policy; appointing site-wide admins, and community moderators to keep out the trolls, and foster a healthy, non-toxic environment where all can feel comfortable contributing.

*Note: Federation is still in active development*

### Why's it called Lemmy?

- Lead singer from [Motörhead](https://invidio.us/watch?v=pWB5JZRGl0U).
- The old school [video game](<https://en.wikipedia.org/wiki/Lemmings_(video_game)>).
- The [Koopa from Super Mario](https://www.mariowiki.com/Lemmy_Koopa).
- The [furry rodents](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/).

### Built With

- [Rust](https://www.rust-lang.org)
- [Actix](https://actix.rs/)
- [Diesel](http://diesel.rs/)
- [Inferno](https://infernojs.org)
- [Typescript](https://www.typescriptlang.org/)

## Features

- Open source, [AGPL License](/LICENSE).
- Self hostable, easy to deploy.
  - Comes with [Docker](#docker), [Ansible](#ansible), [Kubernetes](#kubernetes).
- Clean, mobile-friendly interface.
  - Only a minimum of a username and password is required to sign up!
  - User avatar support.
  - Live-updating Comment threads.
  - Full vote scores `(+/-)` like old reddit.
  - Themes, including light, dark, and solarized.
  - Emojis with autocomplete support. Start typing `:`
  - User tagging using `@`, Community tagging using `#`.
  - Integrated image uploading in both posts and comments.
  - A post can consist of a title and any combination of self text, a URL, or nothing else.
  - Notifications, on comment replies and when you're tagged.
    - Notifications can be sent via email.
    - Private messaging support.
  - i18n / internationalization support.
  - RSS / Atom feeds for `All`, `Subscribed`, `Inbox`, `User`, and `Community`.
- Cross-posting support.
  - A *similar post search* when creating new posts. Great for question / answer communities.
- Moderation abilities.
  - Public Moderation Logs.
  - Can sticky posts to the top of communities.
  - Both site admins, and community moderators, who can appoint other moderators.
  - Can lock, remove, and restore posts and comments.
  - Can ban and unban users from communities and the site.
  - Can transfer site and communities to others.
- Can fully erase your data, replacing all posts and comments.
- NSFW post / community support.
- OEmbed support via Iframely.
- High performance.
  - Server is written in rust.
  - Front end is `~80kB` gzipped.
  - Supports arm64 / Raspberry Pi.

## Installation

- [Docker](https://dev.lemmy.ml/docs/administration_install_docker.html)
- [Ansible](https://dev.lemmy.ml/docs/administration_install_ansible.html)
- [Kubernetes](https://dev.lemmy.ml/docs/administration_install_kubernetes.html)

## Support / Donate

Lemmy is free, open-source software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project.

- [Support on Liberapay](https://liberapay.com/Lemmy).
- [Support on Patreon](https://www.patreon.com/dessalines).
- [Support on OpenCollective](https://opencollective.com/lemmy).
- [List of Sponsors](https://dev.lemmy.ml/sponsors).

### Crypto

- bitcoin: `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- monero: `41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`

## Contributing

- [Contributing instructions](https://dev.lemmy.ml/docs/contributing.html)
- [Docker Development](https://dev.lemmy.ml/docs/contributing_docker_development.html)
- [Local Development](https://dev.lemmy.ml/docs/contributing_local_development.html)

### Translations 

If you want to help with translating, take a look at [Weblate](https://weblate.yerbamate.dev/projects/lemmy/).

## Contact

- [Mastodon](https://mastodon.social/@LemmyDev) - [![Mastodon Follow](https://img.shields.io/mastodon/follow/810572?domain=https%3A%2F%2Fmastodon.social&style=social)](https://mastodon.social/@LemmyDev)
- [Matrix](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org) - [![Matrix](https://img.shields.io/matrix/rust-reddit-fediverse:matrix.org.svg?label=matrix-chat)](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)
- [GitHub](https://github.com/LemmyNet/lemmy)
- [Gitea](https://yerbamate.dev/dessalines/lemmy)
- [GitLab](https://gitlab.com/dessalines/lemmy)

## Credits

Logo made by Andy Cuccaro (@andycuccaro) under the CC-BY-SA 4.0 license.
