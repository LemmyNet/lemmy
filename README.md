# Federated Library of Things

## About The Project

This repo aspires to make the first online & global [Library of Things](https://en.wikipedia.org/wiki/Library_of_things).

People upload pictures of their items to the server, and people request those items be sent to them. The owner sends the item over the mail, the leaser pays shipping, and now the item has a leaser.

If people fail to either re-lease the item, or return the item (optional), within the leasing period, they will be required to purchase the item, set by the original owner. The admins get a percentage of the purchase fee. That makes hosting a FLoT server self-funding as an actual buisness.

Whaaatttt, moderators and admins can make monneeeyyy!? Like a jooobbb?! Yes like a job. Moderation and administration is work. And server admins get to set and moderate everything. They can choose who joins (users really should be vetted), what their server allows you to lease (think themes), and can have all their own community moderation tools.

Why do this? Because consumerism is cancer. It's bad for your minimalist zen, and it's bad for your wallet. Instead of buying a drill on amazon, lease it on your neighborhood or city's FLoT instance.

## Technicalities

This requires an optional payment service. I hope to make this a pluggable interface.

This also can benefit from an optional user identification service. Again, this can be pluggable. In its most bare form, this could be an ask to email the admin with a picture of your drivers license so they can verify your address. In its maximum form, it could be something like [stamp protocol](https://github.com/stamp-protocol) (FOSS, Decentralized, Novel) or something like [Plaid](https://plaid.com/products/identity-verification/) (Proprietary, Centralized, Professional).

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

## Support / Donate to Lemmy

This project is a fork of Lemmy, please consider supporting them. We frequently merge their changes upstream.

Lemmy is free, open-source software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project.

Lemmy is made possible by a generous grant from the [NLnet foundation](https://nlnet.nl/).

- [Support on Liberapay](https://liberapay.com/Lemmy).
- [Support on Patreon](https://www.patreon.com/dessalines).
- [Support on OpenCollective](https://opencollective.com/lemmy).
- [List of Sponsors](https://join-lemmy.org/donate).

### Crypto

- bitcoin: `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- monero: `41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`

## Contributing

Read the following documentation to setup the development environment and start coding:

- [Contributing instructions](https://join-lemmy.org/docs/contributors/01-overview.html)
- [Docker Development](https://join-lemmy.org/docs/contributors/03-docker-development.html)
- [Local Development](https://join-lemmy.org/docs/contributors/02-local-development.html)

When working on an issue or pull request, you can comment with any questions you may have so that maintainers can answer them. You can also join the [Matrix Development Chat](https://matrix.to/#/#lemmydev:matrix.org) for general assistance.

## Community

- [Usufruct Commons](https://librarysocialism.com/)
