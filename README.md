<p align="center">
  <a href="" rel="noopener">
 <img width=200px height=200px src="ui/assets/favicon.svg"></a>
</p>

<h3 align="center">Lemmy</h3>

<div align="center">

![GitHub stars](https://img.shields.io/github/stars/dessalines/lemmy?style=social)
[![Mastodon Follow](https://img.shields.io/mastodon/follow/810572?domain=https%3A%2F%2Fmastodon.social&style=social)](https://mastodon.social/@LemmyDev)
[![Matrix](https://img.shields.io/matrix/rust-reddit-fediverse:matrix.org.svg?label=matrix-chat)](https://riot.im/app/#/room/#rust-reddit-fediverse:matrix.org)
![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/dessalines/lemmy.svg)
[![Build Status](https://travis-ci.org/dessalines/lemmy.svg?branch=master)](https://travis-ci.org/dessalines/lemmy)
[![GitHub issues](https://img.shields.io/github/issues-raw/dessalines/lemmy.svg)](https://github.com/dessalines/lemmy/issues)
[![Docker Pulls](https://img.shields.io/docker/pulls/dessalines/lemmy.svg)](https://cloud.docker.com/repository/docker/dessalines/lemmy/)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/dessalines/lemmy.svg)
![GitHub repo size](https://img.shields.io/github/repo-size/dessalines/lemmy.svg)
[![License](https://img.shields.io/github/license/dessalines/lemmy.svg)](LICENSE)
[![Patreon](https://img.shields.io/badge/-Support%20on%20Patreon-blueviolet.svg)](https://www.patreon.com/dessalines)
</div>

---

<p align="center">A link aggregator / reddit clone for the fediverse.
    <br> 
</p>

[Lemmy Dev instance](https://dev.lemmy.ml) *This data is being backed up, and once federation is working, it will be the basis for a main instance.*

This is a **very early beta version**, and a lot of features are currently broken or in active development, such as federation.

Front Page|Post
---|---
![main screen](https://i.imgur.com/kZSRcRu.png)|![chat screen](https://i.imgur.com/4XghNh6.png)

[Lemmy](https://github.com/dessalines/lemmy) is similar to sites like [Reddit](https://reddit.com), [Lobste.rs](https://lobste.rs), [Raddle](https://raddle.me), or [Hacker News](https://news.ycombinator.com/): you subscribe to forums you're interested in, post links and discussions, then vote, and comment on them. Behind the scenes, it is very different; anyone can easily run a server, and all these servers are federated (think email), and connected to the same universe, called the [Fediverse](https://en.wikipedia.org/wiki/Fediverse).

For a link aggregator, this means a user registered on one server can subscribe to forums on any other server, and can have discussions with users registered elsewhere.

The overall goal is to create an easily self-hostable, decentralized alternative to reddit and other link aggregators, outside of their corporate control and meddling.

Each lemmy server can set its own moderation policy; appointing site-wide admins, and community moderators to keep out the trolls, and foster a healthy, non-toxic environment where all can feel comfortable contributing.

Made with [Rust](https://www.rust-lang.org), [Actix](https://actix.rs/), [Inferno](https://infernojs.org), [Typescript](https://www.typescriptlang.org/) and [Diesel](http://diesel.rs/).

- [Documentation](https://dev.lemmy.ml/docs/index.html)
- [Releases / Changelog](/RELEASES.md)
- [Contributing](https://dev.lemmy.ml/docs/contributing.html)

## Repository Mirrors

- [GitHub](https://github.com/dessalines/lemmy)
- [Gitea](https://yerbamate.dev/dessalines/lemmy)
- [GitLab](https://gitlab.com/dessalines/lemmy)

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
- High performance.
  - Server is written in rust.
  - Front end is `~80kB` gzipped.
  - Supports arm64 / Raspberry Pi.

## Why's it called Lemmy?

- Lead singer from [Motörhead](https://invidio.us/watch?v=pWB5JZRGl0U).
- The old school [video game](<https://en.wikipedia.org/wiki/Lemmings_(video_game)>).
- The [Koopa from Super Mario](https://www.mariowiki.com/Lemmy_Koopa).
- The [furry rodents](http://sunchild.fpwc.org/lemming-the-little-giant-of-the-north/).

## Install

### Docker

Make sure you have both docker and docker-compose(>=`1.24.0`) installed:

```bash
mkdir lemmy/
cd lemmy/
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/docker-compose.yml
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/lemmy.hjson
# Edit lemmy.hjson to do more configuration
docker-compose up -d
```

and go to http://localhost:8536.

[A sample nginx config](/ansible/templates/nginx.conf), could be setup with:

```bash
wget https://raw.githubusercontent.com/dessalines/lemmy/master/ansible/templates/nginx.conf
# Replace the {{ vars }}
sudo mv nginx.conf /etc/nginx/sites-enabled/lemmy.conf
```
#### Updating

To update to the newest version, run:

```bash
wget https://raw.githubusercontent.com/dessalines/lemmy/master/docker/prod/docker-compose.yml
docker-compose up -d
```

### Ansible

First, you need to [install Ansible on your local computer](https://docs.ansible.com/ansible/latest/installation_guide/intro_installation.html) (e.g. using `sudo apt install ansible`) or the equivalent for you platform.

Then run the following commands on your local computer:

```bash
git clone https://github.com/dessalines/lemmy.git
cd lemmy/ansible/
cp inventory.example inventory
nano inventory # enter your server, domain, contact email
ansible-playbook lemmy.yml --become
```

## Support / Donate

Lemmy is free, open-source software, meaning no advertising, monetizing, or venture capital, ever. Your donations directly support full-time development of the project.

- [Support on Patreon](https://www.patreon.com/dessalines).
- [List of Sponsors](https://dev.lemmy.ml/sponsors).
- Soon to add either liberapay or opencollective.

### Crypto

- bitcoin: `1Hefs7miXS5ff5Ck5xvmjKjXf5242KzRtK`
- ethereum: `0x400c96c96acbC6E7B3B43B1dc1BB446540a88A01`
- monero: `41taVyY6e1xApqKyMVDRVxJ76sPkfZhALLTjRvVKpaAh2pBd4wv9RgYj1tSPrx8wc6iE1uWUfjtQdTmTy2FGMeChGVKPQuV`

## Translations 

If you'd like to add translations, take a look a look at the [English translation file](ui/src/translations/en.ts).

- Languages supported: Catalan, (`ca`), English (`en`), Chinese (`zh`), Dutch (`nl`), Esperanto (`eo`), Finnish (`fi`), French (`fr`), Spanish (`es`), Swedish (`sv`), German (`de`), Russian (`ru`), Italian (`it`).

<!-- translations -->

lang | done | missing
---- | ---- | -------
ca | 100% | 
de | 88% | create_private_message,send_secure_message,send_message,message,avatar,upload_avatar,show_avatars,docs,message_sent,messages,old_password,matrix_user_id,private_message_disclaimer,send_notifications_to_email,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,donate_to_lemmy,donate,from,logged_in,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message
eo | 76% | number_of_communities,create_private_message,send_secure_message,send_message,message,preview,upload_image,avatar,upload_avatar,show_avatars,formatting_help,view_source,sticky,unsticky,archive_link,stickied,delete_account,delete_account_confirm,banned,creator,number_online,docs,replies,mentions,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,theme,donate_to_lemmy,donate,from,are_you_sure,yes,no,logged_in,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message
es | 100% | 
fi | 100% | 
fr | 83% | create_private_message,send_secure_message,send_message,message,avatar,upload_avatar,show_avatars,archive_link,docs,replies,mentions,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,donate_to_lemmy,donate,from,logged_in,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message
it | 84% | create_private_message,send_secure_message,send_message,message,avatar,upload_avatar,show_avatars,archive_link,docs,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,donate_to_lemmy,donate,from,logged_in,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message
nl | 93% | create_private_message,send_secure_message,send_message,message,message_sent,messages,matrix_user_id,private_message_disclaimer,donate_to_lemmy,donate,from,logged_in,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message
ru | 72% | cross_posts,cross_post,number_of_communities,create_private_message,send_secure_message,send_message,message,preview,upload_image,avatar,upload_avatar,show_avatars,formatting_help,view_source,sticky,unsticky,archive_link,stickied,delete_account,delete_account_confirm,banned,creator,number_online,docs,replies,mentions,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,recent_comments,theme,donate_to_lemmy,donate,monero,by,to,from,transfer_community,transfer_site,are_you_sure,yes,no,logged_in,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message
sv | 83% | create_private_message,send_secure_message,send_message,message,avatar,upload_avatar,show_avatars,archive_link,docs,replies,mentions,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,donate_to_lemmy,donate,from,logged_in,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message
zh | 70% | cross_posts,cross_post,users,number_of_communities,create_private_message,send_secure_message,send_message,message,preview,upload_image,avatar,upload_avatar,show_avatars,formatting_help,view_source,sticky,unsticky,archive_link,settings,stickied,delete_account,delete_account_confirm,banned,creator,number_online,docs,replies,mentions,message_sent,messages,old_password,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,matrix_user_id,private_message_disclaimer,send_notifications_to_email,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,recent_comments,nsfw,show_nsfw,theme,donate_to_lemmy,donate,monero,by,to,from,transfer_community,transfer_site,are_you_sure,yes,no,logged_in,email_already_exists,couldnt_create_private_message,no_private_message_edit_allowed,couldnt_update_private_message
<!-- translationsstop -->

If you'd like to update this report, run:

```bash 
cd ui
ts-node translation_report.ts
```

## Credits

Logo made by Andy Cuccaro (@andycuccaro) under the CC-BY-SA 4.0 license.
