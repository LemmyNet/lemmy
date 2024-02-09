<div align="center">

![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/LemmyNet/lemmy.svg)
[![Build Status](https://woodpecker.join-lemmy.org/api/badges/LemmyNet/lemmy/status.svg)](https://woodpecker.join-lemmy.org/LemmyNet/lemmy)
[![GitHub issues](https://img.shields.io/github/issues-raw/LemmyNet/lemmy.svg)](https://github.com/LemmyNet/lemmy/issues)
[![Docker Pulls](https://img.shields.io/docker/pulls/dessalines/lemmy.svg)](https://cloud.docker.com/repository/docker/dessalines/lemmy/)
[![Translation status](http://weblate.join-lemmy.org/widgets/lemmy/-/lemmy/svg-badge.svg)](http://weblate.join-lemmy.org/engage/lemmy/)
[![License](https://img.shields.io/github/license/LemmyNet/lemmy.svg)](LICENSE)
![GitHub stars](https://img.shields.io/github/stars/LemmyNet/lemmy?style=social)
<a href="https://endsoftwarepatents.org/innovating-without-patents"><img style="height: 20px;" src="https://static.fsf.org/nosvn/esp/logos/patent-free.svg"></a>

</div>

<p align="center">
  <span>English</span> |
  <a href="readmes/README.es.md">Español</a> |
  <a href="readmes/README.ru.md">Русский</a> |
  <a href="readmes/README.zh.hans.md">汉语</a> |
  <a href="readmes/README.zh.hant.md">漢語</a> |
  <a href="readmes/README.ja.md">日本語</a>
</p>

<p align="center">
  <a href="https://join-lemmy.org/" rel="noopener">
 <img width=200px height=200px src="https://raw.githubusercontent.com/LemmyNet/lemmy-ui/main/src/assets/icons/favicon.svg"></a>

 <h3 align="center"><a href="https://join-lemmy.org">Lemmy</a></h3>
  <p align="center">
    A link aggregator and forum for the fediverse.
    <br />
    <br />
    <a href="https://join-lemmy.org">Join Lemmy</a>
    ·
    <a href="https://join-lemmy.org/docs/index.html">Documentation</a>
    ·
    <a href="https://matrix.to/#/#lemmy-space:matrix.org">Matrix Chat</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/issues">Report Bug</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/issues">Request Feature</a>
    ·
    <a href="https://github.com/LemmyNet/lemmy/blob/main/RELEASES.md">Releases</a>
    ·
    <a href="https://join-lemmy.org/docs/code_of_conduct.html">Code of Conduct</a>
  </p>
</p>

## About The Project

| Desktop                                                                                                         | Mobile                                                                                                      |
| --------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| ![desktop](https://raw.githubusercontent.com/LemmyNet/joinlemmy-site/main/src/assets/images/main_screen_2.webp) | ![mobile](https://raw.githubusercontent.com/LemmyNet/joinlemmy-site/main/src/assets/images/mobile_pic.webp) |

[Lemmy](https://github.com/LemmyNet/lemmy) is similar to sites like [Reddit](https://reddit.com), [Lobste.rs](https://lobste.rs), or [Hacker News](https://news.ycombinator.com/): you subscribe to forums you're interested in, post links and discussions, then vote, and comment on them. Behind the scenes, it is very different; anyone can easily run a server, and all these servers are federated (think email), and connected to the same universe, called the [Fediverse](https://en.wikipedia.org/wiki/Fediverse).

For a link aggregator, this means a user registered on one server can subscribe to forums on any other server, and can have discussions with users registered elsewhere.

It is an easily self-hostable, decentralized alternative to Reddit and other link aggregators, outside of their corporate control and meddling.

Each Lemmy server can set its own moderation policy; appointing site-wide admins, and community moderators to keep out the trolls, and foster a healthy, non-toxic environment where all can feel comfortable contributing.

### Why's it called Lemmy?

- Lead singer from [Motörhead](https://invidio.us/watch?v=3mbvWn1EY6g).
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
  - Comes with [Docker](https://join-lemmy.org/docs/administration/install_docker.html) and [Ansible](https://join-lemmy.org/docs/administration/install_ansible.html).
- Clean, mobile-friendly interface.
  - Only a minimum of a username and password is required to sign up!
  - User avatar support.
  - Live-updating Comment threads.
  - Full vote scores `(+/-)` like old Reddit.
  - Themes, including light, dark, and solarized.
  - Emojis with autocomplete support. Start typing `:`
  - User tagging using `@`, Community tagging using `!`.
  - Integrated image uploading in both posts and comments.
  - A post can consist of a title and any combination of self text, a URL, or nothing else.
  - Notifications, on comment replies and when you're tagged.
    - Notifications can be sent via email.
    - Private messaging support.
  - i18n / internationalization support.
  - RSS / Atom feeds for `All`, `Subscribed`, `Inbox`, `User`, and `Community`.
- Cross-posting support.
  - A _similar post search_ when creating new posts. Great for question / answer communities.
- Moderation abilities.
  - Public Moderation Logs.
  - Can sticky posts to the top of communities.
  - Both site admins, and community moderators, who can appoint other moderators.
  - Can lock, remove, and restore posts and comments.
  - Can ban and unban users from communities and the site.
  - Can transfer site and communities to others.
- Can fully erase your data, replacing all posts and comments.
- NSFW post / community support.
- High performance.
  - Server is written in rust.
  - Supports arm64 / Raspberry Pi.

## Installation

- [Lemmy Administration Docs](https://join-lemmy.org/docs/administration/administration.html)

## Lemmy Projects

- [awesome-lemmy - A community driven list of apps and tools for lemmy](https://github.com/dbeley/awesome-lemmy)

## Support / Donate

Lemmy is free, open-source software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project.

- [Support on Liberapay](https://liberapay.com/Lemmy).
- [Support on Patreon](https://www.patreon.com/dessalines).
- [Support on OpenCollective](https://opencollective.com/lemmy).
- [List of Sponsors](https://join-lemmy.org/donate).

### Crypto

- bitcoin: `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- monero: `41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`
- cardano: `addr1q858t89l2ym6xmrugjs0af9cslfwvnvsh2xxp6x4dcez7pf5tushkp4wl7zxfhm2djp6gq60dk4cmc7seaza5p3slx0sakjutm`

## Contributing

- [Contributing instructions](https://join-lemmy.org/docs/contributors/01-overview.html)
- [Docker Development](https://join-lemmy.org/docs/contributors/03-docker-development.html)
- [Local Development](https://join-lemmy.org/docs/contributors/02-local-development.html)

### Translations

- If you want to help with translating, take a look at [Weblate](https://weblate.join-lemmy.org/projects/lemmy/). You can also help by [translating the documentation](https://github.com/LemmyNet/lemmy-docs#adding-a-new-language).

## Contact

- [Mastodon](https://mastodon.social/@LemmyDev)
- [Lemmy Support Forum](https://lemmy.ml/c/lemmy_support)

## Code Mirrors

- [GitHub](https://github.com/LemmyNet/lemmy)
- [Gitea](https://git.join-lemmy.org/LemmyNet/lemmy)
- [Codeberg](https://codeberg.org/LemmyNet/lemmy)

## Credits

Logo made by Andy Cuccaro (@andycuccaro) under the CC-BY-SA 4.0 license.
